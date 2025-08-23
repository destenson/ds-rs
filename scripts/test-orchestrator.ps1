<#
.SYNOPSIS
    Test Orchestrator for ds-rs project (PowerShell version)
    
.DESCRIPTION
    Manages test scenarios, RTSP servers, and test environments for the ds-rs project
    
.PARAMETER Scenario
    Test scenario to run (default: all)
    
.PARAMETER Config
    Path to configuration file (default: scripts/config/test-scenarios.toml)
    
.PARAMETER List
    List available test scenarios
    
.PARAMETER Verbose
    Enable verbose output
    
.EXAMPLE
    .\test-orchestrator.ps1 -Scenario unit
    .\test-orchestrator.ps1 -Scenario integration -Verbose
    .\test-orchestrator.ps1 -List
#>

[CmdletBinding()]
param(
    [string]$Scenario = "all",
    [string]$Config = "scripts\config\test-scenarios.toml",
    [switch]$List,
    [switch]$DryRun
)

# Script configuration
$ErrorActionPreference = "Stop"
$Script:ProjectRoot = Split-Path -Parent $PSScriptRoot
$Script:ProcessList = @{}
$Script:TestResults = @()

# Import helper functions
$HelperPath = Join-Path $PSScriptRoot "lib\test-helpers.ps1"
if (Test-Path $HelperPath) {
    . $HelperPath
}

# Load TOML parser module
function Import-TomlModule {
    try {
        Import-Module powershell-yaml -ErrorAction Stop
    } catch {
        Write-Host "Installing powershell-yaml module..." -ForegroundColor Yellow
        Install-Module -Name powershell-yaml -Force -Scope CurrentUser
        Import-Module powershell-yaml
    }
}

# Parse TOML configuration (simplified parser)
function Read-TomlConfig {
    param([string]$Path)
    
    # For now, we'll use a simple approach and call Python if available
    if (Get-Command python -ErrorAction SilentlyContinue) {
        $pythonScript = @"
import tomli
import json
import sys
with open(r'$Path', 'rb') as f:
    config = tomli.load(f)
print(json.dumps(config))
"@
        
        $tempPy = [System.IO.Path]::GetTempFileName() + ".py"
        $pythonScript | Out-File -FilePath $tempPy -Encoding UTF8
        
        try {
            $jsonOutput = python $tempPy 2>$null
            $config = $jsonOutput | ConvertFrom-Json
            return $config
        } finally {
            Remove-Item $tempPy -ErrorAction SilentlyContinue
        }
    }
    
    # Fallback: Basic TOML parsing (limited functionality)
    Write-Warning "Python not available, using basic TOML parser"
    $config = @{
        scenarios = @{}
        defaults = @{}
    }
    
    $currentSection = $null
    $content = Get-Content $Path
    
    foreach ($line in $content) {
        $line = $line.Trim()
        
        # Skip comments and empty lines
        if ($line -match '^#' -or $line -eq '') { continue }
        
        # Section header
        if ($line -match '^\[(.+)\]$') {
            $currentSection = $matches[1]
            if ($currentSection -match '^scenarios\.(.+)$') {
                $scenarioName = $matches[1]
                if (-not $config.scenarios.ContainsKey($scenarioName)) {
                    $config.scenarios[$scenarioName] = @{
                        steps = @()
                    }
                }
            }
        }
        # Key-value pair
        elseif ($line -match '^(\w+)\s*=\s*(.+)$') {
            $key = $matches[1]
            $value = $matches[2].Trim('"', "'")
            
            if ($currentSection -match '^scenarios\.(.+)$') {
                $scenarioName = $matches[1]
                $config.scenarios[$scenarioName][$key] = $value
            }
        }
    }
    
    return $config
}

# Check if a TCP port is available
function Test-TcpPort {
    param(
        [string]$Host = "127.0.0.1",
        [int]$Port,
        [int]$Timeout = 5
    )
    
    try {
        $tcpClient = New-Object System.Net.Sockets.TcpClient
        $connect = $tcpClient.BeginConnect($Host, $Port, $null, $null)
        $wait = $connect.AsyncWaitHandle.WaitOne($Timeout * 1000, $false)
        
        if ($wait) {
            $tcpClient.EndConnect($connect)
            $tcpClient.Close()
            return $true
        } else {
            $tcpClient.Close()
            return $false
        }
    } catch {
        return $false
    }
}

