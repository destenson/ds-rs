# PRP: REPL Mode Enhancements for Source-Videos

**Status**: COMPLETED (2025-08-27) - REPL fully implemented with completion

## Executive Summary

Enhance the interactive REPL (Read-Eval-Print Loop) mode with comprehensive commands for dynamic source management, real-time network simulation control, status monitoring, and debugging capabilities. Transform the basic interactive mode into a powerful runtime control interface.

## Problem Statement

### Current State
- Basic interactive mode with limited commands
- No dynamic source management in REPL
- Cannot modify network conditions at runtime
- Limited status and monitoring capabilities
- No command history or completion

### Desired State
- Full-featured REPL with all runtime controls
- Dynamic source addition/removal/modification
- Real-time network simulation adjustments
- Comprehensive monitoring and debugging
- Command history, completion, and scripting
- Multi-line command support
- Export/import of configurations

### Business Value
Provides powerful runtime control for testing and debugging, enables interactive experimentation with network conditions, supports live demonstrations, and improves troubleshooting capabilities.

## Requirements

### Functional Requirements

1. **Source Management**: Add, remove, list, modify sources
2. **Network Control**: Apply/modify network conditions
3. **Status Monitoring**: View stream status and metrics
4. **Configuration**: Load, save, modify configurations
5. **Debugging**: Enable/disable debug output, inspect internals
6. **History**: Command history and recall
7. **Scripting**: Execute command scripts
8. **Help System**: Context-aware help

### Non-Functional Requirements

1. **Responsiveness**: Commands execute immediately
2. **Usability**: Intuitive command syntax
3. **Robustness**: Graceful error handling
4. **Performance**: Minimal overhead
5. **Portability**: Works on all platforms

### Context and Research

Current interactive mode in `src/main.rs` is basic with simple command parsing.

Consider using:
- rustyline for readline functionality
- reedline (from nushell) for advanced REPL
- Built-in completion and history

### Documentation & References

```yaml
- file: crates/source-videos/src/main.rs
  why: Current interactive_command implementation

- url: https://docs.rs/rustyline/latest/rustyline/
  why: Readline library for better REPL

- url: https://docs.rs/reedline/latest/reedline/
  why: Modern REPL library option

- file: crates/ds-rs/examples/
  why: Reference for interactive patterns

- url: https://docs.rs/comfy-table/latest/comfy_table/
  why: Table formatting for status display
```

### List of tasks to be completed

```yaml
Task 1:
CREATE src/repl/mod.rs:
  - DEFINE ReplContext with state
  - IMPLEMENT command parser
  - ADD command registry system
  - CREATE output formatting utilities
  - INCLUDE error handling

Task 2:
CREATE src/repl/commands.rs:
  - DEFINE Command trait
  - IMPLEMENT source commands (add, remove, list, modify)
  - ADD network commands (sim, conditions, profiles)
  - CREATE status commands (streams, metrics, health)
  - ADD config commands (load, save, show)
  - IMPLEMENT control commands (pause, resume, restart)

Task 3:
CREATE src/repl/readline.rs:
  - INTEGRATE rustyline or reedline
  - IMPLEMENT command completion
  - ADD history management
  - CREATE custom key bindings
  - SUPPORT multi-line input

Task 4:
IMPLEMENT source management commands:
  - ADD "add source <type> <params>" command
  - CREATE "remove source <id>" command
  - ADD "list sources" with filtering
  - IMPLEMENT "modify source <id> <params>"
  - ADD "enable/disable source <id>"
  - CREATE "inspect source <id>" for details

Task 5:
ADD network simulation commands:
  - IMPLEMENT "network profile <name>" command
  - ADD "network set <param> <value>" for custom
  - CREATE "network show" for current conditions
  - ADD "network reset" to clear simulation
  - IMPLEMENT "network test <source>" for specific source
  - ADD "network scenario <script>" for sequences

Task 6:
CREATE monitoring commands:
  - ADD "status" for overall system status
  - IMPLEMENT "metrics [source]" for performance
  - ADD "watch <source>" for live updates
  - CREATE "log level <level>" for debug control
  - ADD "stats" for cumulative statistics
  - IMPLEMENT "health" for system health check

Task 7:
ADD configuration commands:
  - IMPLEMENT "config load <file>"
  - ADD "config save <file>"
  - CREATE "config show [section]"
  - ADD "config set <key> <value>"
  - IMPLEMENT "config validate"
  - ADD "config export" for current state

Task 8:
CREATE scripting support:
  - ADD "run <script>" command
  - IMPLEMENT script file format
  - ADD variables and conditionals
  - CREATE loops for repeated commands
  - SUPPORT comments in scripts
  - ADD "record" mode to create scripts

Task 9:
ENHANCE help system:
  - ADD "help [command]" with details
  - CREATE "?" as help shortcut
  - ADD command suggestions on error
  - IMPLEMENT "examples" command
  - CREATE interactive tutorial mode

Task 10:
ADD output formatting:
  - IMPLEMENT table output for lists
  - ADD JSON output option
  - CREATE colored output for terminals
  - ADD progress indicators for long operations
  - IMPLEMENT quiet/verbose modes
```

