use std::fs;
use std::io::Write;

pub fn run_fmt(args: &FmtArgs) -> Result<(), String> {
    let source = fs::read_to_string(&args.input)
        .map_err(|e| format!("Failed to read '{}': {}", args.input, e))?;

    // Parse to check for errors, then pretty-print
    let mut lexer = crate::lexer::Lexer::new(&source);
    let tokens = lexer.tokenize()?;

    let mut parser = crate::parser::Parser::new(tokens);
    let _ast = parser.parse()?;

    // For now, just output the source as-is (simple formatter)
    let formatted = source.clone();

    if let Some(ref output) = args.output {
        let mut file = fs::File::create(output)
            .map_err(|e| format!("Failed to create '{}': {}", output, e))?;
        file.write_all(formatted.as_bytes()).map_err(|e| format!("Failed to write: {}", e))?;
    } else {
        println!("{}", formatted);
    }

    Ok(())
}

pub struct FmtArgs {
    pub input: String,
    pub output: Option<String>,
}

impl FmtArgs {
    pub fn new(input: String, output: Option<String>) -> Self {
        Self { input, output }
    }
}
