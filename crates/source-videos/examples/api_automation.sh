#!/bin/bash

# Source-Videos Control API Automation Examples
# This script demonstrates common automation patterns using the REST API

API_BASE="http://localhost:3000/api/v1"

# Optional: Set authentication token if enabled
# export API_AUTH_TOKEN="your-token-here"
# AUTH_HEADER="Authorization: Bearer $API_AUTH_TOKEN"

echo "Source-Videos API Automation Examples"
echo "====================================="

# 1. Health Check
echo -e "\n1. Checking API health..."
curl -s "$API_BASE/health" | jq '.'

# 2. Start RTSP Server
echo -e "\n2. Starting RTSP server..."
curl -s -X POST "$API_BASE/server/start" \
  -H "Content-Type: application/json" \
  -d '{
    "port": 8554,
    "address": "0.0.0.0",
    "sources": [
      {"name": "test1", "type": "test_pattern", "pattern": "smpte"},
      {"name": "test2", "type": "test_pattern", "pattern": "ball"}
    ]
  }' | jq '.'

# 3. List Sources
echo -e "\n3. Listing all sources..."
curl -s "$API_BASE/sources" | jq '.'

# 4. Add a New Source
echo -e "\n4. Adding a new source..."
curl -s -X POST "$API_BASE/sources" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "snow_pattern",
    "type": "test_pattern",
    "pattern": "snow",
    "resolution": {"width": 1920, "height": 1080},
    "framerate": {"numerator": 30, "denominator": 1}
  }' | jq '.'

# 5. Apply Network Simulation
echo -e "\n5. Applying network profile..."
curl -s -X POST "$API_BASE/network/apply" \
  -H "Content-Type: application/json" \
  -d '{"profile": "4g"}' | jq '.'

# 6. Get Network Status
echo -e "\n6. Getting network status..."
curl -s "$API_BASE/network/status" | jq '.'

# 7. Scan Directory for Videos
echo -e "\n7. Scanning directory for video files..."
curl -s -X POST "$API_BASE/scan" \
  -H "Content-Type: application/json" \
  -d '{
    "path": "/path/to/videos",
    "recursive": true,
    "add_to_server": true
  }' | jq '.'

# 8. Start File Watching
echo -e "\n8. Starting file watching..."
curl -s -X POST "$API_BASE/watch/start" \
  -H "Content-Type: application/json" \
  -d '{
    "directory": "/path/to/watch",
    "recursive": true,
    "auto_reload": true
  }' | jq '.'

# 9. Batch Operations
echo -e "\n9. Performing batch operations..."
curl -s -X POST "$API_BASE/sources/batch" \
  -H "Content-Type: application/json" \
  -d '{
    "operations": [
      {"operation": "add", "source": {"name": "batch1", "type": "test_pattern", "pattern": "checkers-1"}},
      {"operation": "add", "source": {"name": "batch2", "type": "test_pattern", "pattern": "checkers-2"}},
      {"operation": "remove", "source": {"name": "test1"}}
    ],
    "atomic": false
  }' | jq '.'

# 10. Get Server URLs
echo -e "\n10. Getting RTSP URLs..."
curl -s "$API_BASE/server/urls" | jq '.'

# 11. Check Readiness
echo -e "\n11. Checking component readiness..."
curl -s "$API_BASE/health/ready" | jq '.'

# 12. Get Metrics
echo -e "\n12. Getting metrics..."
curl -s "$API_BASE/metrics" | jq '.'

echo -e "\n====================================="
echo "Automation examples completed!"