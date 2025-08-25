# Live Display Automation for Windows PowerShell
# Demonstrates API control and GStreamer display

$API_BASE = "http://localhost:3000/api/v1"
$RTSP_PORT = 8554
$GST_PROCESSES = @()

# Function to launch GStreamer display
function Launch-Display {
    param(
        [string]$RtspUrl,
        [string]$Title
    )
    
    Write-Host "Launching display for: $Title"
    
    $pipeline = "gst-launch-1.0.exe " +
                "rtspsrc location=$RtspUrl latency=100 ! " +
                "decodebin ! " +
                "videoconvert ! " +
                "videoscale ! " +
                "video/x-raw,width=640,height=480 ! " +
                "fpsdisplaysink video-sink=autovideosink text-overlay=true sync=false"
    
    $process = Start-Process -FilePath "cmd.exe" `
                            -ArgumentList "/c", $pipeline `
                            -PassThru `
                            -WindowStyle Normal
    
    $script:GST_PROCESSES += $process
    return $process
}

# Function to check API health
function Test-ApiHealth {
    try {
        $response = Invoke-RestMethod -Uri "$API_BASE/health" -Method Get
        return $true
    }
    catch {
        return $false
    }
}

# Function to start RTSP server
function Start-RtspServer {
    $sources = @(
        @{name="smpte"; type="test_pattern"; pattern="smpte"},
        @{name="ball"; type="test_pattern"; pattern="ball"},
        @{name="snow"; type="test_pattern"; pattern="snow"},
        @{name="bars"; type="test_pattern"; pattern="bar"}
    )
    
    $body = @{
        port = $RTSP_PORT
        address = "0.0.0.0"
        sources = $sources
    } | ConvertTo-Json -Depth 3
    
    $response = Invoke-RestMethod -Uri "$API_BASE/server/start" `
                                  -Method Post `
                                  -ContentType "application/json" `
                                  -Body $body
    
    return $response
}

# Function to get RTSP URLs
function Get-RtspUrls {
    $response = Invoke-RestMethod -Uri "$API_BASE/server/urls" -Method Get
    return $response
}

# Function to apply network profile
function Set-NetworkProfile {
    param([string]$Profile)
    
    $body = @{profile = $Profile} | ConvertTo-Json
    
    $response = Invoke-RestMethod -Uri "$API_BASE/network/apply" `
                                  -Method Post `
                                  -ContentType "application/json" `
                                  -Body $body
    
    return $response
}

# Function to get metrics
function Get-Metrics {
    $response = Invoke-RestMethod -Uri "$API_BASE/metrics" -Method Get
    return $response
}

# Cleanup function
function Stop-AllDisplays {
    Write-Host "`nStopping all displays..."
    
    foreach ($process in $GST_PROCESSES) {
        if (-not $process.HasExited) {
            Stop-Process -Id $process.Id -Force
        }
    }
    
    # Stop RTSP server
    try {
        Invoke-RestMethod -Uri "$API_BASE/server/stop" -Method Post | Out-Null
    }
    catch {
        # Server might already be stopped
    }
    
    Write-Host "Cleanup complete"
}

