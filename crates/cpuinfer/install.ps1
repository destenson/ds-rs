# PowerShell script to install cpuinfer GStreamer plugin on Windows
param(
    [string]$BuildType = "release",
    [switch]$DryRun = $false,
    [switch]$Verbose = $false
)

# Enable strict error handling
$ErrorActionPreference = "Stop"

function Write-Info {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor Green
}

function Write-Error-Message {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}

function Write-Warning {
    param([string]$Message)
    Write-Host "[WARN] $Message" -ForegroundColor Yellow
}

# Find GStreamer installation
function Find-GStreamerPath {
    Write-Info "Searching for GStreamer installation..."
    
    # Check common installation paths
    $commonPaths = @(
        "C:\gstreamer\1.0\msvc_x86_64",
        "C:\gstreamer\1.0\mingw_x86_64",
        "C:\gstreamer\1.0\x86_64",
        "C:\Program Files\GStreamer\1.0\msvc_x86_64",
        "C:\Program Files\GStreamer\1.0\mingw_x86_64",
        "D:\gstreamer\1.0\msvc_x86_64",
        "D:\gstreamer\1.0\mingw_x86_64"
    )
    
    foreach ($path in $commonPaths) {
        if (Test-Path "$path\bin\gst-inspect-1.0.exe") {
            Write-Info "Found GStreamer at: $path"
            return $path
        }
    }
    
    # Check GST_PLUGIN_PATH environment variable
    if ($env:GST_PLUGIN_PATH) {
        Write-Info "GST_PLUGIN_PATH is set: $env:GST_PLUGIN_PATH"
        # Extract base path from plugin path
        $basePath = Split-Path -Parent $env:GST_PLUGIN_PATH
        if (Test-Path "$basePath\bin\gst-inspect-1.0.exe") {
            return $basePath
        }
    }
    
    # Check if gst-inspect-1.0 is in PATH
    $gstInspect = Get-Command gst-inspect-1.0 -ErrorAction SilentlyContinue
    if ($gstInspect) {
        $gstPath = Split-Path -Parent (Split-Path -Parent $gstInspect.Path)
        Write-Info "Found GStreamer in PATH at: $gstPath"
        return $gstPath
    }
    
    return $null
}

# Main installation logic
Write-Info "cpuinfer GStreamer Plugin Installer for Windows"
Write-Info "================================================"

# Find source plugin file
$scriptPath = Split-Path -Parent $MyInvocation.MyCommand.Path
$projectRoot = Split-Path -Parent (Split-Path -Parent $scriptPath)
$pluginFile = "$projectRoot\target\$BuildType\gstcpuinfer.dll"

if (-not (Test-Path $pluginFile)) {
    Write-Error-Message "Plugin not found at: $pluginFile"
    Write-Info "Please build the plugin first with: cargo build --release -p cpuinfer"
    exit 1
}

Write-Info "Found plugin: $pluginFile"

# Find GStreamer installation
$gstPath = Find-GStreamerPath
if (-not $gstPath) {
    Write-Error-Message "GStreamer installation not found!"
    Write-Info "Please install GStreamer or set GST_PLUGIN_PATH environment variable"
    exit 1
}

# Determine plugin directory
$pluginDir = "$gstPath\lib\gstreamer-1.0"
if (-not (Test-Path $pluginDir)) {
    Write-Warning "Plugin directory not found at: $pluginDir"
    Write-Info "Creating plugin directory..."
    if (-not $DryRun) {
        New-Item -ItemType Directory -Path $pluginDir -Force | Out-Null
    }
}

# Check for ONNX Runtime DLLs
$onnxDlls = @("onnxruntime.dll", "onnxruntime_providers_shared.dll")
$dllsFound = $true

foreach ($dll in $onnxDlls) {
    $dllPath = "$projectRoot\target\$BuildType\$dll"
    if (-not (Test-Path $dllPath)) {
        Write-Warning "ONNX Runtime DLL not found: $dll"
        $dllsFound = $false
    }
}

if (-not $dllsFound) {
    Write-Warning "Some ONNX Runtime DLLs are missing. The plugin may not work properly."
    Write-Info "The DLLs should be downloaded automatically when building with the 'ort' feature."
}

# Copy plugin and dependencies
Write-Info "Installing plugin to: $pluginDir"

if ($DryRun) {
    Write-Info "[DRY RUN] Would copy $pluginFile to $pluginDir\gstcpuinfer.dll"
    foreach ($dll in $onnxDlls) {
        $dllPath = "$projectRoot\target\$BuildType\$dll"
        if (Test-Path $dllPath) {
            Write-Info "[DRY RUN] Would copy $dll to $pluginDir\$dll"
        }
    }
} else {
    try {
        # Copy plugin
        Copy-Item $pluginFile -Destination "$pluginDir\gstcpuinfer.dll" -Force
        Write-Info "Copied plugin successfully"
        
        # Copy ONNX Runtime DLLs if present
        foreach ($dll in $onnxDlls) {
            $dllPath = "$projectRoot\target\$BuildType\$dll"
            if (Test-Path $dllPath) {
                Copy-Item $dllPath -Destination "$pluginDir\$dll" -Force
                Write-Info "Copied $dll successfully"
            }
        }
    } catch {
        Write-Error-Message "Failed to copy files: $_"
        exit 1
    }
}

# Clear plugin cache
$cacheDir = "$env:APPDATA\gstreamer-1.0"
if (Test-Path $cacheDir) {
    Write-Info "Clearing plugin cache at: $cacheDir"
    if (-not $DryRun) {
        Remove-Item "$cacheDir\*" -Recurse -Force -ErrorAction SilentlyContinue
    }
}

# Verify installation
Write-Info "Verifying installation..."
if (-not $DryRun) {
    $env:GST_PLUGIN_PATH = $pluginDir
    $output = & "$gstPath\bin\gst-inspect-1.0.exe" cpuinfer 2>&1
    
    if ($LASTEXITCODE -eq 0) {
        Write-Info "Plugin installed successfully!"
        Write-Info "You can now use the plugin with: gst-launch-1.0 ... ! cpudetector ! ..."
        
        if ($Verbose) {
            Write-Info "`nPlugin details:"
            Write-Host $output
        }
    } else {
        Write-Error-Message "Plugin verification failed!"
        Write-Info "Output: $output"
        Write-Info "Try running with GST_DEBUG=3 for more information"
        exit 1
    }
}

Write-Info "`nInstallation complete!"
Write-Info "To use the plugin in your pipelines, make sure GST_PLUGIN_PATH includes: $pluginDir"