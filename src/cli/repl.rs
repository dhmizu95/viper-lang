use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use crate::repl::{InputState, LineStatus, ReplSession};

pub fn run_repl() -> Result<(), String> {
    println!("🐍 Viper REPL {}", env!("CARGO_PKG_VERSION"));
    println!("Type :quit to exit, :clear to clear, :help for commands\n");

    let mut editor = DefaultEditor::new().map_err(|e| e.to_string())?;

    // We intentionally don't load history file here as Viper doesn't have a reliable
    // home directory expansion built-in, but rustyline keeps session history anyway.

    let mut input_state = InputState::new();
    let mut session = ReplSession::new();
    let mut prompt = ">>> ";

    loop {
        let readline = editor.readline(prompt);
        match readline {
            Ok(line) => {
                let trimmed = line.trim();

                // Handle REPL commands when not in a block
                if prompt == ">>> " && trimmed.starts_with(':') {
                    editor.add_history_entry(line.as_str()).unwrap();
                    match handle_command(trimmed, &mut session, &mut input_state) {
                        CommandResult::Continue => continue,
                        CommandResult::Quit => break,
                    }
                }

                // Normal code execution
                if !trimmed.is_empty() {
                    editor.add_history_entry(line.as_str()).unwrap();
                }

                match input_state.feed(&line) {
                    LineStatus::Complete => {
                        let source = input_state.take_buffer();
                        execute_source(&mut session, &source);
                        prompt = ">>> ";
                    }
                    LineStatus::Incomplete => {
                        prompt = "... ";
                    }
                    LineStatus::Empty => {
                        let source = input_state.take_buffer();
                        execute_source(&mut session, &source);
                        prompt = ">>> ";
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl-C
                if prompt == "... " {
                    // Cancel block
                    println!("KeyboardInterrupt");
                    input_state.force_complete();
                    input_state.take_buffer();
                    prompt = ">>> ";
                } else {
                    println!("KeyboardInterrupt");
                }
            }
            Err(ReadlineError::Eof) => {
                // Ctrl-D
                println!("Goodbye!");
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }

    Ok(())
}

fn execute_source(session: &mut ReplSession, source: &str) {
    if let Err(e) = session.execute_chunk(source) {
        eprintln!("Error: {}", e);
    }
}

enum CommandResult {
    Continue,
    Quit,
}

fn handle_command(
    cmd: &str,
    session: &mut ReplSession,
    input_state: &mut InputState,
) -> CommandResult {
    match cmd {
        ":quit" | ":q" => CommandResult::Quit,
        ":clear" | ":c" => {
            print!("\x1B[2J\x1B[1J");
            println!("🐍 Viper REPL {}", env!("CARGO_PKG_VERSION"));
            println!("Type :quit to exit, :clear to clear, :help for commands\n");
            CommandResult::Continue
        }
        ":reset" => {
            session.reset();
            input_state.force_complete();
            input_state.take_buffer(); // clear buffer
            println!("Environment reset.");
            CommandResult::Continue
        }
        ":vars" => {
            let vars = session.vars_summary();
            if vars.is_empty() {
                println!("No variables defined.");
            } else {
                println!("Variables:");
                for var in vars {
                    println!("  {}", var);
                }
            }
            CommandResult::Continue
        }
        ":help" | ":h" => {
            println!("REPL Commands:");
            println!("  :quit,   :q    - Exit the REPL");
            println!("  :clear,  :c    - Clear the screen");
            println!("  :reset         - Clear all variables & state");
            println!("  :vars          - Show currently defined variables");
            println!("  :help,   :h    - Show this help message");
            println!();
            CommandResult::Continue
        }
        _ => {
            println!("Unknown command: {}", cmd);
            CommandResult::Continue
        }
    }
}
