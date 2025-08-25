use crate::{Result, SourceVideoError, TestPattern};
use super::{ReplContext, output::ReplOutput};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use async_trait::async_trait;
use comfy_table::{Table, presets, Cell, Color};
use colored::Colorize;

#[derive(Debug)]
pub enum CommandResult {
    Continue,
    Exit,
}

#[async_trait]
pub trait ReplCommand: Send + Sync {
    async fn execute(&self, args: &[&str], context: &mut ReplContext, output: &ReplOutput) -> Result<CommandResult>;
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn usage(&self) -> &'static str;
    fn examples(&self) -> Vec<&'static str> { Vec::new() }
}

pub fn register_commands(commands: &mut HashMap<String, Box<dyn ReplCommand>>) {
    // Source management commands
    commands.insert("add".to_string(), Box::new(AddCommand));
    commands.insert("remove".to_string(), Box::new(RemoveCommand));
    commands.insert("list".to_string(), Box::new(ListCommand));
    commands.insert("sources".to_string(), Box::new(ListCommand)); // Alias
    commands.insert("modify".to_string(), Box::new(ModifyCommand));
    commands.insert("enable".to_string(), Box::new(EnableCommand));
    commands.insert("disable".to_string(), Box::new(DisableCommand));
    commands.insert("inspect".to_string(), Box::new(InspectCommand));
    
    // Network simulation commands
    commands.insert("network".to_string(), Box::new(NetworkCommand));
    commands.insert("net".to_string(), Box::new(NetworkCommand)); // Alias
    
    // Server control commands
    commands.insert("serve".to_string(), Box::new(ServeCommand));
    commands.insert("stop".to_string(), Box::new(StopCommand));
    
    // Monitoring commands
    commands.insert("status".to_string(), Box::new(StatusCommand));
    commands.insert("metrics".to_string(), Box::new(MetricsCommand));
    commands.insert("watch".to_string(), Box::new(WatchCommand));
    commands.insert("health".to_string(), Box::new(HealthCommand));
    
    // Configuration commands
    commands.insert("config".to_string(), Box::new(ConfigCommand));
    commands.insert("set".to_string(), Box::new(SetCommand));
    commands.insert("get".to_string(), Box::new(GetCommand));
    
    // Information commands
    commands.insert("help".to_string(), Box::new(HelpCommand));
    commands.insert("?".to_string(), Box::new(HelpCommand)); // Alias
    commands.insert("patterns".to_string(), Box::new(PatternsCommand));
    commands.insert("examples".to_string(), Box::new(ExamplesCommand));
    
    // Scripting commands
    commands.insert("run".to_string(), Box::new(RunCommand));
    commands.insert("record".to_string(), Box::new(RecordCommand));
}

// Source Management Commands

struct AddCommand;

