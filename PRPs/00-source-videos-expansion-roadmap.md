# Source-Videos CLI Expansion Roadmap

## Overview

This document provides a roadmap for implementing the comprehensive expansion of the source-videos CLI tool. The expansion is broken into 6 focused PRPs that can be implemented independently but work together to create a powerful video source testing and serving platform.

## PRPs Summary

### PRP-35: Directory and File List Support
**Priority: HIGH**
- Serve video files from directories with recursive traversal
- Support file lists and multiple formats
- Auto-detection of video formats
- Dynamic mount point generation

### PRP-36: File Watching and Auto-Reload  
**Priority: HIGH**
- Monitor directories for file changes
- Automatic stream creation/removal
- Hot-reload of modified files
- Auto-repeat/loop functionality

### PRP-37: Enhanced Configuration System
**Priority: MEDIUM**
- Comprehensive configuration for all features
- Profile-based configurations
- Environment variable substitution
- Configuration validation and hot-reload

### PRP-38: Advanced CLI Options
**Priority: HIGH**
- Complete command-line interface
- Support for all features via CLI
- Shell completion support
- Intuitive options and defaults

### PRP-39: REPL Mode Enhancements
**Priority: MEDIUM**
- Full-featured interactive control
- Dynamic source management
- Real-time network simulation control
- Command history and completion

### PRP-40: Network Simulation Integration
**Priority: HIGH**
- Per-source network conditions
- Dynamic condition changes
- Time-based scenarios
- Comprehensive metrics

## Implementation Order

### Phase 1: Core File Serving (Weeks 1-2)
1. **PRP-35**: Directory and File List Support
   - Essential foundation for file-based serving
   - Enables real video file testing
   - No dependencies on other PRPs

2. **PRP-38**: Advanced CLI Options (partial)
   - Basic CLI options for directory serving
   - Essential for testing Phase 1 features

### Phase 2: Dynamic Capabilities (Weeks 3-4)
3. **PRP-36**: File Watching and Auto-Reload
   - Builds on directory serving
   - Adds dynamic behavior
   - Enables continuous testing scenarios

4. **PRP-40**: Network Simulation Integration
   - Can be developed in parallel
   - High value for testing
   - Leverages existing network module

### Phase 3: Enhanced Control (Weeks 5-6)
5. **PRP-37**: Enhanced Configuration System
   - Supports all previous features
   - Enables complex deployments
   - Can be developed incrementally

6. **PRP-39**: REPL Mode Enhancements
   - Provides runtime control
   - Integrates all features
   - Best implemented after core features

7. **PRP-38**: Advanced CLI Options (completion)
   - Final CLI enhancements
   - Shell completions
   - Advanced options

## Key Dependencies

### External Crates to Add
```toml
[dependencies]
walkdir = "2.4"          # Directory traversal
notify = "6.1"           # File watching (already present)
rustyline = "13.0"       # REPL enhancements
comfy-table = "7.0"      # Table formatting
regex = "1.10"           # Pattern matching
mime_guess = "2.0"       # File type detection
clap_complete = "4.4"    # Shell completions
```

### Existing Infrastructure to Leverage
- `src/network/` - Network simulation framework
- `src/config/watcher.rs` - File watching patterns
- `src/rtsp/` - RTSP server infrastructure
- `src/pipeline/` - GStreamer pipeline building

## Testing Strategy

### Unit Tests
- Directory scanning and filtering
- File watching event handling
- Configuration parsing and validation
- Network simulation accuracy

### Integration Tests
- End-to-end file serving
- Dynamic source addition/removal
- Network simulation with streaming
- Configuration hot-reload

### Performance Tests
- Large directory handling (1000+ files)
- Multiple concurrent watchers
- Network simulation overhead
- Memory usage monitoring

## Success Metrics

1. **Functionality**
   - All video files in a directory can be served
   - File changes are detected within 2 seconds
   - Network conditions accurately simulated
   - Configuration changes apply without restart

2. **Performance**
   - Startup time < 1 second for 100 files
   - Network simulation overhead < 5%
   - Memory usage scales linearly with sources
   - CPU usage minimal when idle

3. **Usability**
   - Common use cases require < 3 CLI options
   - REPL commands are intuitive
   - Error messages are helpful
   - Documentation is comprehensive

## Risk Mitigation

### Technical Risks
- **Platform Compatibility**: Test early on Windows/Linux/macOS
- **Performance**: Profile and optimize critical paths
- **Complexity**: Incremental implementation, good defaults

### Schedule Risks
- **Scope Creep**: Stick to PRP boundaries
- **Dependencies**: Parallel development where possible
- **Testing**: Automated tests from the start

## Future Enhancements (Post-MVP)

1. **Cloud Integration**
   - S3/GCS bucket serving
   - Cloud-native deployment
   - Distributed serving

2. **Advanced Features**
   - Playlist management (lower priority per user feedback)
   - Thumbnail generation
   - Metadata extraction
   - Web UI

3. **Production Features**
   - Prometheus metrics
   - Health checks
   - Rate limiting
   - Authentication

## Validation Checklist

### Phase 1 Complete
- [ ] Can serve all MP4 files from a directory
- [ ] Recursive directory traversal works
- [ ] Basic CLI options functional
- [ ] RTSP streams accessible

### Phase 2 Complete
- [ ] File changes trigger stream updates
- [ ] Auto-repeat works for continuous playback
- [ ] Network simulation applies to streams
- [ ] Multiple scenarios can run

### Phase 3 Complete
- [ ] Configuration hot-reload works
- [ ] REPL provides full control
- [ ] Shell completions installed
- [ ] All features documented

## Commands Quick Reference

### Basic Usage
```bash
# Serve directory
source-videos serve -d /videos

# With watching
source-videos serve -d /videos --watch --auto-repeat

# With network simulation  
source-videos serve -d /videos --network-profile lossy
```

### Advanced Usage
```bash
# Full featured
source-videos serve \
  -d /videos --recursive \
  --watch --reload-on-change \
  --network-scenario degrading.yaml \
  --config production.toml \
  --daemon --metrics
```

### REPL Mode
```bash
source-videos interactive
> add source directory /videos --watch
> network apply source-1 residential
> status
> help
```

## Documentation Requirements

1. **User Guide**
   - Quick start guide
   - Common use cases
   - Configuration examples
   - Troubleshooting

2. **API Documentation**
   - CLI reference
   - Configuration schema
   - REPL commands
   - Network profiles

3. **Examples**
   - Directory serving
   - File watching
   - Network simulation
   - Complex scenarios

---

## Roadmap Metadata

- **Author**: Claude
- **Created**: 2025-01-25
- **Total PRPs**: 6
- **Estimated Duration**: 6 weeks
- **Priority Focus**: Directory serving, file watching, network simulation
- **Lower Priority**: Playlist functionality (as noted by user)