# Main execution
try {
    Clear-Host
    Write-Host "Live Display Automation - PowerShell" -ForegroundColor Cyan
    Write-Host "=====================================" -ForegroundColor Cyan
    
    # Step 1: Check API
    Write-Host "`n1. Checking API health..." -ForegroundColor Yellow
    if (-not (Test-ApiHealth)) {
        Write-Host "Error: API not responding. Start with: cargo run -- serve --api" -ForegroundColor Red
        exit 1
    }
    Write-Host "   API is healthy" -ForegroundColor Green
    
    # Step 2: Start RTSP server
    Write-Host "`n2. Starting RTSP server..." -ForegroundColor Yellow
    $serverStatus = Start-RtspServer
    Write-Host "   Server running: $($serverStatus.running)" -ForegroundColor Green
    Write-Host "   Port: $($serverStatus.port)" -ForegroundColor Green
    
    # Step 3: Get URLs
    Write-Host "`n3. Getting RTSP URLs..." -ForegroundColor Yellow
    $urls = Get-RtspUrls
    foreach ($url in $urls) {
        Write-Host "   - $url" -ForegroundColor Cyan
    }
    
    # Step 4: Display mode selection
    Write-Host "`n4. Select display mode:" -ForegroundColor Yellow
    Write-Host "   1) Individual windows"
    Write-Host "   2) Tiled display"
    Write-Host "   3) Sequential display"
    $choice = Read-Host "   Choice [1-3]"
    
    # Step 5: Launch displays
    Write-Host "`n5. Launching displays..." -ForegroundColor Yellow
    
    switch ($choice) {
        "1" {
            # Individual windows
            $index = 0
            foreach ($url in $urls[0..3]) {
                Launch-Display -RtspUrl $url -Title "Stream $index"
                Start-Sleep -Milliseconds 500
                $index++
            }
        }
        "2" {
            # Tiled display using videomixer
            Write-Host "Launching tiled display..."
            $pipeline = "gst-launch-1.0.exe compositor name=comp " +
                       "! videoconvert ! videoscale " +
                       "! video/x-raw,width=1280,height=720 " +
                       "! fpsdisplaysink video-sink=autovideosink sync=false"
            
            $x = 0
            $y = 0
            for ($i = 0; $i -lt [Math]::Min(4, $urls.Count); $i++) {
                $x = ($i % 2) * 640
                $y = [Math]::Floor($i / 2) * 360
                
                $pipeline += " rtspsrc location=$($urls[$i]) latency=100 ! decodebin " +
                            "! videoconvert ! videoscale " +
                            "! video/x-raw,width=640,height=360 " +
                            "! comp.sink_$i " +
                            "comp.sink_$i::xpos=$x comp.sink_$i::ypos=$y"
            }
            
            $process = Start-Process -FilePath "cmd.exe" `
                                    -ArgumentList "/c", $pipeline `
                                    -PassThru `
                                    -WindowStyle Normal
            $GST_PROCESSES += $process
        }
        "3" {
            # Sequential display
            foreach ($url in $urls) {
                Write-Host "Displaying: $url (press any key for next)"
                $process = Launch-Display -RtspUrl $url -Title "Sequential"
                Read-Host
                Stop-Process -Id $process.Id -Force
            }
        }
    }
    
    # Step 6: Interactive control
    Write-Host "`n6. Interactive control:" -ForegroundColor Yellow
    Write-Host "   n - Cycle network profiles"
    Write-Host "   m - Show metrics"
    Write-Host "   a - Add test source"
    Write-Host "   q - Quit"
    
    $profiles = @("perfect", "4g", "3g", "poor")
    $profileIndex = 0
    
    while ($true) {
        $key = Read-Host "`nCommand"
        
        switch ($key) {
            "n" {
                $profileIndex = ($profileIndex + 1) % $profiles.Count
                $profile = $profiles[$profileIndex]
                Write-Host "Applying network profile: $profile" -ForegroundColor Cyan
                $result = Set-NetworkProfile -Profile $profile
                Write-Host "  $($result.message)" -ForegroundColor Green
            }
            "m" {
                Write-Host "Metrics:" -ForegroundColor Cyan
                $metrics = Get-Metrics
                Write-Host "  Sources: $($metrics.source_count)"
                Write-Host "  Connections: $($metrics.active_connections)"
                Write-Host "  Requests: $($metrics.total_requests)"
            }
            "a" {
                $name = "test_$(Get-Date -Format 'HHmmss')"
                $body = @{
                    name = $name
                    type = "test_pattern"
                    pattern = "circular"
                } | ConvertTo-Json
                
                $response = Invoke-RestMethod -Uri "$API_BASE/sources" `
                                             -Method Post `
                                             -ContentType "application/json" `
                                             -Body $body
                
                Write-Host "Added source: $($response.name)" -ForegroundColor Green
                Write-Host "URL: rtsp://localhost:$RTSP_PORT/$name" -ForegroundColor Cyan
            }
            "q" {
                Write-Host "Quitting..." -ForegroundColor Yellow
                break
            }
            default {
                Write-Host "Unknown command" -ForegroundColor Red
            }
        }
    }
}
finally {
    # Ensure cleanup happens
    Stop-AllDisplays
}

Write-Host "`nAutomation complete!" -ForegroundColor Green