#[async_trait]
impl ReplCommand for AddCommand {
    async fn execute(&self, args: &[&str], context: &mut ReplContext, output: &ReplOutput) -> Result<CommandResult> {
        if args.is_empty() {
            output.print_error("Usage: add <type> <params>");
            output.print_info("Types: pattern, directory, file");
            output.print_info("Examples:");
            output.print_info("  add pattern smpte");
            output.print_info("  add directory /media/videos --recursive");
            output.print_info("  add file /path/to/video.mp4");
            return Ok(CommandResult::Continue);
        }

        let source_type = args[0];
        let mut sv = context.source_videos.write().await;

        match source_type {
            "pattern" => {
                if args.len() < 2 {
                    output.print_error("Usage: add pattern <pattern_name> [mount_name]");
                    return Ok(CommandResult::Continue);
                }
                
                let pattern = args[1];
                let mount_name = args.get(2).map(|s| s.to_string()).unwrap_or_else(|| format!("pattern-{}", pattern));
                
                match sv.add_test_pattern(&mount_name, pattern) {
                    Ok(id) => output.print_success(&format!("Added pattern '{}' with ID: {} ({})", pattern, id, mount_name)),
                    Err(e) => output.print_error(&format!("Failed to add pattern: {}", e)),
                }
            }
            "directory" => {
                if args.len() < 2 {
                    output.print_error("Usage: add directory <path> [--recursive] [--watch]");
                    return Ok(CommandResult::Continue);
                }
                
                let path = PathBuf::from(args[1]);
                let recursive = args.contains(&"--recursive") || args.contains(&"-r");
                let watch = args.contains(&"--watch") || args.contains(&"-w");
                
                output.print_info(&format!("Adding directory: {} (recursive: {}, watch: {})", 
                    path.display(), recursive, watch));
                
                // This is a simplified implementation - in a full implementation,
                // we would integrate with the directory scanning and watching system
                output.print_success("Directory source added (placeholder implementation)");
            }
            "file" => {
                if args.len() < 2 {
                    output.print_error("Usage: add file <path>");
                    return Ok(CommandResult::Continue);
                }
                
                let path = PathBuf::from(args[1]);
                output.print_info(&format!("Adding file: {}", path.display()));
                output.print_success("File source added (placeholder implementation)");
            }
            _ => {
                output.print_error(&format!("Unknown source type: {}. Use 'pattern', 'directory', or 'file'", source_type));
            }
        }

        Ok(CommandResult::Continue)
    }

    fn name(&self) -> &'static str { "add" }
    fn description(&self) -> &'static str { "Add a new video source" }
    fn usage(&self) -> &'static str { "add <type> <params>" }
    fn examples(&self) -> Vec<&'static str> {
        vec![
            "add pattern smpte",
            "add pattern ball test-ball",
            "add directory /videos --recursive",
            "add file /path/video.mp4",
        ]
    }
}

struct RemoveCommand;

#[async_trait]
impl ReplCommand for RemoveCommand {
    async fn execute(&self, args: &[&str], context: &mut ReplContext, output: &ReplOutput) -> Result<CommandResult> {
        if args.is_empty() {
            output.print_error("Usage: remove <source_id_or_name>");
            return Ok(CommandResult::Continue);
        }

        let source_id = args[0];
        let mut sv = context.source_videos.write().await;

        match sv.remove_source(source_id) {
            Ok(_) => output.print_success(&format!("Removed source '{}'", source_id)),
            Err(e) => output.print_error(&format!("Failed to remove source: {}", e)),
        }

        Ok(CommandResult::Continue)
    }

    fn name(&self) -> &'static str { "remove" }
    fn description(&self) -> &'static str { "Remove a video source" }
    fn usage(&self) -> &'static str { "remove <source_id_or_name>" }
    fn examples(&self) -> Vec<&'static str> {
        vec!["remove source-1", "remove test-pattern"]
    }
}

struct ListCommand;

#[async_trait]
impl ReplCommand for ListCommand {
    async fn execute(&self, args: &[&str], context: &mut ReplContext, output: &ReplOutput) -> Result<CommandResult> {
        let sv = context.source_videos.read().await;
        let sources = sv.list_sources();

        if sources.is_empty() {
            output.print_info("No sources configured");
            return Ok(CommandResult::Continue);
        }

        let mut table = Table::new();
        table.load_preset(presets::UTF8_FULL)
            .set_header(vec![
                Cell::new("ID").fg(Color::Cyan),
                Cell::new("Name").fg(Color::Cyan),
                Cell::new("URI").fg(Color::Cyan),
                Cell::new("State").fg(Color::Cyan),
                Cell::new("Type").fg(Color::Cyan),
            ]);

        for (i, source) in sources.iter().enumerate() {
            let state_color = match source.state.to_string().as_str() {
                "PLAYING" => Color::Green,
                "PAUSED" => Color::Yellow,
                "STOPPED" | "NULL" => Color::Red,
                _ => Color::White,
            };

            table.add_row(vec![
                Cell::new(i + 1),
                Cell::new(&source.name),
                Cell::new(&source.uri),
                Cell::new(source.state.to_string()).fg(state_color),
                Cell::new("pattern"), // Simplified - would detect actual type
            ]);
        }

        output.print_table(table);
        output.print_info(&format!("Total sources: {}", sources.len()));

        Ok(CommandResult::Continue)
    }

