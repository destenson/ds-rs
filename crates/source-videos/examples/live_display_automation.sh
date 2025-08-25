#!/bin/bash

# Live Display Automation Script
# This script demonstrates starting sources via API and displaying them with GStreamer
# Requires: gstreamer, jq, curl

API_BASE="http://localhost:3000/api/v1"
RTSP_PORT=8554

echo "Live Display Automation - Source Videos + GStreamer"
echo "===================================================="

# Function to launch GStreamer display pipeline
launch_display() {
    local rtsp_url=$1
    local window_title=$2
    local x_pos=$3
    local y_pos=$4
    
    echo "Launching display for: $rtsp_url"
    
    # Launch GStreamer pipeline in background
    gst-launch-1.0 \
        rtspsrc location="$rtsp_url" latency=100 ! \
        decodebin ! \
        videoconvert ! \
        videoscale ! \
        video/x-raw,width=640,height=480 ! \
        fpsdisplaysink video-sink="autovideosink" text-overlay=true sync=false \
        name="$window_title" &
    
    # Store PID for cleanup
    echo $! >> /tmp/gst_pids.txt
}

# Function to launch mosaic display (multiple streams in one window)
launch_mosaic() {
    local urls=("$@")
    
    echo "Launching mosaic display with ${#urls[@]} streams..."
    
    # Build compositor pipeline
    local pipeline="compositor name=comp"
    
    # Add sink
    pipeline+=" ! videoconvert ! videoscale ! video/x-raw,width=1280,height=720"
    pipeline+=" ! fpsdisplaysink video-sink=autovideosink sync=false"
    
    # Add sources
    local x=0
    local y=0
    local idx=0
    
    for url in "${urls[@]}"; do
        # Calculate position in 2x2 grid
        x=$((($idx % 2) * 640))
        y=$((($idx / 2) * 360))
        
        pipeline+=" rtspsrc location=$url latency=100 ! decodebin"
        pipeline+=" ! videoconvert ! videoscale"
        pipeline+=" ! video/x-raw,width=640,height=360"
        pipeline+=" ! comp.sink_$idx"
        pipeline+=" comp.sink_$idx::xpos=$x comp.sink_$idx::ypos=$y"
        
        idx=$((idx + 1))
    done
    
    echo "Pipeline: $pipeline"
    gst-launch-1.0 $pipeline &
    echo $! >> /tmp/gst_pids.txt
}

# Cleanup function
cleanup() {
    echo -e "\nCleaning up..."
    
    # Kill all GStreamer processes
    if [ -f /tmp/gst_pids.txt ]; then
        while read pid; do
            kill $pid 2>/dev/null
        done < /tmp/gst_pids.txt
        rm /tmp/gst_pids.txt
    fi
    
    # Stop RTSP server
    curl -s -X POST "$API_BASE/server/stop" > /dev/null
    
    echo "Cleanup complete"
    exit 0
}

# Set trap for cleanup on exit
trap cleanup INT TERM EXIT

# Clear any previous PIDs
rm -f /tmp/gst_pids.txt

# Step 1: Check API health
echo -e "\n1. Checking API health..."
health=$(curl -s "$API_BASE/health")
if [ -z "$health" ]; then
    echo "Error: API not responding. Start with: cargo run -- serve --api"
    exit 1
fi
echo "   API is healthy"

# Step 2: Start RTSP server with test patterns
echo -e "\n2. Starting RTSP server with test sources..."
response=$(curl -s -X POST "$API_BASE/server/start" \
  -H "Content-Type: application/json" \
  -d '{
    "port": 8554,
    "sources": [
      {"name": "smpte", "type": "test_pattern", "pattern": "smpte"},
      {"name": "ball", "type": "test_pattern", "pattern": "ball"},
      {"name": "snow", "type": "test_pattern", "pattern": "snow"},
      {"name": "checkers", "type": "test_pattern", "pattern": "checkers-1"}
    ]
  }')

if [ $? -ne 0 ]; then
    echo "Error starting RTSP server"
    exit 1
fi

echo "   Server started successfully"

# Step 3: Get RTSP URLs
echo -e "\n3. Getting RTSP URLs..."
urls=$(curl -s "$API_BASE/server/urls")
echo "   Available streams:"
echo "$urls" | jq -r '.[]' | while read url; do
    echo "   - $url"
done

