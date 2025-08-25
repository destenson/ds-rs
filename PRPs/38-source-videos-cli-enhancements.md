# PRP: Advanced CLI Options for Source-Videos

## Executive Summary

Enhance the source-videos command-line interface with comprehensive options for all new features including directory serving, playlist modes, file watching, network simulation, and advanced source control. Provide both short and long-form options with intuitive defaults.

## Problem Statement

### Current State
- Limited CLI options focused on basic serving
- No support for directory or file list inputs
- Cannot configure advanced features from command line
- No playlist or sequencing options
- Limited runtime control options

### Desired State
- Comprehensive CLI supporting all features
- Intuitive short and long options
- Playlist mode for sequential/random playback
- Network simulation controls
- Rich filtering and selection options
- Runtime control via signals or commands
- Shell completion support

### Business Value
Enables rapid testing and deployment without configuration files, supports scripting and automation, provides flexibility for different use cases, and improves developer experience.

## Requirements

### Functional Requirements

1. **Source Specification**: Files, directories, patterns, playlists
2. **Playlist Modes**: Sequential, random, shuffle, repeat
3. **Watch Options**: Enable/disable watching, intervals
4. **Network Simulation**: Apply conditions via CLI
5. **Filter Options**: Include/exclude patterns
6. **Output Control**: Logging, status, metrics
7. **Runtime Commands**: Pause, skip, reload
8. **Batch Operations**: Multiple sources in one command

### Non-Functional Requirements

1. **Usability**: Intuitive option names and shortcuts
2. **Discoverability**: Good help text and examples
3. **Compatibility**: POSIX-style arguments
4. **Performance**: Fast argument parsing
5. **Extensibility**: Easy to add new options

### Context and Research

Current CLI structure in `src/main.rs` uses clap with subcommands:
- serve, generate, list, interactive, test

The clap crate provides excellent CLI building with:
- Derive API for type-safe arguments
- Automatic help generation
- Shell completion scripts
- Validation and error messages

### Documentation & References

```yaml
- file: crates/source-videos/src/main.rs
  why: Current CLI implementation to extend

- url: https://docs.rs/clap/latest/clap/
  why: CLI framework documentation

- url: https://clig.dev/
  why: Command Line Interface Guidelines

- file: crates/ds-rs/src/main.rs
  why: Reference for complex CLI patterns

- url: https://docs.rs/clap_complete/latest/clap_complete/
  why: Shell completion generation
```

### List of tasks to be completed

```yaml
Task 1:
EXTEND src/main.rs Commands enum:
  - ADD ServeFiles variant for file/directory serving
  - ADD Playlist variant for playlist mode
  - ADD Monitor variant for watching with stats
  - ADD Simulate variant for network testing
  - EXTEND Serve with new options

Task 2:
ADD source specification options:
  - IMPLEMENT --files/-f for file lists
  - ADD --directory/-d for directories
  - ADD --recursive/-r for subdirectories
  - ADD --playlist/-p for playlist mode
  - SUPPORT multiple source types in one command
  - ADD --mount-prefix for URL generation

Task 3:
IMPLEMENT playlist options:
  - ADD --playlist-mode (sequential|random|shuffle)
  - ADD --playlist-repeat (none|all|one)
  - ADD --playlist-file for m3u/pls files
  - ADD --transition-duration for gaps
  - ADD --crossfade for smooth transitions
  - IMPLEMENT --playlist-from-dir to treat directory as playlist

Task 4:
ADD filtering options:
  - IMPLEMENT --include/-i patterns
  - ADD --exclude/-e patterns
  - ADD --format to filter by video format
  - ADD --min-duration/--max-duration
  - ADD --modified-since for date filtering
  - SUPPORT regex patterns

Task 5:
IMPLEMENT watch options:
  - ADD --watch/-w to enable watching
  - ADD --watch-interval for poll frequency
  - ADD --reload-on-change
  - ADD --watch-events to select event types
  - ADD --no-initial-scan option

Task 6:
ADD network simulation options:
  - IMPLEMENT --network-profile
  - ADD --packet-loss percentage
  - ADD --latency milliseconds
  - ADD --bandwidth limit
  - ADD --jitter variance
  - ADD --connection-drops frequency

Task 7:
IMPLEMENT output options:
  - ADD --verbose/-v with levels
  - ADD --quiet/-q for silent operation
  - ADD --status-interval for periodic updates
  - ADD --metrics to show performance data
  - ADD --list-streams to show active streams
  - ADD --output-format (text|json|csv)

Task 8:
ADD control options:
  - IMPLEMENT --daemon/-D for background mode
  - ADD --pid-file for process management
  - ADD --control-socket for runtime commands
  - ADD --signal-handlers for graceful shutdown
  - ADD --max-streams limit

Task 9:
CREATE shell completions:
  - GENERATE bash completion script
  - ADD zsh completion support
  - CREATE fish completion
  - ADD PowerShell completion
  - INCLUDE in installation

Task 10:
ENHANCE help system:
  - ADD examples to help text
  - CREATE detailed man page
  - ADD --help-all for complete options
  - IMPLEMENT --show-config to display active settings
  - ADD --dry-run to preview without starting
```