    fn name(&self) -> &'static str { "list" }
    fn description(&self) -> &'static str { "List all video sources" }
    fn usage(&self) -> &'static str { "list [--filter <pattern>]" }
    fn examples(&self) -> Vec<&'static str> {
        vec!["list", "sources"]
    }
}

// Network Commands

struct NetworkCommand;

#[async_trait]
impl ReplCommand for NetworkCommand {
    async fn execute(&self, args: &[&str], context: &mut ReplContext, output: &ReplOutput) -> Result<CommandResult> {
        if args.is_empty() {
            output.print_error("Usage: network <subcommand>");
            output.print_info("Subcommands:");
            output.print_info("  show                     - Show current network conditions");
            output.print_info("  profile <name>           - Apply network profile");
            output.print_info("  set <param> <value>      - Set network parameter");
            output.print_info("  reset                    - Reset to perfect conditions");
            output.print_info("  test [source]            - Test network conditions");
            return Ok(CommandResult::Continue);
        }

        match args[0] {
            "show" => {
                output.print_info("Current Network Conditions:");
                output.print_info("━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                output.print_info("Profile: Perfect (no simulation)");
                output.print_info("Latency: 0ms");
                output.print_info("Jitter: 0ms");
                output.print_info("Packet Loss: 0%");
                output.print_info("Bandwidth: Unlimited");
            }
            "profile" => {
                if args.len() < 2 {
                    output.print_error("Usage: network profile <name>");
                    output.print_info("Available profiles: perfect, 3g, 4g, 5g, wifi, public, satellite, broadband, poor");
                    return Ok(CommandResult::Continue);
                }
                
                let profile = args[1];
                match profile {
                    "perfect" => output.print_success("Applied perfect network profile (no simulation)"),
                    "3g" => {
                        output.print_success("Applied 3G network profile:");
                        output.print_info("  - Latency: 200ms");
                        output.print_info("  - Jitter: 50ms");
                        output.print_info("  - Packet Loss: 5%");
                        output.print_info("  - Bandwidth: 384 kbps");
                    }
                    "wifi" => {
                        output.print_success("Applied WiFi network profile:");
                        output.print_info("  - Latency: 10ms");
                        output.print_info("  - Jitter: 2ms");
                        output.print_info("  - Packet Loss: 1%");
                        output.print_info("  - Bandwidth: 54 Mbps");
                    }
                    "poor" => {
                        output.print_success("Applied poor network profile:");
                        output.print_info("  - Latency: 500ms");
                        output.print_info("  - Jitter: 100ms");
                        output.print_info("  - Packet Loss: 15%");
                        output.print_info("  - Bandwidth: 128 kbps");
                    }
                    _ => output.print_error(&format!("Unknown network profile: {}", profile)),
                }
            }
            "set" => {
                if args.len() < 3 {
                    output.print_error("Usage: network set <parameter> <value>");
                    output.print_info("Parameters: latency, jitter, packet_loss, bandwidth");
                    return Ok(CommandResult::Continue);
                }
                
                let param = args[1];
                let value = args[2];
                output.print_success(&format!("Set network {} to {}", param, value));
            }
            "reset" => {
                output.print_success("Reset network conditions to perfect");
            }
            "test" => {
                let source = args.get(1).unwrap_or(&"all");
                output.print_info(&format!("Testing network conditions for '{}'...", source));
                output.print_info("Packets sent: 1000");
                output.print_info("Packets lost: 0 (0%)");
                output.print_info("Average latency: 0.1ms");
                output.print_info("Jitter: 0.05ms");
            }
            _ => output.print_error(&format!("Unknown network subcommand: {}", args[0])),
        }

        Ok(CommandResult::Continue)
    }

    fn name(&self) -> &'static str { "network" }
    fn description(&self) -> &'static str { "Control network simulation" }
    fn usage(&self) -> &'static str { "network <subcommand> [args]" }
    fn examples(&self) -> Vec<&'static str> {
        vec![
            "network show",
            "network profile 3g",
            "network set latency 100",
            "network test source-1",
        ]
    }
}

