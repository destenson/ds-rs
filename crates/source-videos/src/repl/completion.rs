use rustyline::completion::{Completer as RustylineCompleter, FilenameCompleter, Pair};
use rustyline::{Context, Result};
use std::collections::HashMap;

pub struct ReplCompleter {
    commands: Vec<String>,
    filename_completer: FilenameCompleter,
}

impl ReplCompleter {
    pub fn new() -> Self {
        let commands = vec![
            // Source management
            "add".to_string(),
            "remove".to_string(),
            "list".to_string(),
            "sources".to_string(),
            "modify".to_string(),
            "enable".to_string(),
            "disable".to_string(),
            "inspect".to_string(),
            
            // Network commands
            "network".to_string(),
            "net".to_string(),
            
            // Server control
            "serve".to_string(),
            "stop".to_string(),
            "status".to_string(),
            
            // Monitoring
            "metrics".to_string(),
            "watch".to_string(),
            "health".to_string(),
            
            // Configuration
            "config".to_string(),
            "set".to_string(),
            "get".to_string(),
            
            // Information
            "help".to_string(),
            "?".to_string(),
            "patterns".to_string(),
            "examples".to_string(),
            
            // Scripting
            "run".to_string(),
            "record".to_string(),
            
            // Built-in commands
            "quit".to_string(),
            "exit".to_string(),
            "clear".to_string(),
            "cls".to_string(),
            "history".to_string(),
            "verbose".to_string(),
        ];

        Self {
            commands,
            filename_completer: FilenameCompleter::new(),
        }
    }

    fn complete_command(&self, line: &str, pos: usize, ctx: &Context<'_>) -> (usize, Vec<Pair>) {
        let words: Vec<&str> = line[..pos].split_whitespace().collect();
        
        if words.is_empty() || (words.len() == 1 && !line[..pos].ends_with(' ')) {
            // Complete command name
            let prefix = words.get(0).unwrap_or(&"");
            let matches: Vec<Pair> = self.commands
                .iter()
                .filter(|cmd| cmd.starts_with(prefix))
                .map(|cmd| Pair {
                    display: cmd.clone(),
                    replacement: cmd.clone(),
                })
                .collect();
            
            (pos - prefix.len(), matches)
        } else if words.len() >= 1 {
            // Complete subcommands and parameters
            let command = words[0];
            match command {
                "add" => self.complete_add_command(&words, line, pos, ctx),
                "remove" | "inspect" | "enable" | "disable" | "modify" => {
                    self.complete_source_id(&words, line, pos)
                }
                "network" | "net" => self.complete_network_command(&words, line, pos),
                "config" => self.complete_config_command(&words, line, pos),
                "help" | "?" => self.complete_help_command(&words, line, pos),
                "run" => self.filename_completer.complete(line, pos, ctx).unwrap_or((pos, vec![])),
                _ => (pos, vec![]),
            }
        } else {
            (pos, vec![])
        }
    }

    fn complete_add_command(&self, words: &[&str], line: &str, pos: usize, ctx: &Context<'_>) -> (usize, Vec<Pair>) {
        if words.len() == 2 && !line.ends_with(' ') {
            // Complete source type
            let prefix = words.get(1).unwrap_or(&"");
            let types = vec!["pattern", "directory", "file"];
            let matches: Vec<Pair> = types
                .iter()
                .filter(|t| t.starts_with(prefix))
                .map(|t| Pair {
                    display: t.to_string(),
                    replacement: t.to_string(),
                })
                .collect();
            (pos - prefix.len(), matches)
        } else if words.len() >= 3 {
            match words[1] {
                "pattern" => self.complete_pattern_name(&words, line, pos),
                "directory" | "file" => self.filename_completer.complete(line, pos, ctx).unwrap_or((pos, vec![])),
                _ => (pos, vec![]),
            }
        } else {
            (pos, vec![])
        }
    }

    fn complete_pattern_name(&self, words: &[&str], line: &str, pos: usize) -> (usize, Vec<Pair>) {
        if words.len() == 3 && !line.ends_with(' ') {
            let prefix = words.get(2).unwrap_or(&"");
            let patterns = vec![
                "smpte", "ball", "snow", "black", "white", "red", "green", "blue",
                "checkers-1", "checkers-2", "checkers-4", "checkers-8",
                "circular", "blink", "smpte75", "zone-plate", "gamut", "chroma-zone-plate",
                "solid-color", "gradient", "mandelbrot", "spokes", "color-bars",
            ];
            let matches: Vec<Pair> = patterns
                .iter()
                .filter(|p| p.starts_with(prefix))
                .map(|p| Pair {
                    display: p.to_string(),
                    replacement: p.to_string(),
                })
                .collect();
            (pos - prefix.len(), matches)
        } else {
            (pos, vec![])
        }
    }

