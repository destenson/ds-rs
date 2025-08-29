use crate::{Result, SourceVideoError, SourceVideos};
use colored::Colorize;
use comfy_table::{Cell, Color, Table, presets};
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::{CmdKind, Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::{MatchingBracketValidator, Validator};
use rustyline::{Editor, Helper};
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

pub mod commands;
pub mod completion;
pub mod output;

use commands::{CommandResult, ReplCommand};
use completion::ReplCompleter;
use output::{OutputFormat, ReplOutput};

pub struct ReplHelper {
    completer: ReplCompleter,
    highlighter: MatchingBracketHighlighter,
    hinter: HistoryHinter,
    validator: MatchingBracketValidator,
}

impl ReplHelper {
    pub fn new() -> Self {
        Self {
            completer: ReplCompleter::new(),
            highlighter: MatchingBracketHighlighter::new(),
            hinter: HistoryHinter::new(),
            validator: MatchingBracketValidator::new(),
        }
    }
}

impl Completer for ReplHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        self.completer.complete(line, pos, ctx)
    }
}

impl Hinter for ReplHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &rustyline::Context<'_>) -> Option<String> {
        self.hinter.hint(line, pos, ctx)
    }
}

impl Highlighter for ReplHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        self.highlighter.highlight_prompt(prompt, default)
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        self.highlighter.highlight_hint(hint)
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize, kind: CmdKind) -> bool {
        self.highlighter.highlight_char(line, pos, kind)
    }
}

impl Validator for ReplHelper {
    fn validate(
        &self,
        ctx: &mut rustyline::validate::ValidationContext,
    ) -> rustyline::Result<rustyline::validate::ValidationResult> {
        self.validator.validate(ctx)
    }

    fn validate_while_typing(&self) -> bool {
        self.validator.validate_while_typing()
    }
}

impl Helper for ReplHelper {}

pub struct ReplContext {
    pub source_videos: Arc<RwLock<SourceVideos>>,
    pub output_format: OutputFormat,
    pub verbose: bool,
    pub start_time: Instant,
    pub command_history: Vec<String>,
    pub variables: HashMap<String, String>,
}

impl ReplContext {
    pub fn new(source_videos: SourceVideos) -> Self {
        Self {
            source_videos: Arc::new(RwLock::new(source_videos)),
            output_format: OutputFormat::Text,
            verbose: false,
            start_time: Instant::now(),
            command_history: Vec::new(),
            variables: HashMap::new(),
        }
    }

    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }
}

pub struct EnhancedRepl {
    editor: Editor<ReplHelper, rustyline::history::FileHistory>,
    context: ReplContext,
    commands: HashMap<String, Box<dyn ReplCommand>>,
    output: ReplOutput,
}

