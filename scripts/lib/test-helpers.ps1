<#
.SYNOPSIS
    Helper functions for test orchestration
.DESCRIPTION
    Common PowerShell functions for test orchestration, process management, and reporting
#>

# Process management functions
function Start-TestProcess {
    <#
    .SYNOPSIS
        Starts a background process for testing
    .PARAMETER Name
        Friendly name for the process
    .PARAMETER Command
        Command to execute
    .PARAMETER Arguments
        Command arguments
    .PARAMETER WorkingDirectory
        Working directory for the process
    .PARAMETER WaitForPattern
        Optional pattern to wait for in output
    .PARAMETER Timeout
        Timeout in seconds
    #>
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [string]$Name,
        
        [Parameter(Mandatory)]
        [string]$Command,
        
        [string]$Arguments = "",
        
        [string]$WorkingDirectory = (Get-Location),
        
        [string]$WaitForPattern = "",
        
        [int]$Timeout = 30
    )
    
    $processInfo = New-Object System.Diagnostics.ProcessStartInfo
    $processInfo.FileName = $Command
    $processInfo.Arguments = $Arguments
    $processInfo.WorkingDirectory = $WorkingDirectory
    $processInfo.UseShellExecute = $false
    $processInfo.RedirectStandardOutput = $true
    $processInfo.RedirectStandardError = $true
    $processInfo.CreateNoWindow = $true
    
    $process = New-Object System.Diagnostics.Process
    $process.StartInfo = $processInfo
    
    # Set up output handlers
    $outputBuilder = New-Object System.Text.StringBuilder
    $errorBuilder = New-Object System.Text.StringBuilder
    
    $outputHandler = {
        if ($EventArgs.Data) {
            $outputBuilder.AppendLine($EventArgs.Data) | Out-Null
            if ($VerbosePreference -eq 'Continue') {
                Write-Host "[${Name}] $($EventArgs.Data)" -ForegroundColor Gray
            }
        }
    }
    
    $errorHandler = {
        if ($EventArgs.Data) {
            $errorBuilder.AppendLine($EventArgs.Data) | Out-Null
            if ($VerbosePreference -eq 'Continue') {
                Write-Host "[${Name}] ERROR: $($EventArgs.Data)" -ForegroundColor Red
            }
        }
    }
    
    Register-ObjectEvent -InputObject $process -EventName OutputDataReceived -Action $outputHandler | Out-Null
    Register-ObjectEvent -InputObject $process -EventName ErrorDataReceived -Action $errorHandler | Out-Null
    
    try {
        $process.Start() | Out-Null
        $process.BeginOutputReadLine()
        $process.BeginErrorReadLine()
        
        # Wait for pattern if specified
        if ($WaitForPattern) {
            $startTime = Get-Date
            $found = $false
            
            while (-not $found -and ((Get-Date) - $startTime).TotalSeconds -lt $Timeout) {
                if ($outputBuilder.ToString() -match $WaitForPattern) {
                    $found = $true
                    break
                }
                Start-Sleep -Milliseconds 100
            }
            
            if (-not $found) {
                throw "Timeout waiting for pattern '$WaitForPattern' in $Name output"
            }
        }
        
        return @{
            Process = $process
            Name = $Name
            StartTime = Get-Date
            Output = $outputBuilder
            Error = $errorBuilder
        }
    }
    catch {
        if ($process -and -not $process.HasExited) {
            $process.Kill()
        }
        throw
    }
}

function Stop-TestProcess {
    <#
    .SYNOPSIS
        Stops a test process gracefully
    #>
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        $ProcessInfo,
        
        [int]$GracePeriod = 5
    )
    
    if (-not $ProcessInfo.Process.HasExited) {
        try {
            # Try graceful shutdown first
            $ProcessInfo.Process.CloseMainWindow() | Out-Null
            $ProcessInfo.Process.WaitForExit($GracePeriod * 1000) | Out-Null
        }
        catch {
            # Force kill if graceful shutdown fails
            $ProcessInfo.Process.Kill()
        }
    }
    
    return @{
        ExitCode = $ProcessInfo.Process.ExitCode
        Duration = (Get-Date) - $ProcessInfo.StartTime
        Output = $ProcessInfo.Output.ToString()
        Error = $ProcessInfo.Error.ToString()
    }
}

