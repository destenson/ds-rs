# PRP: Detection Data Export and Streaming Integration

## Executive Summary

Implement real-time export of bounding box detection data to external systems for analytics, persistence, and downstream processing. This PRP establishes data export pipelines that stream detection metadata (coordinates, confidence scores, timestamps) to various backends including MQTT brokers, RabbitMQ, databases (PostgreSQL/MongoDB), and file systems, enabling comprehensive analytics and audit trails.

## Problem Statement

### Current State
- Detection results (ObjectMeta with BoundingBox) exist only within the processing pipeline
- No persistence or external visibility of detection data
- No integration with analytics or monitoring systems
- Detection history not available for pattern analysis or replay
- No real-time streaming to external consumers

### Desired State
- Real-time streaming of detection data to multiple configurable destinations
- Structured data export with timestamps, coordinates, and confidence scores
- Support for multiple export backends (MQTT, RabbitMQ, databases, files)
- Buffered and reliable delivery with retry mechanisms
- Minimal performance impact on detection pipeline

### Business Value
Enables integration with enterprise analytics platforms, provides audit trails for compliance, supports real-time monitoring dashboards, and creates data foundations for machine learning model improvement and operational intelligence.

## Requirements

### Functional Requirements

1. **Data Serialization**: Convert ObjectMeta/BoundingBox to structured formats (JSON, Protocol Buffers)
2. **Multi-Backend Export**: Support MQTT, RabbitMQ, PostgreSQL, MongoDB, and file append
3. **Streaming Interface**: Real-time data export as detections occur
4. **Batching Support**: Efficient batching for database inserts and file operations
5. **Schema Management**: Consistent data schema across all export targets
6. **Error Recovery**: Retry logic and dead letter queues for failed exports
7. **Performance Monitoring**: Track export latency and throughput metrics

### Non-Functional Requirements

1. **Performance**: Export overhead <2% of detection pipeline latency
2. **Reliability**: At-least-once delivery guarantees with retry mechanisms
3. **Scalability**: Handle 100+ detections/second across multiple streams
4. **Memory Efficiency**: Bounded buffers to prevent memory exhaustion
5. **Configuration**: Runtime-configurable export destinations and formats

### Context and Research

The detection pipeline generates rich metadata that has value beyond real-time visualization:
- **ObjectMeta**: Contains object_id, class_id, confidence, tracker_id
- **BoundingBox**: Provides left, top, width, height coordinates
- **Timestamps**: Frame timestamps for temporal analysis
- **Source Information**: Stream ID/URI for multi-camera correlation

Export targets serve different purposes:
- **MQTT**: IoT integration and edge device notifications
- **RabbitMQ**: Enterprise message queuing and workflow integration
- **PostgreSQL**: Structured queries and relational analytics
- **MongoDB**: Time-series analysis and geospatial queries
- **Files**: Audit logs and offline analysis

### Documentation & References
```yaml
- file: crates/ds-rs/src/metadata/object.rs
  why: ObjectMeta and BoundingBox structures to be exported

- file: crates/ds-rs/src/inference/mod.rs
  why: DetectionResult structure containing all detection metadata

- url: https://docs.rs/rumqttc/latest/rumqttc/
  why: MQTT v5 client for Rust with async Tokio support

- url: https://github.com/rabbitmq/rabbitmq-stream-rust-client
  why: Official RabbitMQ Stream client for high-throughput messaging

- url: https://docs.rs/lapin/latest/lapin/
  why: Full AMQP protocol implementation for RabbitMQ

- url: https://docs.rs/tokio-postgres/latest/tokio_postgres/
  why: Async PostgreSQL client for time-series data insertion

- url: https://docs.rs/mongodb/latest/mongodb/
  why: MongoDB Rust driver with change streams and time-series support

- url: https://www.mongodb.com/docs/drivers/rust/current/fundamentals/time-series/
  why: MongoDB time-series collections for detection history

- file: crates/ds-rs/src/source/events.rs
  why: Event handling patterns for async data export
```

### List of tasks to be completed to fulfill the PRP