    fn complete_source_id(&self, words: &[&str], line: &str, pos: usize) -> (usize, Vec<Pair>) {
        if words.len() == 2 && !line.ends_with(' ') {
            // In a real implementation, we would fetch actual source IDs
            let prefix = words.get(1).unwrap_or(&"");
            let source_ids = vec!["source-1", "source-2", "pattern-1", "test-pattern"];
            let matches: Vec<Pair> = source_ids
                .iter()
                .filter(|id| id.starts_with(prefix))
                .map(|id| Pair {
                    display: id.to_string(),
                    replacement: id.to_string(),
                })
                .collect();
            (pos - prefix.len(), matches)
        } else {
            (pos, vec![])
        }
    }

    fn complete_network_command(&self, words: &[&str], line: &str, pos: usize) -> (usize, Vec<Pair>) {
        if words.len() == 2 && !line.ends_with(' ') {
            let prefix = words.get(1).unwrap_or(&"");
            let subcommands = vec!["show", "profile", "set", "reset", "test"];
            let matches: Vec<Pair> = subcommands
                .iter()
                .filter(|cmd| cmd.starts_with(prefix))
                .map(|cmd| Pair {
                    display: cmd.to_string(),
                    replacement: cmd.to_string(),
                })
                .collect();
            (pos - prefix.len(), matches)
        } else if words.len() == 3 && words[1] == "profile" && !line.ends_with(' ') {
            let prefix = words.get(2).unwrap_or(&"");
            let profiles = vec!["perfect", "3g", "4g", "5g", "wifi", "public", "satellite", "broadband", "poor"];
            let matches: Vec<Pair> = profiles
                .iter()
                .filter(|p| p.starts_with(prefix))
                .map(|p| Pair {
                    display: p.to_string(),
                    replacement: p.to_string(),
                })
                .collect();
            (pos - prefix.len(), matches)
        } else if words.len() == 3 && words[1] == "set" && !line.ends_with(' ') {
            let prefix = words.get(2).unwrap_or(&"");
            let params = vec!["latency", "jitter", "packet_loss", "bandwidth"];
            let matches: Vec<Pair> = params
                .iter()
                .filter(|p| p.starts_with(prefix))
                .map(|p| Pair {
                    display: p.to_string(),
                    replacement: p.to_string(),
                })
                .collect();
            (pos - prefix.len(), matches)
        } else {
            (pos, vec![])
        }
    }

    fn complete_config_command(&self, words: &[&str], line: &str, pos: usize) -> (usize, Vec<Pair>) {
        if words.len() == 2 && !line.ends_with(' ') {
            let prefix = words.get(1).unwrap_or(&"");
            let subcommands = vec!["load", "save", "show", "set", "validate", "export"];
            let matches: Vec<Pair> = subcommands
                .iter()
                .filter(|cmd| cmd.starts_with(prefix))
                .map(|cmd| Pair {
                    display: cmd.to_string(),
                    replacement: cmd.to_string(),
                })
                .collect();
            (pos - prefix.len(), matches)
        } else {
            (pos, vec![])
        }
    }

    fn complete_help_command(&self, words: &[&str], line: &str, pos: usize) -> (usize, Vec<Pair>) {
        if words.len() == 2 && !line.ends_with(' ') {
            let prefix = words.get(1).unwrap_or(&"");
            let matches: Vec<Pair> = self.commands
                .iter()
                .filter(|cmd| cmd.starts_with(prefix))
                .map(|cmd| Pair {
                    display: cmd.clone(),
                    replacement: cmd.clone(),
                })
                .collect();
            (pos - prefix.len(), matches)
        } else {
            (pos, vec![])
        }
    }
}

impl RustylineCompleter for ReplCompleter {
    type Candidate = Pair;

    fn complete(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Result<(usize, Vec<Pair>)> {
        Ok(self.complete_command(line, pos, ctx))
    }
}