# Start a background process
function Start-BackgroundProcess {
    param(
        [string]$Name,
        [string]$Command,
        [string]$WorkingDirectory = $null,
        [hashtable]$Environment = @{}
    )
    
    Write-Host "Starting process: $Name" -ForegroundColor Cyan
    Write-Verbose "Command: $Command"
    
    $processInfo = New-Object System.Diagnostics.ProcessStartInfo
    $processInfo.FileName = "cmd.exe"
    $processInfo.Arguments = "/c $Command"
    $processInfo.UseShellExecute = $false
    $processInfo.RedirectStandardOutput = $true
    $processInfo.RedirectStandardError = $true
    $processInfo.CreateNoWindow = $true
    
    if ($WorkingDirectory) {
        $processInfo.WorkingDirectory = $WorkingDirectory
    }
    
    # Set environment variables
    foreach ($key in $Environment.Keys) {
        $processInfo.EnvironmentVariables[$key] = $Environment[$key]
    }
    
    $process = New-Object System.Diagnostics.Process
    $process.StartInfo = $processInfo
    
    if ($process.Start()) {
        $Script:ProcessList[$Name] = $process
        return $process
    } else {
        throw "Failed to start process: $Name"
    }
}

# Stop a background process
function Stop-BackgroundProcess {
    param([string]$Name)
    
    if ($Script:ProcessList.ContainsKey($Name)) {
        $process = $Script:ProcessList[$Name]
        
        Write-Host "Stopping process: $Name" -ForegroundColor Yellow
        
        try {
            if (-not $process.HasExited) {
                $process.Kill()
                $process.WaitForExit(5000)
            }
        } catch {
            Write-Warning "Error stopping process $Name: $_"
        }
        
        $Script:ProcessList.Remove($Name)
    }
}

# Clean up all processes on exit
function Stop-AllProcesses {
    foreach ($name in @($Script:ProcessList.Keys)) {
        Stop-BackgroundProcess -Name $name
    }
}

# Register cleanup on exit
Register-EngineEvent -SourceIdentifier PowerShell.Exiting -Action {
    Stop-AllProcesses
}