# RTSP server management
function Start-RtspTestServer {
    <#
    .SYNOPSIS
        Starts an RTSP server for testing
    #>
    [CmdletBinding()]
    param(
        [int]$Port = 8554,
        [string]$Pattern = "smpte",
        [string[]]$MountPoints = @("/test"),
        [string]$ProjectRoot = (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))
    )
    
    $sourceVideosPath = Join-Path $ProjectRoot "crates\source-videos"
    
    # Build source-videos if needed
    Write-Host "Building source-videos..." -ForegroundColor Cyan
    $buildResult = & cargo build --release --manifest-path (Join-Path $sourceVideosPath "Cargo.toml") 2>&1
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to build source-videos: $buildResult"
    }
    
    # Prepare RTSP server arguments
    $rtspArgs = @(
        "run", "--release", "--",
        "serve",
        "--port", $Port,
        "--pattern", $Pattern
    )
    
    foreach ($mount in $MountPoints) {
        $rtspArgs += "--mount-point"
        $rtspArgs += $mount
    }
    
    # Start RTSP server
    $rtspServer = Start-TestProcess `
        -Name "RTSP-Server" `
        -Command "cargo" `
        -Arguments ($rtspArgs -join " ") `
        -WorkingDirectory $sourceVideosPath `
        -WaitForPattern "RTSP server started" `
        -Timeout 30
    
    return $rtspServer
}

# Test result parsing
function Get-CargoTestResults {
    <#
    .SYNOPSIS
        Parses cargo test output for results
    #>
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [string]$Output
    )
    
    $results = @{
        Passed = 0
        Failed = 0
        Ignored = 0
        Total = 0
        Duration = ""
        FailedTests = @()
    }
    
    # Parse test result line
    if ($Output -match 'test result: .*?(\d+) passed.*?(\d+) failed.*?(\d+) ignored.*?finished in ([\d.]+)s') {
        $results.Passed = [int]$Matches[1]
        $results.Failed = [int]$Matches[2]
        $results.Ignored = [int]$Matches[3]
        $results.Duration = $Matches[4] + "s"
        $results.Total = $results.Passed + $results.Failed + $results.Ignored
    }
    
    # Extract failed test names
    $failedPattern = 'test (.+?) \.\.\. FAILED'
    $matches = [regex]::Matches($Output, $failedPattern)
    foreach ($match in $matches) {
        $results.FailedTests += $match.Groups[1].Value
    }
    
    return $results
}

# Report generation
function New-TestHtmlReport {
    <#
    .SYNOPSIS
        Generates an HTML test report
    #>
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [hashtable]$TestResults,
        
        [string]$OutputPath = "test-report.html",
        
        [string]$Title = "Test Execution Report"
    )
    
    $html = @"
