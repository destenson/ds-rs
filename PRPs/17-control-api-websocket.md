# PRP: Control API & WebSocket Interface

**Status**: PARTIAL (as of 2025-08-27) - REST API implemented with Axum, no WebSocket yet

## Executive Summary

Build a comprehensive control interface for the source-videos application using REST API for command/control and WebSocket for real-time updates. This enables external applications, scripts, and UIs to manage video sources dynamically, receive status updates, and control all aspects of the service programmatically.

## Problem Statement

### Current State
- Only CLI interface for control
- No programmatic access to source management
- No real-time status updates
- Limited observability into running sources
- No remote control capability
- Manual intervention required for changes

### Desired State
- REST API for CRUD operations on sources
- WebSocket for real-time status and events
- Authentication and authorization support
- Comprehensive source metrics and statistics
- Remote management capability
- Scriptable control interface

### Business Value
Enables integration with monitoring systems, supports building custom UIs, allows automation and orchestration, provides production-ready management interface, and facilitates testing and debugging.

## Requirements

### Functional Requirements

1. **REST API Endpoints**: Full CRUD for sources and configuration
2. **WebSocket Events**: Real-time updates and bidirectional communication
3. **Authentication**: Optional but configurable security
4. **Metrics Export**: Source statistics and health data
5. **Batch Operations**: Atomic multi-source updates
6. **Query Interface**: Filter and search sources
7. **Control Commands**: Start, stop, restart operations

### Non-Functional Requirements

1. **Performance**: Handle 100+ concurrent WebSocket connections
2. **Latency**: API response time < 50ms
3. **Reliability**: Graceful handling of client disconnections
4. **Security**: Support TLS and authentication tokens
5. **Compatibility**: OpenAPI specification compliance

### Context and Research

Axum provides excellent WebSocket support through WebSocketUpgrade extractor and integrates seamlessly with Tokio. It leverages Tower middleware for cross-cutting concerns like authentication, rate limiting, and CORS. The framework is production-ready and used by major Rust projects.

WebSocket enables bidirectional communication for real-time updates without polling overhead. JSON-RPC or REST+WebSocket hybrid approaches are common patterns. The current VideoSourceManager can be wrapped with an Axum state layer for thread-safe access.

### Documentation & References

```yaml
- url: https://docs.rs/axum/latest/axum/
  why: Axum web framework documentation

- url: https://docs.rs/axum/latest/axum/extract/ws/index.html
  why: WebSocket implementation in Axum

- url: https://github.com/tokio-rs/axum/tree/main/examples/websockets
  why: Official WebSocket examples

- file: crates/source-videos/src/manager.rs
  why: VideoSourceManager to expose via API

- url: https://www.shuttle.dev/blog/2023/12/06/using-axum-rust
  why: Production Axum patterns and best practices

- url: https://docs.rs/tower/latest/tower/
  why: Middleware ecosystem for Axum

- url: https://swagger.io/specification/
  why: OpenAPI specification for API documentation
```

### List of tasks to be completed