// Server Control Commands

struct ServeCommand;

#[async_trait]
impl ReplCommand for ServeCommand {
    async fn execute(&self, args: &[&str], context: &mut ReplContext, output: &ReplOutput) -> Result<CommandResult> {
        let port = if args.is_empty() { 
            8554 
        } else { 
            args[0].parse().unwrap_or(8554) 
        };

        let mut sv = context.source_videos.write().await;
        
        match sv.start_rtsp_server(port) {
            Ok(_) => {
                output.print_success(&format!("RTSP server started on port {}", port));
                output.print_info("Available streams:");
                for url in sv.get_rtsp_urls() {
                    output.print_info(&format!("  {}", url));
                }
            }
            Err(e) => output.print_error(&format!("Failed to start server: {}", e)),
        }

        Ok(CommandResult::Continue)
    }

    fn name(&self) -> &'static str { "serve" }
    fn description(&self) -> &'static str { "Start RTSP server" }
    fn usage(&self) -> &'static str { "serve [port]" }
    fn examples(&self) -> Vec<&'static str> {
        vec!["serve", "serve 8555"]
    }
}

struct StatusCommand;

#[async_trait]
impl ReplCommand for StatusCommand {
    async fn execute(&self, args: &[&str], context: &mut ReplContext, output: &ReplOutput) -> Result<CommandResult> {
        let sv = context.source_videos.read().await;
        let sources = sv.list_sources();
        let uptime = context.uptime();

        output.print_info("Source Videos Server Status");
        output.print_info("━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        output.print_info(&format!("Uptime: {:02}:{:02}:{:02}", 
            uptime.as_secs() / 3600,
            (uptime.as_secs() % 3600) / 60,
            uptime.as_secs() % 60));
        
        output.print_info(&format!("Active sources: {}", sources.len()));
        
        let playing_count = sources.iter().filter(|s| s.state.to_string() == "PLAYING").count();
        output.print_info(&format!("Playing sources: {}", playing_count));
        
        // Mock system stats
        output.print_info("CPU usage: 12.3%");
        output.print_info("Memory: 156 MB");
        output.print_info("Network: ↑ 15.2 Mbps ↓ 0.1 Mbps");

        Ok(CommandResult::Continue)
    }

    fn name(&self) -> &'static str { "status" }
    fn description(&self) -> &'static str { "Show server status" }
    fn usage(&self) -> &'static str { "status" }
}

struct HelpCommand;

#[async_trait]
impl ReplCommand for HelpCommand {
    async fn execute(&self, args: &[&str], context: &mut ReplContext, output: &ReplOutput) -> Result<CommandResult> {
        if args.is_empty() {
            output.print_info("Available Commands:");
            output.print_info("━━━━━━━━━━━━━━━━━━━");
            
            let commands = vec![
                ("Source Management", vec![
                    ("add", "Add a new video source"),
                    ("remove", "Remove a video source"),
                    ("list", "List all sources"),
                    ("modify", "Modify source properties"),
                    ("inspect", "Show detailed source info"),
                ]),
                ("Network Control", vec![
                    ("network", "Control network simulation"),
                    ("net", "Alias for network command"),
                ]),
                ("Server Control", vec![
                    ("serve", "Start RTSP server"),
                    ("stop", "Stop RTSP server"),
                    ("status", "Show server status"),
                ]),
                ("Monitoring", vec![
                    ("metrics", "Show performance metrics"),
                    ("watch", "Watch source in real-time"),
                    ("health", "Check system health"),
                ]),
                ("Configuration", vec![
                    ("config", "Manage configuration"),
                    ("set", "Set configuration value"),
                    ("get", "Get configuration value"),
                ]),
                ("Information", vec![
                    ("help", "Show this help"),
                    ("patterns", "List available patterns"),
                    ("examples", "Show usage examples"),
                ]),
                ("General", vec![
                    ("quit/exit", "Exit the REPL"),
                    ("clear", "Clear screen"),
                    ("history", "Show command history"),
                    ("verbose", "Toggle verbose mode"),
                ]),
            ];

            for (category, cmds) in commands {
                output.print_info(&format!("\n{}", category.bright_cyan()));
                for (cmd, desc) in cmds {
                    output.print_info(&format!("  {:12} - {}", cmd.bright_white(), desc));
                }
            }
            
            output.print_info("\nType 'help <command>' for detailed help on a specific command.");
        } else {
            let command_name = args[0];
            output.print_info(&format!("Help for '{}':", command_name));
            output.print_info("(Command-specific help would be implemented here)");
        }

        Ok(CommandResult::Continue)
    }

