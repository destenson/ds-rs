# PRP: Source-Videos Control API for Automation

## Executive Summary

Implement a REST API control interface for source-videos to enable programmatic automation, monitoring, and management. This API will expose all current CLI functionality through HTTP endpoints, enabling integration with CI/CD pipelines, monitoring systems, and custom automation scripts. Unlike PRP-17 which focused on comprehensive WebSocket real-time features, this PRP prioritizes simplicity and automation use cases.

## Problem Statement

### Current State
- Only CLI interface available for control
- No programmatic access to source management
- Cannot integrate with automation tools
- Limited remote control capabilities
- No standardized API for external systems

### Desired State
- REST API exposing all CLI operations
- Batch operations for efficiency
- Health and status monitoring endpoints
- Configuration management via API
- Simple authentication for security
- OpenAPI documentation for client generation

### Business Value
Enables automation workflows, supports CI/CD integration, allows remote management, facilitates monitoring and alerting, and provides foundation for custom UIs and tools.

## Requirements

### Functional Requirements

1. **Source Management**: CRUD operations for video sources
2. **Server Control**: Start, stop, restart operations
3. **Configuration**: Get/set configuration parameters
4. **Network Simulation**: Control network conditions via API
5. **File Operations**: Trigger file generation and scanning
6. **Status Monitoring**: Health checks and metrics
7. **Batch Operations**: Process multiple sources atomically

### Non-Functional Requirements

1. **Simplicity**: Easy to use for automation scripts
2. **Performance**: Response time < 100ms for most operations
3. **Documentation**: OpenAPI spec for all endpoints
4. **Security**: Optional token-based authentication
5. **Compatibility**: JSON request/response format

### Context and Research

The project already has axum 0.8.4 in dependencies. Axum provides excellent ergonomics for REST APIs with type-safe extractors, built-in JSON support via serde, and Tower middleware integration. The existing VideoSourceManager and RtspServer can be wrapped with Arc for thread-safe access.

Current source-videos operations from CLI:
- serve: Start RTSP server with various options
- generate: Create test video files
- list: Show available sources/patterns
- interactive: REPL mode
- test: Run test server

These map naturally to REST endpoints.

### Documentation & References

```yaml
- url: https://docs.rs/axum/0.8.4/axum/
  why: Axum 0.8.4 documentation (current version in project)

- url: https://github.com/tokio-rs/axum/tree/main/examples
  why: Official Axum examples including REST APIs

- file: crates/source-videos/src/main.rs
  why: Current CLI commands to expose via API

- file: crates/source-videos/src/manager.rs
  why: VideoSourceManager for source operations

- file: crates/source-videos/src/rtsp/mod.rs
  why: RtspServer for server control

- url: https://docs.rs/utoipa/latest/utoipa/
  why: OpenAPI generation for Rust/Axum

- url: https://docs.rs/tower-http/latest/tower_http/
  why: Middleware for CORS, tracing, compression

- file: crates/source-videos/src/network/mod.rs
  why: Network simulation to expose via API
```

### List of tasks to be completed

