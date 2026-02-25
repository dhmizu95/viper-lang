use std::io::{self, Write};

pub fn run_repl() -> Result<(), String> {
    println!("Viper REPL v0.2.3");
    println!("Type :quit to exit, :clear to clear\n");

    let mut history: Vec<String> = Vec::new();

    loop {
        print!(">>> ");
        io::stdout().flush().map_err(|e| e.to_string())?;

        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .map_err(|e| e.to_string())?;

        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        match line {
            ":quit" | ":q" => {
                println!("Goodbye!");
                break;
            }
            ":clear" => {
                print!("\x1B[2J\x1B[1J");
                println!("Viper REPL v0.2.3");
                println!("Type :quit to exit, :clear to clear\n");
                continue;
            }
            ":help" => {
                println!("REPL Commands:");
                println!("  :quit, :q   - Exit the REPL");
                println!("  :clear      - Clear the screen");
                println!("  :help       - Show this help");
                println!("  :history    - Show command history");
                println!();
                continue;
            }
            ":history" => {
                for (i, cmd) in history.iter().enumerate() {
                    println!("  {}: {}", i + 1, cmd);
                }
                continue;
            }
            _ => {}
        }

        history.push(line.to_string());

        // Try to evaluate the expression
        match eval_expression(line) {
            Ok(result) => {
                println!("{}", result);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }

    Ok(())
}

fn eval_expression(line: &str) -> Result<String, String> {
    // Try to parse and evaluate simple expressions
    let source = format!("def __repl__(): {}", line);

    let mut lexer = crate::lexer::Lexer::new(&source);
    let tokens = lexer.tokenize()?;

    let mut parser = crate::parser::Parser::new(tokens);
    let _ast = parser.parse()?;

    // For now, just echo back what we parsed
    Ok(format!("parsed: {}", line))
}
