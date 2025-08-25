# PRP-19: Network Simulation for Source Videos (Simplified Implementation)

## Executive Summary

Add network simulation capabilities to the source-videos crate to test the error recovery system (PRP-34) with realistic network conditions including packet loss, bandwidth limits, latency, and connection interruptions.

## Simplified Scope

Focus on the most valuable features for testing error recovery:
1. **Connection interruptions** - Simulate network disconnects
2. **Packet loss** - Random and burst patterns
3. **Bandwidth throttling** - Limit throughput
4. **Latency injection** - Add delays
5. **Predefined profiles** - Common network conditions

## Implementation Plan

### Phase 1: Core Network Simulation (What we'll implement now)

1. **Network Simulator Module**
   - Basic simulator trait
   - Condition profiles (3G, 4G, WiFi, etc.)
   - Enable/disable control

2. **GStreamer Integration**
   - Use queue element for buffering/dropping
   - Use identity element for latency
   - Valve element for connection control

3. **Connection Interruptions**
   - Periodic disconnect/reconnect
   - Random connection drops
   - Configurable durations

4. **Basic Packet Loss**
   - Random uniform loss
   - Simple burst patterns

5. **Integration with RTSP Server**
   - Add simulation to RTSP pipelines
   - Per-stream control

### Phase 2: Advanced Features (Future)
- Gilbert-Elliott model for realistic packet loss
- Token bucket bandwidth shaping
- Jitter simulation
- Network metrics collection

## Success Criteria

- [ ] Can trigger error recovery in ds-rs
- [ ] Connection drops cause retry attempts
- [ ] Packet loss triggers health monitoring
- [ ] Bandwidth limits cause buffer underruns
- [ ] All with < 5% CPU overhead

## Simplified Task List

```yaml
Task 1: Create basic network simulator
Task 2: Add GStreamer pipeline integration  
Task 3: Implement connection control
Task 4: Add packet loss simulation
Task 5: Create network profiles
Task 6: Integrate with RTSP server
Task 7: Add example demonstrating with ds-rs
Task 8: Create tests
```

This simplified version will be sufficient to test our error recovery system while being much quicker to implement.