```yaml
Task 1:
CREATE crates/source-videos/src/api/mod.rs:
  - DEFINE ControlApi struct wrapping RtspServer and managers
  - SETUP Axum Router with routes
  - CONFIGURE middleware (CORS, tracing, compression)
  - IMPLEMENT bind_and_serve method
  - ADD graceful shutdown handler
  - INTEGRATE with existing source-videos components

Task 2:
CREATE crates/source-videos/src/api/state.rs:
  - DEFINE ApiState struct with Arc-wrapped components
  - INCLUDE RtspServer, VideoSourceManager, WatcherManager
  - ADD network simulator references
  - IMPLEMENT Clone for Axum state extraction
  - PROVIDE thread-safe access methods
  - MAINTAIN operation status tracking

Task 3:
CREATE crates/source-videos/src/api/routes/sources.rs:
  - IMPLEMENT GET /api/v1/sources - list all sources
  - IMPLEMENT GET /api/v1/sources/{id} - get source details
  - IMPLEMENT POST /api/v1/sources - add new source
  - IMPLEMENT DELETE /api/v1/sources/{id} - remove source
  - IMPLEMENT PUT /api/v1/sources/{id} - update source
  - ADD POST /api/v1/sources/batch - batch operations

Task 4:
CREATE crates/source-videos/src/api/routes/server.rs:
  - IMPLEMENT POST /api/v1/server/start - start RTSP server
  - IMPLEMENT POST /api/v1/server/stop - stop server
  - IMPLEMENT GET /api/v1/server/status - server status
  - IMPLEMENT GET /api/v1/server/info - server information
  - ADD POST /api/v1/server/restart - restart server
  - INCLUDE GET /api/v1/server/urls - list RTSP URLs

Task 5:
CREATE crates/source-videos/src/api/routes/config.rs:
  - IMPLEMENT GET /api/v1/config - get current configuration
  - IMPLEMENT PUT /api/v1/config - update configuration
  - IMPLEMENT GET /api/v1/config/defaults - get default config
  - ADD POST /api/v1/config/validate - validate config
  - SUPPORT configuration presets
  - HANDLE configuration persistence

Task 6:
CREATE crates/source-videos/src/api/routes/network.rs:
  - IMPLEMENT GET /api/v1/network/profiles - list profiles
  - IMPLEMENT POST /api/v1/network/apply - apply profile
  - IMPLEMENT PUT /api/v1/network/conditions - set custom conditions
  - IMPLEMENT GET /api/v1/network/status - current conditions
  - ADD POST /api/v1/network/reset - reset to perfect
  - SUPPORT per-source network conditions

Task 7:
CREATE crates/source-videos/src/api/routes/operations.rs:
  - IMPLEMENT POST /api/v1/generate - generate test video
  - IMPLEMENT POST /api/v1/scan - scan directory for videos
  - IMPLEMENT GET /api/v1/patterns - list test patterns
  - IMPLEMENT POST /api/v1/watch/start - start file watching
  - IMPLEMENT POST /api/v1/watch/stop - stop file watching
  - ADD GET /api/v1/watch/status - watcher status

Task 8:
CREATE crates/source-videos/src/api/routes/health.rs:
  - IMPLEMENT GET /api/v1/health - basic health check
  - IMPLEMENT GET /api/v1/health/live - liveness probe
  - IMPLEMENT GET /api/v1/health/ready - readiness probe
  - ADD GET /api/v1/metrics - Prometheus metrics
  - INCLUDE system resource information
  - PROVIDE detailed component status

Task 9:
CREATE crates/source-videos/src/api/auth.rs:
  - IMPLEMENT simple Bearer token authentication
  - ADD optional API key header support
  - CREATE auth middleware for protected routes
  - SUPPORT environment variable configuration
  - PROVIDE bypass for local/development
  - HANDLE unauthorized responses

Task 10:
CREATE crates/source-videos/src/api/error.rs:
  - DEFINE ApiError enum for all error types
  - IMPLEMENT IntoResponse for ApiError
  - MAP internal errors to HTTP status codes
  - PROVIDE detailed error messages
  - SUPPORT error context and tracing
  - HANDLE validation errors

Task 11:
CREATE crates/source-videos/src/api/models.rs:
  - DEFINE request/response DTOs with serde
  - IMPLEMENT From traits for conversions
  - ADD validation with validator crate
  - SUPPORT API versioning in models
  - PROVIDE comprehensive examples
  - ENSURE backward compatibility

Task 12:
UPDATE crates/source-videos/src/main.rs:
  - ADD --api flag to enable API server
  - CONFIGURE API port and bind address
  - SUPPORT concurrent CLI and API operation
  - HANDLE API server lifecycle
  - INTEGRATE with existing serve command
  - ADD API-specific configuration options

Task 13:
CREATE OpenAPI documentation:
  - INTEGRATE utoipa for OpenAPI generation
  - ANNOTATE all endpoints with utoipa macros
  - GENERATE OpenAPI 3.0 specification
  - SERVE Swagger UI at /api/docs
  - PROVIDE request/response examples
  - SUPPORT try-it-out functionality

Task 14:
CREATE integration tests:
  - TEST all REST endpoints
  - VERIFY error handling
  - TEST authentication flows
  - VALIDATE batch operations
  - TEST concurrent requests
  - BENCHMARK API performance

Task 15:
CREATE automation examples:
  - PROVIDE bash/curl examples
  - CREATE Python automation script
  - ADD PowerShell examples
  - INCLUDE CI/CD integration samples
  - DOCUMENT common automation patterns
  - PROVIDE monitoring integration examples
```