### Out of Scope
- GUI interface
- Web-based console
- Remote REPL access
- Plugin system for custom commands

## Success Criteria

- [ ] All major features controllable from REPL
- [ ] Command completion works for all commands
- [ ] History persists between sessions
- [ ] Network conditions can be changed in real-time
- [ ] Status displays are clear and informative
- [ ] Scripts can automate common tasks
- [ ] Help is comprehensive and accessible
- [ ] Error messages are helpful

## Dependencies

### Technical Dependencies
- rustyline or reedline for readline functionality
- comfy-table for formatted output
- colored for terminal colors
- serde for configuration serialization

### Knowledge Dependencies
- REPL design patterns
- Terminal control sequences
- Command parsing strategies

## Risks and Mitigation

| Risk | Probability | Impact | Mitigation Strategy |
|------|------------|--------|-------------------|
| Platform compatibility | Low | Medium | Use cross-platform libraries |
| Command complexity | Medium | Medium | Progressive disclosure, good help |
| Performance impact | Low | Low | Async command execution |
| State consistency | Medium | Medium | Proper state management |

## Architecture Decisions

### Decision: REPL Library
**Options Considered:**
1. Custom implementation
2. rustyline
3. reedline

**Decision:** Option 2 - rustyline

**Rationale:** Mature, well-tested, good cross-platform support

### Decision: Command Structure
**Options Considered:**
1. Simple string parsing
2. Structured command objects
3. DSL with parser

**Decision:** Option 2 - Structured commands

**Rationale:** Type-safe, extensible, good for completion

## Validation Strategy

### Validation Commands
```bash
# Test REPL mode
cargo run -- interactive

# In REPL:
> add source directory /videos --recursive
> list sources
> network profile lossy
> status
> metrics
> help add
> run test_script.repl

# Test command completion
> add so<TAB>  # Should complete to "add source"

# Test history
> <UP>  # Should show previous command
```

## REPL Command Examples

### Source Management
```
> add source directory /media/videos --recursive --watch
Source added: source-1 (23 files found)

> add source pattern smpte --mount test1
Source added: source-2 (test pattern: smpte)

> list sources
┌──────────┬──────────┬─────────────────┬────────┬─────────┐
│ ID       │ Type     │ Path/Pattern    │ Status │ Streams │
├──────────┼──────────┼─────────────────┼────────┼─────────┤
│ source-1 │ directory│ /media/videos   │ active │ 23      │
│ source-2 │ pattern  │ smpte           │ active │ 1       │
└──────────┴──────────┴─────────────────┴────────┴─────────┘

> remove source source-1
Source removed: source-1
```

### Network Simulation
```
> network show
Current network conditions: Perfect (no simulation)

> network profile residential
Applied network profile: residential
  - Latency: 20ms
  - Jitter: 5ms
  - Packet loss: 0.1%

> network set packet_loss 5
Network packet loss set to 5%

> network test source-2
Testing network conditions on source-2...
  Packets sent: 1000
  Packets lost: 51 (5.1%)
  Average latency: 20.3ms
```

### Monitoring
```
> status
Source Videos Server Status
━━━━━━━━━━━━━━━━━━━━━━━━━
Uptime: 00:15:42
Active sources: 2
Total streams: 24
CPU usage: 12.3%
Memory: 156 MB
Network: ↑ 15.2 Mbps ↓ 0.1 Mbps

> watch source-2
Watching source-2... (Ctrl-C to stop)
[00:00:01] FPS: 30.0 | Bitrate: 2.5 Mbps | Clients: 3
[00:00:02] FPS: 29.9 | Bitrate: 2.4 Mbps | Clients: 3
```

## Future Considerations

- Web-based terminal interface
- Collaborative REPL sessions
- Record and replay of REPL sessions
- Integration with testing frameworks
- Custom command plugins

## References

- Rustyline documentation: https://docs.rs/rustyline/
- REPL design best practices
- Terminal UI guidelines

---

## PRP Metadata

- **Author**: Claude
- **Created**: 2025-01-25
- **Status**: Ready for Implementation
- **Confidence Level**: 8/10 - Well-established patterns with rustyline
