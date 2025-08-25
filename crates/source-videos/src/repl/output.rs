use colored::{Colorize, ColoredString};
use comfy_table::Table;
use super::ReplContext;

#[derive(Debug, Clone)]
pub enum OutputFormat {
    Text,
    Json,
    Csv,
}

pub struct ReplOutput {
    pub format: OutputFormat,
}

impl ReplOutput {
    pub fn new() -> Self {
        Self {
            format: OutputFormat::Text,
        }
    }

    pub fn print_welcome(&self, context: &ReplContext) {
        println!("{}", "Source Videos Enhanced REPL".bright_cyan().bold());
        println!("{}", "═══════════════════════════".cyan());
        println!();
        println!("Welcome to the enhanced interactive mode!");
        println!("Type '{}' to see all available commands.", "help".bright_white());
        println!("Type '{}' for usage examples.", "examples".bright_white());
        println!("Use {} or {} for auto-completion.", "TAB".bright_yellow(), "↑/↓".bright_yellow());
        println!();
        
        if context.verbose {
            println!("Verbose mode: {}", "ON".bright_green());
            println!("Session started at: {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
        }
        
        println!("Ready for commands!");
        println!();
    }

    pub fn print_success(&self, message: &str) {
        match self.format {
            OutputFormat::Text => {
                println!("{} {}", "✓".bright_green(), message);
            }
            OutputFormat::Json => {
                println!(r#"{{"status": "success", "message": "{}"}}"#, message);
            }
            OutputFormat::Csv => {
                println!("success,{}", message);
            }
        }
    }

    pub fn print_error(&self, message: &str) {
        match self.format {
            OutputFormat::Text => {
                eprintln!("{} {}", "✗".bright_red(), message.bright_red());
            }
            OutputFormat::Json => {
                eprintln!(r#"{{"status": "error", "message": "{}"}}"#, message);
            }
            OutputFormat::Csv => {
                eprintln!("error,{}", message);
            }
        }
    }

    pub fn print_warning(&self, message: &str) {
        match self.format {
            OutputFormat::Text => {
                println!("{} {}", "⚠".bright_yellow(), message.bright_yellow());
            }
            OutputFormat::Json => {
                println!(r#"{{"status": "warning", "message": "{}"}}"#, message);
            }
            OutputFormat::Csv => {
                println!("warning,{}", message);
            }
        }
    }

    pub fn print_info(&self, message: &str) {
        match self.format {
            OutputFormat::Text => {
                println!("{}", message);
            }
            OutputFormat::Json => {
                println!(r#"{{"status": "info", "message": "{}"}}"#, message);
            }
            OutputFormat::Csv => {
                println!("info,{}", message);
            }
        }
    }

    pub fn print_debug(&self, message: &str) {
        match self.format {
            OutputFormat::Text => {
                println!("{} {}", "🐛".bright_blue(), message.bright_black());
            }
            OutputFormat::Json => {
                println!(r#"{{"status": "debug", "message": "{}"}}"#, message);
            }
            OutputFormat::Csv => {
                println!("debug,{}", message);
            }
        }
    }

    pub fn print_table(&self, table: Table) {
        match self.format {
            OutputFormat::Text => {
                println!("{}", table);
            }
            OutputFormat::Json => {
                // For JSON output, we would need to convert the table to JSON
                // This is a simplified implementation
                println!(r#"{{"type": "table", "data": "Table output not yet supported in JSON format"}}"#);
            }
            OutputFormat::Csv => {
                println!("table,Table output not yet supported in CSV format");
            }
        }
    }

    pub fn print_separator(&self) {
        match self.format {
            OutputFormat::Text => {
                println!("{}", "─".repeat(50).bright_black());
            }
            OutputFormat::Json => {
                println!(r#"{{"type": "separator"}}"#);
            }
            OutputFormat::Csv => {
                println!("separator,");
            }
        }
    }

    pub fn print_header(&self, title: &str) {
        match self.format {
            OutputFormat::Text => {
                println!();
                println!("{}", title.bright_cyan().bold());
                println!("{}", "═".repeat(title.len()).cyan());
            }
            OutputFormat::Json => {
                println!(r#"{{"type": "header", "title": "{}"}}"#, title);
            }
            OutputFormat::Csv => {
                println!("header,{}", title);
            }
        }
    }

    pub fn print_progress(&self, message: &str, current: usize, total: usize) {
        match self.format {
            OutputFormat::Text => {
                let percent = if total > 0 { (current * 100) / total } else { 0 };
                let progress_bar = self.create_progress_bar(percent);
                println!("{} {} [{}/{}]", progress_bar, message, current, total);
            }
            OutputFormat::Json => {
                println!(r#"{{"type": "progress", "message": "{}", "current": {}, "total": {}, "percent": {}}}"#, 
                    message, current, total, if total > 0 { (current * 100) / total } else { 0 });
            }
            OutputFormat::Csv => {
                println!("progress,{},{},{}", message, current, total);
            }
        }
    }

    fn create_progress_bar(&self, percent: usize) -> ColoredString {
        let width = 20;
        let filled = (percent * width) / 100;
        let empty = width - filled;
        
        let bar = format!("[{}{}]", 
            "█".repeat(filled).bright_green(),
            "░".repeat(empty).bright_black());
        
        format!("{:3}% {}", percent, bar).normal()
    }

    pub fn print_key_value(&self, key: &str, value: &str) {
        match self.format {
            OutputFormat::Text => {
                println!("{}: {}", key.bright_white(), value);
            }
            OutputFormat::Json => {
                println!(r#"{{"{}", "{}"}}"#, key, value);
            }
            OutputFormat::Csv => {
                println!("{},{}", key, value);
            }
        }
    }

    pub fn print_list(&self, title: &str, items: Vec<&str>) {
        match self.format {
            OutputFormat::Text => {
                println!("{}", title.bright_cyan());
                for item in items {
                    println!("  • {}", item);
                }
            }
            OutputFormat::Json => {
                let items_json: Vec<String> = items.iter().map(|s| format!(r#""{}""#, s)).collect();
                println!(r#"{{"type": "list", "title": "{}", "items": [{}]}}"#, title, items_json.join(","));
            }
            OutputFormat::Csv => {
                for item in items {
                    println!("{},{}", title, item);
                }
            }
        }
    }

    pub fn print_metrics(&self, metrics: &[(&str, String)]) {
        match self.format {
            OutputFormat::Text => {
                for (name, value) in metrics {
                    println!("{:20}: {}", name.bright_white(), value);
                }
            }
            OutputFormat::Json => {
                let metrics_json: Vec<String> = metrics.iter()
                    .map(|(k, v)| format!(r#""{}": "{}""#, k, v))
                    .collect();
                println!(r#"{{"type": "metrics", "data": {{{}}}}}"#, metrics_json.join(","));
            }
            OutputFormat::Csv => {
                for (name, value) in metrics {
                    println!("{},{}", name, value);
                }
            }
        }
    }

    pub fn set_format(&mut self, format: OutputFormat) {
        self.format = format;
    }
}