### Out of Scope
- GUI application
- Web-based configuration interface
- Interactive menu system (beyond current REPL)
- Remote CLI client

## Success Criteria

- [ ] All features accessible via CLI
- [ ] Playlist mode works for directories
- [ ] Multiple source types can be combined
- [ ] Network simulation configurable from CLI
- [ ] Shell completions work correctly
- [ ] Help text is comprehensive and clear
- [ ] Common use cases require minimal options
- [ ] Complex scenarios are possible with full options

## Dependencies

### Technical Dependencies
- clap 4.x with derive feature
- clap_complete for shell completions
- regex for pattern matching
- chrono for date filtering

### Knowledge Dependencies
- CLI design best practices
- Shell completion mechanisms
- Signal handling for daemon mode

## Risks and Mitigation

| Risk | Probability | Impact | Mitigation Strategy |
|------|------------|--------|-------------------|
| Option complexity | Medium | Medium | Group related options, good defaults |
| Argument conflicts | Low | Low | Clap validation and conflicts |
| Platform differences | Low | Medium | Test on Windows/Linux/macOS |
| Performance with many args | Low | Low | Lazy evaluation where possible |

## Architecture Decisions

### Decision: Playlist Implementation
**Options Considered:**
1. Separate streams for each file
2. Single stream with sequential playback
3. Both modes available

**Decision:** Option 3 - Both modes with --playlist flag

**Rationale:** Provides flexibility for different use cases

### Decision: Option Style
**Options Considered:**
1. GNU style (--long-option)
2. Single dash (-option)
3. Mixed with conventional shortcuts

**Decision:** Option 3 - Mixed conventional

**Rationale:** Familiar to users, follows Unix conventions

## Validation Strategy

### Validation Commands
```bash
# Serve directory as separate streams
cargo run -- serve -d /videos -r --watch

# Serve directory as playlist
cargo run -- serve -d /videos --playlist --playlist-mode shuffle

# Multiple sources
cargo run -- serve -d /videos -f file1.mp4 -f file2.mkv --patterns smpte

# Network simulation
cargo run -- serve -d /videos --network-profile lossy --packet-loss 5

# Filter options
cargo run -- serve -d /videos --include "*.mp4" --exclude "*test*"

# Daemon mode with metrics
cargo run -- serve -d /videos --daemon --metrics --status-interval 10

# Generate completions
cargo run -- completions bash > /etc/bash_completion.d/source-videos
```

## CLI Examples

### Basic Usage
```bash
# Serve all MP4 files from directory
source-videos serve -d /media/videos -i "*.mp4"

# Playlist mode with shuffle
source-videos serve -d /media/videos --playlist --playlist-mode shuffle --playlist-repeat all

# Watch directory with auto-reload
source-videos serve -d /media/videos --watch --reload-on-change

# Multiple sources with different configs
source-videos serve \
  -d /media/videos --recursive \
  -f /special/file.mp4 \
  --patterns smpte,ball \
  --port 8554
```

### Advanced Usage
```bash
# Full featured example
source-videos serve \
  --directory /media/videos \
  --recursive \
  --playlist \
  --playlist-mode sequential \
  --playlist-repeat all \
  --transition-duration 2s \
  --watch \
  --watch-interval 5s \
  --reload-on-change \
  --network-profile residential \
  --include "*.mp4" "*.mkv" \
  --exclude "*temp*" "*backup*" \
  --mount-prefix /streams \
  --verbose \
  --metrics \
  --daemon \
  --pid-file /var/run/source-videos.pid
```

## Future Considerations

- REST API for runtime control
- Configuration wizard mode
- Performance profiling options
- Cloud source support (S3, GCS)
- Cluster mode for distributed serving

## References

- Clap documentation: https://docs.rs/clap/
- Command Line Interface Guidelines: https://clig.dev/
- GNU Coding Standards - Command Line Interfaces

---

## PRP Metadata

- **Author**: Claude
- **Created**: 2025-01-25
- **Status**: Ready for Implementation
- **Confidence Level**: 9/10 - Clear patterns to follow with clap