impl EnhancedRepl {
    pub fn new(source_videos: SourceVideos) -> Result<Self> {
        let mut editor = Editor::new()
            .map_err(|e| SourceVideoError::config(format!("Failed to create editor: {}", e)))?;
        editor.set_helper(Some(ReplHelper::new()));

        // Load history file if it exists
        let history_file = dirs::config_dir().map(|mut path| {
            path.push("source-videos");
            std::fs::create_dir_all(&path).ok();
            path.push("repl_history");
            path
        });

        if let Some(ref history_path) = history_file {
            let _ = editor.load_history(history_path);
        }

        let context = ReplContext::new(source_videos);
        let output = ReplOutput::new();
        let mut commands: HashMap<String, Box<dyn ReplCommand>> = HashMap::new();

        // Register all commands
        commands::register_commands(&mut commands);

        Ok(Self {
            editor,
            context,
            commands,
            output,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        self.output.print_welcome(&self.context);

        loop {
            let prompt = self.get_prompt();
            let readline = self.editor.readline(&prompt);
            match readline {
                Ok(line) => {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }

                    self.editor.add_history_entry(line).ok();
                    self.context.command_history.push(line.to_string());

                    match self.execute_command(line).await {
                        Ok(CommandResult::Continue) => continue,
                        Ok(CommandResult::Exit) => break,
                        Err(e) => {
                            self.output.print_error(&format!("Error: {}", e));
                        }
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    self.output.print_info("Use 'quit' or Ctrl-D to exit");
                }
                Err(ReadlineError::Eof) => {
                    self.output.print_info("Goodbye!");
                    break;
                }
                Err(err) => {
                    self.output
                        .print_error(&format!("Error reading line: {}", err));
                    break;
                }
            }
        }

        // Save history
        if let Some(history_path) = dirs::config_dir().map(|mut path| {
            path.push("source-videos");
            path.push("repl_history");
            path
        }) {
            let _ = self.editor.save_history(&history_path);
        }

        Ok(())
    }

    fn get_prompt(&self) -> String {
        if self.context.verbose {
            format!("[{}] > ", self.format_uptime())
        } else {
            "> ".to_string()
        }
    }

    fn format_uptime(&self) -> String {
        let duration = self.context.uptime();
        let hours = duration.as_secs() / 3600;
        let minutes = (duration.as_secs() % 3600) / 60;
        let seconds = duration.as_secs() % 60;
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }

    async fn execute_command(&mut self, line: &str) -> Result<CommandResult> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(CommandResult::Continue);
        }

        let command_name = parts[0];
        let args = &parts[1..];

        // Check for built-in commands first
        match command_name {
            "quit" | "exit" => return Ok(CommandResult::Exit),
            "clear" | "cls" => {
                print!("\x1B[2J\x1B[1;1H");
                return Ok(CommandResult::Continue);
            }
            "history" => {
                self.show_history();
                return Ok(CommandResult::Continue);
            }
            "verbose" => {
                self.context.verbose = !self.context.verbose;
                self.output.print_success(&format!(
                    "Verbose mode: {}",
                    if self.context.verbose { "ON" } else { "OFF" }
                ));
                return Ok(CommandResult::Continue);
            }
            _ => {}
        }

        // Look for registered commands
        if let Some(command) = self.commands.get(command_name) {
            command.execute(args, &mut self.context, &self.output).await
        } else {
            self.output.print_error(&format!(
                "Unknown command: '{}'. Type 'help' for available commands.",
                command_name
            ));
            self.suggest_similar_command(command_name);
            Ok(CommandResult::Continue)
        }
    }

    fn show_history(&self) {
        let mut table = Table::new();
        table
            .load_preset(presets::UTF8_FULL)
            .set_header(vec!["#", "Command"]);

        for (i, cmd) in self.context.command_history.iter().enumerate() {
            table.add_row(vec![Cell::new(i + 1), Cell::new(cmd)]);
        }

        self.output.print_table(table);
    }

    fn suggest_similar_command(&self, input: &str) {
        let mut suggestions = Vec::new();

        for cmd_name in self.commands.keys() {
            if cmd_name.starts_with(input) {
                suggestions.push(cmd_name.clone());
            }
        }

        if suggestions.is_empty() {
            // Try fuzzy matching
            for cmd_name in self.commands.keys() {
                if self.levenshtein_distance(input, cmd_name) <= 2 {
                    suggestions.push(cmd_name.clone());
                }
            }
        }

        if !suggestions.is_empty() {
            self.output
                .print_info(&format!("Did you mean: {}", suggestions.join(", ")));
        }
    }

    fn levenshtein_distance(&self, a: &str, b: &str) -> usize {
        let a_chars: Vec<char> = a.chars().collect();
        let b_chars: Vec<char> = b.chars().collect();
        let a_len = a_chars.len();
        let b_len = b_chars.len();

        if a_len == 0 {
            return b_len;
        }
        if b_len == 0 {
            return a_len;
        }

        let mut matrix = vec![vec![0; b_len + 1]; a_len + 1];

        for i in 0..=a_len {
            matrix[i][0] = i;
        }
        for j in 0..=b_len {
            matrix[0][j] = j;
        }

        for i in 1..=a_len {
            for j in 1..=b_len {
                let cost = if a_chars[i - 1] == b_chars[j - 1] {
                    0
                } else {
                    1
                };
                matrix[i][j] = (matrix[i - 1][j] + 1)
                    .min(matrix[i][j - 1] + 1)
                    .min(matrix[i - 1][j - 1] + cost);
            }
        }

        matrix[a_len][b_len]
    }
}
