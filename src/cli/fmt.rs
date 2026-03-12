use std::fs;
use std::io::Write;

pub fn run_fmt(args: &FmtArgs) -> crate::error::Result<()> {
    let source = fs::read_to_string(&args.input)
        .map_err(crate::error::ViperError::Io)?;

    // Parse to check for errors, then pretty-print
    let mut lexer = crate::lexer::Lexer::new(&source);
    let tokens = lexer.tokenize()?;

    let mut parser = crate::parser::Parser::new(tokens);
    let _ast = parser.parse()?;

    // For now, just output the source as-is (simple formatter)
    let formatted = source.clone();

    if let Some(ref output) = args.output {
        let mut file = fs::File::create(output)
            .map_err(crate::error::ViperError::Io)?;
        file.write_all(formatted.as_bytes()).map_err(crate::error::ViperError::Io)?;
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