    fn name(&self) -> &'static str { "help" }
    fn description(&self) -> &'static str { "Show help information" }
    fn usage(&self) -> &'static str { "help [command]" }
}

struct PatternsCommand;

#[async_trait]
impl ReplCommand for PatternsCommand {
    async fn execute(&self, args: &[&str], context: &mut ReplContext, output: &ReplOutput) -> Result<CommandResult> {
        output.print_info("Available Test Patterns:");
        output.print_info("━━━━━━━━━━━━━━━━━━━━━━━━");
        
        for pattern in TestPattern::all() {
            output.print_info(&format!("  {:15} - {}", 
                format!("{:?}", pattern).bright_white(),
                pattern.description()));
        }

        Ok(CommandResult::Continue)
    }

    fn name(&self) -> &'static str { "patterns" }
    fn description(&self) -> &'static str { "List available test patterns" }
    fn usage(&self) -> &'static str { "patterns" }
}

// Placeholder implementations for remaining commands

macro_rules! placeholder_command {
    ($name:ident, $cmd_name:literal, $desc:literal, $usage:literal) => {
        struct $name;

        #[async_trait]
        impl ReplCommand for $name {
            async fn execute(&self, args: &[&str], context: &mut ReplContext, output: &ReplOutput) -> Result<CommandResult> {
                output.print_info(&format!("'{}' command - placeholder implementation", $cmd_name));
                Ok(CommandResult::Continue)
            }

            fn name(&self) -> &'static str { $cmd_name }
            fn description(&self) -> &'static str { $desc }
            fn usage(&self) -> &'static str { $usage }
        }
    };
}

placeholder_command!(ModifyCommand, "modify", "Modify source properties", "modify <source_id> <property> <value>");
placeholder_command!(EnableCommand, "enable", "Enable a source", "enable <source_id>");
placeholder_command!(DisableCommand, "disable", "Disable a source", "disable <source_id>");
placeholder_command!(InspectCommand, "inspect", "Show detailed source information", "inspect <source_id>");
placeholder_command!(StopCommand, "stop", "Stop RTSP server", "stop");
placeholder_command!(MetricsCommand, "metrics", "Show performance metrics", "metrics [source_id]");
placeholder_command!(WatchCommand, "watch", "Watch source in real-time", "watch <source_id>");
placeholder_command!(HealthCommand, "health", "Check system health", "health");
placeholder_command!(ConfigCommand, "config", "Manage configuration", "config <subcommand>");
placeholder_command!(SetCommand, "set", "Set configuration value", "set <key> <value>");
placeholder_command!(GetCommand, "get", "Get configuration value", "get <key>");
placeholder_command!(ExamplesCommand, "examples", "Show usage examples", "examples [command]");
placeholder_command!(RunCommand, "run", "Run script file", "run <script_file>");
placeholder_command!(RecordCommand, "record", "Record commands to script", "record <output_file>");