### Out of Scope
- WebSocket real-time updates (see PRP-17 for future)
- GraphQL interface
- Database persistence
- Multi-tenant support
- Complex authentication systems

## Success Criteria

- [ ] All CLI operations available via REST API
- [ ] Response time < 100ms for standard operations
- [ ] OpenAPI documentation complete and accurate
- [ ] Batch operations work atomically
- [ ] Authentication protects sensitive operations
- [ ] Integration tests pass with 100% coverage
- [ ] Automation examples work correctly

## Dependencies

### Technical Dependencies
- axum 0.8.4 (already in project)
- serde and serde_json for serialization
- utoipa for OpenAPI generation
- tower-http for middleware
- Optional: validator for request validation

### Knowledge Dependencies
- REST API design principles
- OpenAPI specification
- Axum routing and extractors
- Tower middleware patterns

## Risks and Mitigation

| Risk | Probability | Impact | Mitigation Strategy |
|------|------------|--------|-------------------|
| State synchronization issues | Medium | Medium | Use Arc<RwLock> for thread safety |
| API breaking changes | Low | High | Version API from start (/api/v1) |
| Performance degradation | Low | Medium | Benchmark and optimize hot paths |
| Security vulnerabilities | Medium | High | Use standard auth patterns, security audit |

## Architecture Decisions

### Decision: API Style
**Options Considered:**
1. REST with JSON
2. GraphQL
3. gRPC

**Decision:** Option 1 - REST with JSON

**Rationale:** Simplest for automation scripts, wide tooling support, matches existing patterns

### Decision: Authentication
**Options Considered:**
1. No authentication (local only)
2. Simple Bearer token
3. OAuth2/JWT

**Decision:** Option 2 - Simple Bearer token

**Rationale:** Balance between security and simplicity for automation

### Decision: Documentation
**Options Considered:**
1. Manual documentation
2. OpenAPI with utoipa
3. Custom documentation

**Decision:** Option 2 - OpenAPI with utoipa

**Rationale:** Industry standard, automatic client generation, integrated UI

## Validation Strategy

### Validation Commands
```bash
# Build and test
cargo build --release
cargo test api

# Run API server
cargo run -- serve --api --api-port 3000

# Test endpoints
curl http://localhost:3000/api/v1/health
curl http://localhost:3000/api/v1/sources
curl -X POST http://localhost:3000/api/v1/sources \
  -H "Content-Type: application/json" \
  -d '{"name":"test","pattern":"smpte"}'

# Verify OpenAPI docs
curl http://localhost:3000/api/docs/openapi.json

# Run integration tests
cargo test --test api_integration
```

## API Endpoint Examples

### Core Operations
```
GET    /api/v1/health              - Health check
GET    /api/v1/sources             - List all sources
POST   /api/v1/sources             - Add new source
DELETE /api/v1/sources/{id}        - Remove source
POST   /api/v1/server/start        - Start RTSP server
GET    /api/v1/server/status       - Server status
POST   /api/v1/network/apply       - Apply network profile
POST   /api/v1/generate            - Generate test video
```

### Automation Use Cases
```bash
# CI/CD: Start test server
curl -X POST http://localhost:3000/api/v1/server/start \
  -d '{"port":8554,"sources":["smpte","ball"]}'

# Monitoring: Check health
curl http://localhost:3000/api/v1/health

# Testing: Apply network conditions
curl -X POST http://localhost:3000/api/v1/network/apply \
  -d '{"profile":"poor"}'

# Batch operations
curl -X POST http://localhost:3000/api/v1/sources/batch \
  -d '{"operations":[{"add":"test1"},{"remove":"test2"}]}'
```

## Future Considerations

- WebSocket for real-time updates (PRP-17)
- GraphQL for flexible queries
- Webhook notifications for events
- Advanced authentication (OAuth2)
- Rate limiting and quotas
- Multi-tenant support

## References

- Axum 0.8.4 Documentation: https://docs.rs/axum/0.8.4/
- OpenAPI Specification: https://swagger.io/specification/
- REST API Best Practices: https://restfulapi.net/
- utoipa Documentation: https://docs.rs/utoipa/

---

## PRP Metadata

- **Author**: Claude
- **Created**: 2025-01-25
- **Status**: Ready for Implementation
- **Confidence Level**: 9/10 - Clear requirements with existing patterns to follow