# Run a test command
function Invoke-TestCommand {
    param(
        [hashtable]$Step,
        [hashtable]$ScenarioEnv = @{}
    )
    
    $name = if ($Step.name) { $Step.name } else { "unnamed" }
    $command = $Step.command
    $cwd = if ($Step.cwd) { Join-Path $Script:ProjectRoot $Step.cwd } else { $Script:ProjectRoot }
    $expectedExitCode = if ($Step.expected_exit_code) { $Step.expected_exit_code } else { 0 }
    $retryCount = if ($Step.retry_count) { $Step.retry_count } else { 0 }
    $allowFailure = if ($Step.allow_failure) { $Step.allow_failure } else { $false }
    $timeout = if ($Step.timeout) { $Step.timeout } else { 300 }
    
    Write-Host "`nRunning: $name" -ForegroundColor Green
    Write-Verbose "Command: $command"
    Write-Verbose "CWD: $cwd"
    
    # Merge environment variables
    $env = @{}
    $ScenarioEnv.GetEnumerator() | ForEach-Object { $env[$_.Key] = $_.Value }
    if ($Step.env) {
        $Step.env.GetEnumerator() | ForEach-Object { $env[$_.Key] = $_.Value }
    }
    
    # Run with retries
    for ($attempt = 0; $attempt -le $retryCount; $attempt++) {
        if ($attempt -gt 0) {
            Write-Host "Retry attempt $attempt/$retryCount" -ForegroundColor Yellow
        }
        
        try {
            Push-Location $cwd
            
            # Set environment variables for this command
            $originalEnv = @{}
            foreach ($key in $env.Keys) {
                $originalEnv[$key] = [Environment]::GetEnvironmentVariable($key)
                [Environment]::SetEnvironmentVariable($key, $env[$key])
            }
            
            # Run the command
            $process = Start-Process -FilePath "cmd.exe" `
                -ArgumentList "/c", $command `
                -WorkingDirectory $cwd `
                -PassThru `
                -Wait `
                -NoNewWindow `
                -RedirectStandardOutput "$env:TEMP\test-output.txt" `
                -RedirectStandardError "$env:TEMP\test-error.txt"
            
            $exitCode = $process.ExitCode
            
            # Restore environment variables
            foreach ($key in $originalEnv.Keys) {
                [Environment]::SetEnvironmentVariable($key, $originalEnv[$key])
            }
            
            Pop-Location
            
            if ($exitCode -eq $expectedExitCode) {
                Write-Host "✓ $name completed successfully" -ForegroundColor Green
                return $true
            } elseif ($allowFailure) {
                Write-Warning "⚠ $name failed but marked as allow_failure"
                return $true
            } else {
                Write-Host "✗ $name failed with exit code $exitCode" -ForegroundColor Red
                
                if ($VerbosePreference -eq 'Continue') {
                    $output = Get-Content "$env:TEMP\test-output.txt" -ErrorAction SilentlyContinue
                    $error = Get-Content "$env:TEMP\test-error.txt" -ErrorAction SilentlyContinue
                    if ($output) { Write-Verbose "stdout: $output" }
                    if ($error) { Write-Verbose "stderr: $error" }
                }
            }
            
        } catch {
            Write-Host "✗ $name failed with error: $_" -ForegroundColor Red
            Pop-Location
        }
    }
    
    return $false
}

# Setup a test scenario
function Initialize-Scenario {
    param([hashtable]$Scenario)
    
    if (-not $Scenario.setup) { return $true }
    
    $setup = $Scenario.setup
    
    # Start RTSP server if needed
    if ($setup.rtsp_server -and $setup.rtsp_server.enabled) {
        $serverConfig = $setup.rtsp_server
        
        Start-BackgroundProcess `
            -Name "rtsp_server" `
            -Command $serverConfig.command `
            -WorkingDirectory (Join-Path $Script:ProjectRoot $serverConfig.cwd)
        
        # Wait for startup
        $startupDelay = if ($serverConfig.startup_delay) { $serverConfig.startup_delay } else { 5 }
        Write-Host "Waiting $startupDelay seconds for RTSP server to start..." -ForegroundColor Yellow
        Start-Sleep -Seconds $startupDelay
        
        # Health check
        if ($serverConfig.health_check) {
            $hc = $serverConfig.health_check
            $port = $hc.port
            $timeout = if ($hc.timeout) { $hc.timeout } else { 10 }
            
            Write-Host "Checking RTSP server health..." -ForegroundColor Yellow
            
            $elapsed = 0
            while ($elapsed -lt $timeout) {
                if (Test-TcpPort -Port $port -Timeout 1) {
                    Write-Host "RTSP server is ready" -ForegroundColor Green
                    return $true
                }
                Start-Sleep -Seconds 1
                $elapsed++
            }
            
            Write-Host "RTSP server health check failed" -ForegroundColor Red
            return $false
        }
    }
    
    # Create test files if needed
    if ($setup.test_files -and $setup.test_files.enabled) {
        foreach ($file in $setup.test_files.files) {
            $pattern = $file.pattern
            $duration = $file.duration
            $output = $file.output
            
            $command = "cargo run -- generate --pattern $pattern --duration $duration --output $output"
            $cwd = Join-Path $Script:ProjectRoot "crates\source-videos"
            
            Write-Host "Generating test file: $output" -ForegroundColor Cyan
            
            Push-Location $cwd
            $result = & cmd /c $command 2>&1
            $exitCode = $LASTEXITCODE
            Pop-Location
            
            if ($exitCode -ne 0) {
                Write-Host "Failed to generate test file: $output" -ForegroundColor Red
                return $false
            }
        }
    }
    
    return $true
}

# Cleanup after a test scenario
function Clear-Scenario {
    param([hashtable]$Scenario)
    
    if (-not $Scenario.cleanup) { return }
    
    $cleanup = $Scenario.cleanup
    
    # Stop RTSP server
    if ($cleanup.stop_rtsp_server) {
        Stop-BackgroundProcess -Name "rtsp_server"
    }
    
    # Remove test files
    if ($cleanup.remove_test_files) {
        $testFiles = @("test_smpte.mp4", "test_ball.mp4")
        foreach ($file in $testFiles) {
            $filePath = Join-Path $Script:ProjectRoot "crates\ds-rs\$file"
            if (Test-Path $filePath) {
                Write-Host "Removing test file: $file" -ForegroundColor Yellow
                Remove-Item $filePath -ErrorAction SilentlyContinue
            }
        }
    }
}

# Run a test scenario
function Invoke-Scenario {
    param(
        [string]$ScenarioName,
        [hashtable]$Config
    )
    
    if (-not $Config.scenarios.ContainsKey($ScenarioName)) {
        Write-Host "Scenario '$ScenarioName' not found" -ForegroundColor Red
        return $false
    }
    
    $scenario = $Config.scenarios[$ScenarioName]
    
    Write-Host "`n$('='*60)" -ForegroundColor Cyan
    Write-Host "Running scenario: $ScenarioName" -ForegroundColor Cyan
    $description = if ($scenario.description) { $scenario.description } else { "No description" }
    Write-Host "Description: $description" -ForegroundColor Cyan
    Write-Host "$('='*60)`n" -ForegroundColor Cyan
    
    # Check if this scenario includes other scenarios
    if ($scenario.include_scenarios) {
        $success = $true
        foreach ($included in $scenario.include_scenarios) {
            if (-not (Invoke-Scenario -ScenarioName $included -Config $Config)) {
                $success = $false
                if (-not $scenario.continue_on_failure) {
                    return $false
                }
            }
        }
        return $success
    }
    
    # Setup
    if (-not (Initialize-Scenario -Scenario $scenario)) {
        Write-Host "Scenario setup failed" -ForegroundColor Red
        Clear-Scenario -Scenario $scenario
        return $false
    }
    
    # Get scenario environment variables
    $scenarioEnv = if ($scenario.env) { $scenario.env } else { @{} }
    
    # Run steps
    $steps = if ($scenario.steps) { $scenario.steps } else { @() }
    $success = $true
    
    foreach ($step in $steps) {
        if (-not (Invoke-TestCommand -Step $step -ScenarioEnv $scenarioEnv)) {
            $success = $false
            if (-not $scenario.continue_on_failure) {
                break
            }
        }
    }
    
    # Cleanup
    Clear-Scenario -Scenario $scenario
    
    # Report results
    if ($success) {
        Write-Host "`n✓ Scenario '$ScenarioName' completed successfully" -ForegroundColor Green
    } else {
        Write-Host "`n✗ Scenario '$ScenarioName' failed" -ForegroundColor Red
    }
    
    $Script:TestResults += @{
        scenario = $ScenarioName
        success = $success
    }
    
    return $success
}

# List available scenarios
function Show-Scenarios {
    param([hashtable]$Config)
    
    Write-Host "`nAvailable test scenarios:" -ForegroundColor Cyan
    Write-Host ("-" * 60) -ForegroundColor Cyan
    
    foreach ($name in $Config.scenarios.Keys) {
        $scenario = $Config.scenarios[$name]
        $description = if ($scenario.description) { $scenario.description } else { "No description" }
        Write-Host ("  {0,-20} - {1}" -f $name, $description)
    }
    
    Write-Host ""
}

# Print test summary
function Show-Summary {
    Write-Host "`n$('='*60)" -ForegroundColor Cyan
    Write-Host "TEST EXECUTION SUMMARY" -ForegroundColor Cyan
    Write-Host "$('='*60)" -ForegroundColor Cyan
    
    $total = $Script:TestResults.Count
    $passed = ($Script:TestResults | Where-Object { $_.success }).Count
    $failed = $total - $passed
    
    foreach ($result in $Script:TestResults) {
        $status = if ($result.success) { "✓ PASSED" } else { "✗ FAILED" }
        $color = if ($result.success) { "Green" } else { "Red" }
        Write-Host ("  {0,-20} - {1}" -f $result.scenario, $status) -ForegroundColor $color
    }
    
    Write-Host ("-" * 60) -ForegroundColor Cyan
    Write-Host "Total: $total, Passed: $passed, Failed: $failed"
    
    if ($failed -eq 0) {
        Write-Host "`n✓ All tests passed!" -ForegroundColor Green
    } else {
        Write-Host "`n✗ $failed test(s) failed" -ForegroundColor Red
    }
}

# Main execution
try {
    # Find config file
    $configPath = $Config
    if (-not [System.IO.Path]::IsPathRooted($configPath)) {
        $configPath = Join-Path $Script:ProjectRoot $configPath
    }
    
    if (-not (Test-Path $configPath)) {
        Write-Host "Configuration file not found: $configPath" -ForegroundColor Red
        exit 1
    }
    
    # Load configuration
    Write-Verbose "Loading configuration from: $configPath"
    $configData = Read-TomlConfig -Path $configPath
    
    # List scenarios if requested
    if ($List) {
        Show-Scenarios -Config $configData
        exit 0
    }
    
    # Run scenario
    $success = Invoke-Scenario -ScenarioName $Scenario -Config $configData
    
    # Show summary
    Show-Summary
    
    # Exit with appropriate code
    exit $(if ($success) { 0 } else { 1 })
    
} catch {
    Write-Host "Test execution failed: $_" -ForegroundColor Red
    Write-Host $_.ScriptStackTrace -ForegroundColor Red
    Stop-AllProcesses
    exit 1
} finally {
    Stop-AllProcesses
}