# Step 4: Apply network simulation (optional)
echo -e "\n4. Applying network simulation..."
curl -s -X POST "$API_BASE/network/apply" \
  -H "Content-Type: application/json" \
  -d '{"profile": "perfect"}' > /dev/null
echo "   Network profile: perfect (no simulation)"

# Step 5: Launch display mode selection
echo -e "\n5. Select display mode:"
echo "   1) Individual windows (one per stream)"
echo "   2) Mosaic view (all in one window)"
echo "   3) Tiled windows (arranged on screen)"
echo -n "   Choice [1-3]: "
read choice

case $choice in
    1)
        echo -e "\n6. Launching individual displays..."
        # Launch each stream in its own window
        launch_display "rtsp://localhost:8554/smpte" "SMPTE Pattern" 0 0
        sleep 0.5
        launch_display "rtsp://localhost:8554/ball" "Ball Pattern" 650 0
        sleep 0.5
        launch_display "rtsp://localhost:8554/snow" "Snow Pattern" 0 500
        sleep 0.5
        launch_display "rtsp://localhost:8554/checkers" "Checkers Pattern" 650 500
        ;;
    
    2)
        echo -e "\n6. Launching mosaic display..."
        # Launch all streams in one mosaic window
        urls=(
            "rtsp://localhost:8554/smpte"
            "rtsp://localhost:8554/ball"
            "rtsp://localhost:8554/snow"
            "rtsp://localhost:8554/checkers"
        )
        launch_mosaic "${urls[@]}"
        ;;
    
    3)
        echo -e "\n6. Launching tiled displays with videotestsrc comparison..."
        # Launch test pattern directly from videotestsrc for comparison
        echo "   Launching local videotestsrc..."
        gst-launch-1.0 videotestsrc pattern=smpte ! \
            video/x-raw,width=640,height=480 ! \
            fpsdisplaysink video-sink="autovideosink" text-overlay=true &
        echo $! >> /tmp/gst_pids.txt
        
        sleep 0.5
        
        # Launch RTSP stream
        echo "   Launching RTSP stream..."
        launch_display "rtsp://localhost:8554/smpte" "RTSP SMPTE" 650 0
        ;;
    
    *)
        echo "Invalid choice"
        exit 1
        ;;
esac

echo -e "\n7. Streams are now displaying"
echo "   Press Ctrl+C to stop all streams and cleanup"

# Step 7: Monitor and apply dynamic changes
echo -e "\n8. Dynamic control (press key + Enter):"
echo "   n - Apply network simulation (cycle through profiles)"
echo "   a - Add new source"
echo "   r - Remove a source"
echo "   m - Show metrics"
echo "   q - Quit"

network_profiles=("perfect" "4g" "3g" "poor")
profile_idx=0

while true; do
    read -n 1 -t 1 key
    
    case $key in
        n)
            # Cycle through network profiles
            profile_idx=$(( (profile_idx + 1) % ${#network_profiles[@]} ))
            profile=${network_profiles[$profile_idx]}
            echo -e "\n   Applying network profile: $profile"
            curl -s -X POST "$API_BASE/network/apply" \
                -H "Content-Type: application/json" \
                -d "{\"profile\": \"$profile\"}" | jq '.message'
            ;;
        
        a)
            # Add a new source
            echo -e "\n   Adding new source..."
            name="dynamic_$(date +%s)"
            curl -s -X POST "$API_BASE/sources" \
                -H "Content-Type: application/json" \
                -d "{
                    \"name\": \"$name\",
                    \"type\": \"test_pattern\",
                    \"pattern\": \"circular\"
                }" | jq '.name'
            echo "   Added source: $name"
            echo "   Connect with: gst-launch-1.0 rtspsrc location=rtsp://localhost:8554/$name ! decodebin ! autovideosink"
            ;;
        
        r)
            # Remove a source
            echo -e "\n   Current sources:"
            sources=$(curl -s "$API_BASE/sources" | jq -r '.[].name')
            echo "$sources"
            echo -n "   Enter source name to remove: "
            read source_name
            curl -s -X DELETE "$API_BASE/sources/$source_name" | jq '.message'
            ;;
        
        m)
            # Show metrics
            echo -e "\n   Metrics:"
            curl -s "$API_BASE/metrics" | jq '.'
            ;;
        
        q)
            echo -e "\n   Quitting..."
            break
            ;;
    esac
done

echo -e "\nStopping automation..."
# Cleanup will be called by trap