```yaml
Task 1:
CREATE src/export/mod.rs:
  - DEFINE DataExporter trait for export backends
  - IMPLEMENT ExportManager for coordinating multiple exporters
  - ADD buffering and batching strategies
  - INCLUDE metrics collection for export performance

Task 2:
CREATE src/export/schema.rs:
  - DEFINE DetectionData structure for serialization
  - ADD JSON and Protocol Buffer serialization
  - INCLUDE timestamp formatting standards
  - SUPPORT schema versioning for compatibility

Task 3:
CREATE src/export/mqtt.rs:
  - IMPLEMENT MQTT exporter using rumqttc
  - ADD topic configuration for different detection types
  - INCLUDE QoS level configuration
  - HANDLE connection recovery and retry

Task 4:
CREATE src/export/rabbitmq.rs:
  - IMPLEMENT RabbitMQ Stream exporter
  - ADD exchange and routing key configuration
  - INCLUDE message persistence options
  - HANDLE super stream partitioning for scale

Task 5:
CREATE src/export/postgres.rs:
  - IMPLEMENT PostgreSQL time-series exporter
  - CREATE detection_events table schema
  - ADD batch insert optimization
  - INCLUDE connection pooling with deadpool

Task 6:
CREATE src/export/mongodb.rs:
  - IMPLEMENT MongoDB time-series collection exporter
  - CONFIGURE time-series collection parameters
  - ADD geospatial indexing for coordinate queries
  - INCLUDE change stream support for downstream consumers

Task 7:
CREATE src/export/file.rs:
  - IMPLEMENT file-based exporter with rotation
  - ADD CSV and JSONL format support
  - INCLUDE compression options (gzip)
  - HANDLE file rotation by size/time

Task 8:
CREATE src/export/config.rs:
  - DEFINE ExportConfig for all backend configurations
  - ADD runtime configuration loading
  - INCLUDE environment variable support
  - SUPPORT multiple simultaneous export destinations

Task 9:
MODIFY src/inference/mod.rs:
  - INTEGRATE export pipeline with DetectionResult
  - ADD export hooks in detection processing
  - INCLUDE async export without blocking detection
  - HANDLE export backpressure gracefully

Task 10:
CREATE examples/detection_export_demo.rs:
  - DEMONSTRATE multi-backend export configuration
  - SHOW real-time streaming to MQTT and RabbitMQ
  - INCLUDE database insertion examples
  - VALIDATE export performance metrics

Task 11:
CREATE tests/export_integration_tests.rs:
  - TEST each export backend independently
  - VALIDATE data schema consistency
  - BENCHMARK export throughput
  - INCLUDE failure recovery scenarios

Task 12:
CREATE scripts/setup_export_backends.sh:
  - PROVIDE Docker Compose for test infrastructure
  - SETUP MQTT broker, RabbitMQ, PostgreSQL, MongoDB
  - CREATE database schemas and collections
  - INCLUDE sample consumer applications
```

### Out of Scope
- Data transformation or enrichment beyond basic serialization
- Complex event processing or stream analytics
- Data aggregation or summarization before export
- Encryption of data in transit (rely on transport layer security)

## Success Criteria

- [ ] Detection data exports to all configured backends in real-time
- [ ] Export latency <50ms for message queues, <100ms for databases
- [ ] Successful handling of 100+ detections/second
- [ ] Zero data loss with retry mechanisms functioning
- [ ] Consistent data schema across all export destinations
- [ ] Example application demonstrates multi-backend export
- [ ] Docker Compose setup enables quick testing of all backends

## Dependencies

### Technical Dependencies
- Completed PRPs 10-12 for detection data generation
- External systems: MQTT broker, RabbitMQ, PostgreSQL, MongoDB
- Async runtime (Tokio) for non-blocking exports
- Serialization libraries (serde_json, prost for protobuf)

### Knowledge Dependencies
- Message queue patterns and best practices
- Time-series database design
- Async programming and backpressure handling
- Data serialization formats and schemas

## Risks and Mitigation

| Risk | Probability | Impact | Mitigation Strategy |
|------|------------|--------|-------------------|
| Export backpressure affecting detection | Medium | High | Implement bounded buffers and dropping strategies |
| Network failures causing data loss | High | Medium | Add retry queues and persistent buffering |
| Schema evolution breaking consumers | Medium | Medium | Version schemas and maintain compatibility |
| Database performance degradation | Medium | High | Use batch inserts and connection pooling |

## Architecture Decisions

### Decision: Export Architecture
**Options Considered:**
1. Synchronous export in detection pipeline
2. Async export with bounded channels
3. Separate export service with IPC

**Decision:** Option 2 - Async export with bounded channels

**Rationale:** Minimizes impact on detection pipeline while maintaining data locality

### Decision: Data Format
**Options Considered:**
1. JSON only for simplicity
2. Protocol Buffers for efficiency
3. Pluggable serialization formats

**Decision:** Option 3 - Pluggable formats with JSON default

**Rationale:** Provides flexibility for different backend requirements while maintaining simplicity

### Decision: Delivery Guarantees
**Options Considered:**
1. Best-effort delivery
2. At-least-once with retries
3. Exactly-once with deduplication

**Decision:** Option 2 - At-least-once delivery

**Rationale:** Balances reliability with implementation complexity

## Validation Strategy

### Validation Commands
```bash
# Start test infrastructure
docker-compose -f scripts/docker-compose.export.yml up -d

# Build with export features
cargo build --features opencv,export

# Test export to all backends
cargo test --features opencv,export export_integration

# Run export demo with ball detection
cargo run --example detection_export_demo rtsp://127.0.0.1:8554/test2

# Monitor export metrics
cargo run --example export_metrics_dashboard

# Verify data in backends
psql -h localhost -U postgres -d detections -c "SELECT * FROM detection_events ORDER BY timestamp DESC LIMIT 10;"
mongosh --eval "db.detections.find().sort({timestamp:-1}).limit(10)"
```

## Future Considerations

- Apache Kafka integration for high-throughput scenarios
- ClickHouse integration for analytics workloads
- Prometheus metrics export for monitoring
- GraphQL subscriptions for real-time data access
- Data lake export (S3, Azure Blob) for long-term storage
- Stream processing integration (Apache Flink, Spark Streaming)

## References

- MQTT v5 Specification
- RabbitMQ Streams Documentation
- PostgreSQL Time-Series Best Practices
- MongoDB Time Series Collections Guide
- Async Rust Patterns for Data Pipelines

---

## PRP Metadata

- **Author**: Claude
- **Created**: 2025-08-23
- **Last Modified**: 2025-08-23
- **Status**: Draft
- **Confidence Level**: 8 - Well-established patterns with mature Rust libraries available