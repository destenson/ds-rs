# Source-Videos Control API

REST API for programmatic control of video source management and RTSP streaming.

## Quick Start

Start the server with API enabled:

```bash
cargo run -- serve --api --api-port 3000
```

## Authentication

Set environment variables for authentication (optional):

```bash
export API_AUTH_ENABLED=true
export API_AUTH_TOKEN=your-secret-token
```

Then include in requests:
```bash
curl -H "Authorization: Bearer your-secret-token" http://localhost:3000/api/v1/sources
```

## Core Endpoints

### Health & Monitoring

- `GET /api/v1/health` - Basic health check
- `GET /api/v1/health/ready` - Readiness probe with component status
- `GET /api/v1/metrics` - Server metrics

### Source Management

- `GET /api/v1/sources` - List all sources
- `POST /api/v1/sources` - Add new source
- `GET /api/v1/sources/{id}` - Get source details
- `DELETE /api/v1/sources/{id}` - Remove source
- `POST /api/v1/sources/batch` - Batch operations

### Server Control

- `POST /api/v1/server/start` - Start RTSP server
- `POST /api/v1/server/stop` - Stop server
- `GET /api/v1/server/status` - Server status
- `GET /api/v1/server/urls` - List RTSP URLs

### Network Simulation

- `GET /api/v1/network/profiles` - List available profiles
- `POST /api/v1/network/apply` - Apply network profile
- `GET /api/v1/network/status` - Current conditions
- `POST /api/v1/network/reset` - Reset to perfect

### Operations

- `POST /api/v1/generate` - Generate test video
- `POST /api/v1/scan` - Scan directory for videos
- `GET /api/v1/patterns` - List test patterns
- `POST /api/v1/watch/start` - Start file watching

## Examples

### Add Test Pattern Source

```bash
curl -X POST http://localhost:3000/api/v1/sources \
  -H "Content-Type: application/json" \
  -d '{
    "name": "test_camera",
    "type": "test_pattern",
    "pattern": "smpte"
  }'
```

### Start Server with Sources

```bash
curl -X POST http://localhost:3000/api/v1/server/start \
  -H "Content-Type: application/json" \
  -d '{
    "port": 8554,
    "sources": [
      {"name": "cam1", "type": "test_pattern", "pattern": "ball"},
      {"name": "cam2", "type": "test_pattern", "pattern": "snow"}
    ]
  }'
```

### Apply Network Conditions

```bash
curl -X POST http://localhost:3000/api/v1/network/apply \
  -H "Content-Type: application/json" \
  -d '{"profile": "4g"}'
```

### Batch Operations

```bash
curl -X POST http://localhost:3000/api/v1/sources/batch \
  -H "Content-Type: application/json" \
  -d '{
    "operations": [
      {"operation": "add", "source": {"name": "src1", "type": "test_pattern", "pattern": "smpte"}},
      {"operation": "add", "source": {"name": "src2", "type": "test_pattern", "pattern": "ball"}},
      {"operation": "remove", "source": {"name": "old_source"}}
    ]
  }'
```

## Automation Examples

See the `examples/` directory for complete automation scripts:

- `api_automation.sh` - Basic API usage with curl
- `api_automation.py` - Python client implementation
- `live_display_automation.sh` - Bash script with GStreamer display
- `live_display_automation.py` - Python with GStreamer integration
- `live_display_automation.ps1` - PowerShell for Windows

### Live Display Example

Start sources and display them with GStreamer:

```bash
# Start API server
cargo run -- serve --api --api-port 3000

# In another terminal, run automation
./examples/live_display_automation.sh
```

The live display scripts will:
1. Start RTSP server with test patterns
2. Launch GStreamer pipelines to display streams
3. Provide interactive control (network simulation, metrics, etc.)
4. Support multiple display modes (individual, mosaic, tiled)

## Error Handling

All errors return JSON with structure:

```json
{
  "error": {
    "type": "error_type",
    "message": "Human readable message",
    "status": 400
  }
}
```

Status codes:
- 200: Success
- 400: Bad Request
- 401: Unauthorized
- 404: Not Found
- 409: Conflict
- 422: Validation Error
- 500: Internal Error
- 503: Service Unavailable