<!DOCTYPE html>
<html>
<head>
    <title>$Title</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        h1 { color: #333; }
        .summary { background: #f0f0f0; padding: 15px; border-radius: 5px; margin: 20px 0; }
        .passed { color: green; font-weight: bold; }
        .failed { color: red; font-weight: bold; }
        .ignored { color: orange; }
        table { width: 100%; border-collapse: collapse; margin: 20px 0; }
        th { background: #333; color: white; padding: 10px; text-align: left; }
        td { padding: 8px; border-bottom: 1px solid #ddd; }
        tr:hover { background: #f5f5f5; }
        .test-failed { background: #ffeeee; }
        .test-passed { background: #eeffee; }
        .timestamp { color: #666; font-size: 0.9em; }
    </style>
</head>
<body>
    <h1>$Title</h1>
    <div class="timestamp">Generated: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')</div>
"@
    
    # Calculate summary
    $totalSuites = $TestResults.Count
    $passedSuites = ($TestResults.Values | Where-Object { $_.Success }).Count
    $failedSuites = $totalSuites - $passedSuites
    $successRate = if ($totalSuites -gt 0) { [Math]::Round($passedSuites / $totalSuites * 100, 1) } else { 0 }
    
    $html += @"
    <div class="summary">
        <h2>Summary</h2>
        <p>Total Test Suites: <strong>$totalSuites</strong></p>
        <p>Passed: <span class="passed">$passedSuites</span></p>
        <p>Failed: <span class="failed">$failedSuites</span></p>
        <p>Success Rate: <strong>$successRate%</strong></p>
    </div>
    
    <h2>Test Results</h2>
    <table>
        <tr>
            <th>Test Suite</th>
            <th>Status</th>
            <th>Tests</th>
            <th>Duration</th>
            <th>Details</th>
        </tr>
"@
    
    foreach ($suite in $TestResults.Keys) {
        $result = $TestResults[$suite]
        $statusClass = if ($result.Success) { "test-passed" } else { "test-failed" }
        $status = if ($result.Success) { "PASSED" } else { "FAILED" }
        
        $testInfo = ""
        if ($result.TestResults) {
            $tr = $result.TestResults
            $testInfo = "$($tr.Passed) passed, $($tr.Failed) failed, $($tr.Ignored) ignored"
        }
        
        $duration = if ($result.TestResults -and $result.TestResults.Duration) { 
            $result.TestResults.Duration 
        } else { 
            "N/A" 
        }
        
        $details = ""
        if ($result.TestResults -and $result.TestResults.FailedTests.Count -gt 0) {
            $details = "Failed: " + ($result.TestResults.FailedTests -join ", ")
        }
        
        $html += @"
        <tr class="$statusClass">
            <td>$suite</td>
            <td><strong>$status</strong></td>
            <td>$testInfo</td>
            <td>$duration</td>
            <td>$details</td>
        </tr>
"@
    }
    
    $html += @"
    </table>
</body>
</html>
"@
    
    $html | Set-Content -Path $OutputPath
    return $OutputPath
}

# Environment validation
function Test-ProjectEnvironment {
    <#
    .SYNOPSIS
        Validates the project environment for testing
    #>
    [CmdletBinding()]
    param(
        [string[]]$RequiredCommands = @("cargo", "git"),
        [string[]]$OptionalCommands = @("gst-inspect-1.0", "python"),
        [hashtable]$MinVersions = @{
            "cargo" = "1.70.0"
            "git" = "2.0.0"
        }
    )
    
    $results = @{
        Valid = $true
        Required = @{}
        Optional = @{}
        Warnings = @()
    }
    
    # Check required commands
    foreach ($cmd in $RequiredCommands) {
        try {
            $version = & $cmd --version 2>&1
            if ($LASTEXITCODE -eq 0) {
                $results.Required[$cmd] = @{
                    Found = $true
                    Version = $version
                }
                
                # Check minimum version if specified
                if ($MinVersions.ContainsKey($cmd)) {
                    # Simple version check (can be improved)
                    if ($version -notmatch $MinVersions[$cmd]) {
                        $results.Warnings += "Warning: $cmd version may be too old. Required: $($MinVersions[$cmd])"
                    }
                }
            }
            else {
                $results.Required[$cmd] = @{ Found = $false }
                $results.Valid = $false
            }
        }
        catch {
            $results.Required[$cmd] = @{ Found = $false }
            $results.Valid = $false
        }
    }
    
    # Check optional commands
    foreach ($cmd in $OptionalCommands) {
        try {
            $version = & $cmd --version 2>&1
            if ($LASTEXITCODE -eq 0) {
                $results.Optional[$cmd] = @{
                    Found = $true
                    Version = $version
                }
            }
            else {
                $results.Optional[$cmd] = @{ Found = $false }
                $results.Warnings += "Optional dependency not found: $cmd"
            }
        }
        catch {
            $results.Optional[$cmd] = @{ Found = $false }
            $results.Warnings += "Optional dependency not found: $cmd"
        }
    }
    
    # Check GStreamer plugins
    if ($results.Optional["gst-inspect-1.0"].Found) {
        $requiredPlugins = @("videotestsrc", "x264enc", "rtph264pay")
        foreach ($plugin in $requiredPlugins) {
            $pluginCheck = & gst-inspect-1.0 $plugin 2>&1
            if ($LASTEXITCODE -ne 0) {
                $results.Warnings += "GStreamer plugin not found: $plugin"
            }
        }
    }
    
    return $results
}

# Parallel test execution
function Invoke-ParallelTests {
    <#
    .SYNOPSIS
        Runs tests in parallel across multiple crates
    #>
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [string[]]$Crates,
        
        [string[]]$TestArgs = @(),
        
        [int]$MaxParallel = 4,
        
        [string]$ProjectRoot = (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))
    )
    
    $jobs = @()
    $results = @{}
    
    foreach ($crate in $Crates) {
        # Wait if we're at max parallel jobs
        while ((Get-Job -State Running).Count -ge $MaxParallel) {
            Start-Sleep -Milliseconds 500
        }
        
        $cratePath = Join-Path $ProjectRoot "crates\$crate"
        
        $job = Start-Job -ScriptBlock {
            param($CratePath, $TestArgs)
            
            Set-Location $CratePath
            $output = & cargo test @TestArgs 2>&1
            $exitCode = $LASTEXITCODE
            
            return @{
                Output = $output -join "`n"
                ExitCode = $exitCode
                Success = $exitCode -eq 0
            }
        } -ArgumentList $cratePath, $TestArgs
        
        $jobs += @{
            Job = $job
            Crate = $crate
        }
        
        Write-Host "Started tests for $crate (Job ID: $($job.Id))" -ForegroundColor Cyan
    }
    
    # Wait for all jobs to complete
    Write-Host "Waiting for all test jobs to complete..." -ForegroundColor Yellow
    
    foreach ($jobInfo in $jobs) {
        $result = Wait-Job -Job $jobInfo.Job | Receive-Job
        Remove-Job -Job $jobInfo.Job
        
        $results[$jobInfo.Crate] = $result
        
        if ($result.Success) {
            Write-Host "✓ $($jobInfo.Crate) tests passed" -ForegroundColor Green
        }
        else {
            Write-Host "✗ $($jobInfo.Crate) tests failed" -ForegroundColor Red
        }
    }
    
    return $results
}

# Export functions
Export-ModuleMember -Function @(
    'Start-TestProcess',
    'Stop-TestProcess',
    'Start-RtspTestServer',
    'Get-CargoTestResults',
    'New-TestHtmlReport',
    'Test-ProjectEnvironment',
    'Invoke-ParallelTests'
)