```yaml
Task 1:
CREATE crates/source-videos/src/api/mod.rs:
  - DEFINE ApiServer struct
  - CONFIGURE Axum router
  - SETUP middleware stack
  - IMPLEMENT graceful shutdown
  - ADD metrics collection
  - INTEGRATE with VideoSourceManager

Task 2:
CREATE crates/source-videos/src/api/routes/sources.rs:
  - IMPLEMENT GET /api/sources (list all)
  - IMPLEMENT GET /api/sources/:id (get one)
  - IMPLEMENT POST /api/sources (create)
  - IMPLEMENT PUT /api/sources/:id (update)
  - IMPLEMENT DELETE /api/sources/:id (remove)
  - ADD query parameters for filtering

Task 3:
CREATE crates/source-videos/src/api/routes/config.rs:
  - IMPLEMENT GET /api/config (current config)
  - IMPLEMENT PUT /api/config (update config)
  - IMPLEMENT POST /api/config/reload (trigger reload)
  - IMPLEMENT GET /api/config/schema (config schema)
  - ADD config validation endpoint

Task 4:
CREATE crates/source-videos/src/api/websocket.rs:
  - IMPLEMENT WebSocket upgrade handler
  - CREATE WebSocketConnection manager
  - HANDLE client subscriptions
  - IMPLEMENT message routing
  - ADD connection pooling
  - SUPPORT reconnection logic

Task 5:
CREATE crates/source-videos/src/api/messages.rs:
  - DEFINE WebSocket message types
  - IMPLEMENT JSON serialization
  - ADD message versioning
  - CREATE event types enum
  - SUPPORT request/response correlation

Task 6:
CREATE crates/source-videos/src/api/auth.rs:
  - IMPLEMENT Bearer token authentication
  - ADD API key support
  - CREATE permission system
  - IMPLEMENT rate limiting
  - ADD CORS configuration
  - SUPPORT TLS termination

Task 7:
CREATE crates/source-videos/src/api/state.rs:
  - WRAP VideoSourceManager in Arc
  - IMPLEMENT AppState struct
  - ADD WebSocket client registry
  - MAINTAIN connection tracking
  - PROVIDE metrics collection
  - HANDLE concurrent access

Task 8:
CREATE crates/source-videos/src/api/handlers.rs:
  - IMPLEMENT source CRUD handlers
  - ADD batch operation support
  - CREATE health check endpoint
  - IMPLEMENT metrics endpoint
  - ADD OpenAPI documentation
  - HANDLE error responses

Task 9:
UPDATE crates/source-videos/src/manager.rs:
  - ADD event emission for changes
  - IMPLEMENT Subscribe trait
  - SUPPORT change notifications
  - ADD metrics collection
  - PROVIDE state snapshots
  - ENHANCE thread safety

Task 10:
CREATE crates/source-videos/src/api/broadcast.rs:
  - IMPLEMENT event broadcaster
  - FILTER events by subscription
  - HANDLE backpressure
  - SUPPORT event replay
  - ADD event persistence
  - OPTIMIZE for many clients

Task 11:
CREATE WebSocket client features:
  - IMPLEMENT heartbeat/ping-pong
  - ADD automatic reconnection
  - SUPPORT subscription management
  - HANDLE connection state
  - PROVIDE event buffering
  - ADD compression support

Task 12:
CREATE crates/source-videos/src/api/metrics.rs:
  - COLLECT source statistics
  - TRACK API usage
  - MONITOR WebSocket connections
  - EXPORT Prometheus metrics
  - ADD custom metrics
  - IMPLEMENT aggregation

Task 13:
ADD OpenAPI documentation:
  - GENERATE OpenAPI spec
  - DOCUMENT all endpoints
  - PROVIDE example requests
  - ADD response schemas
  - INCLUDE authentication docs
  - SUPPORT try-it-out feature

Task 14:
CREATE integration tests:
  - TEST REST API endpoints
  - VERIFY WebSocket communication
  - TEST authentication flows
  - BENCHMARK concurrent connections
  - VALIDATE error handling

Task 15:
CREATE client libraries:
  - BUILD Rust client crate
  - ADD JavaScript/TypeScript client
  - PROVIDE Python client
  - DOCUMENT usage examples
  - ADD reconnection logic
```

### Out of Scope
- GraphQL API
- gRPC interface  
- Database persistence
- Multi-node clustering
- Load balancing

## Success Criteria

- [ ] REST API handles 1000+ requests/second
- [ ] WebSocket supports 100+ concurrent connections
- [ ] All CRUD operations functional
- [ ] Real-time updates delivered < 10ms
- [ ] OpenAPI documentation complete
- [ ] Authentication and authorization working

## Dependencies

### Technical Dependencies
- Axum web framework
- Tower middleware
- tokio-tungstenite for WebSocket
- serde for JSON serialization
- Optional: OpenTelemetry for tracing

### Knowledge Dependencies
- REST API design principles
- WebSocket protocol
- Authentication patterns
- OpenAPI specification

## Risks and Mitigation

| Risk | Probability | Impact | Mitigation Strategy |
|------|------------|--------|-------------------|
| WebSocket connection storms | Medium | High | Rate limiting and backpressure |
| Authentication bypass | Low | Critical | Security audit and testing |
| Memory leaks from connections | Medium | Medium | Connection limits and timeouts |
| API versioning issues | Low | Low | Version from start |

## Architecture Decisions

### Decision: Web Framework
**Options Considered:**
1. Actix-web
2. Axum
3. Warp

**Decision:** Option 2 - Axum

**Rationale:** Best Tokio integration, Tower middleware ecosystem, active development

### Decision: WebSocket Protocol
**Options Considered:**
1. Raw WebSocket with custom protocol
2. Socket.IO compatible
3. JSON-RPC over WebSocket

**Decision:** Option 3 - JSON-RPC over WebSocket

**Rationale:** Standard protocol, good tooling support, request/response correlation

## Validation Strategy

- **Unit Tests**: Test handlers and message processing
- **Integration Tests**: End-to-end API flows
- **Load Tests**: Concurrent connection handling
- **Security Tests**: Authentication and authorization

## Future Considerations

- GraphQL API addition
- Multi-tenant support
- Distributed event bus
- WebRTC signaling
- Webhook notifications

## References

- Axum Documentation
- Tower Middleware Guide
- WebSocket RFC 6455
- OpenAPI Specification 3.0
- JSON-RPC 2.0 Specification

---

## PRP Metadata

- **Author**: Claude
- **Created**: 2025-08-23
- **Last Modified**: 2025-08-23
- **Status**: Complete
- **Confidence Level**: 9 - Well-established patterns with excellent framework support
