use std::fs;
use std::path::Path;

pub fn run_doc(args: &DocArgs) -> crate::error::Result<()> {
    let source = fs::read_to_string(&args.input).map_err(crate::error::ViperError::Io)?;

    let mut lexer = crate::lexer::Lexer::new(&source);
    let tokens = lexer.tokenize()?;

    let mut parser = crate::parser::Parser::new(tokens);
    let _module = parser.parse()?;

    // Extract docstrings using simple string search
    let docs = extract_docstrings_simple(&source);

    // Create output directory if it doesn't exist
    fs::create_dir_all(&args.output).map_err(crate::error::ViperError::Io)?;

    // Generate markdown file
    let input_stem =
        Path::new(&args.input).file_stem().and_then(|s| s.to_str()).unwrap_or("module");

    let output_path = Path::new(&args.output).join(format!("{}.md", input_stem));

    let mut content = String::new();
    content.push_str(&format!("# Module: {}\n\n", input_stem));

    if docs.is_empty() {
        content.push_str("No documented items found.\n");
    } else {
        for (name, doc) in &docs {
            content.push_str(&format!("## `{}`\n\n{}\n\n---\n\n", name, doc));
        }
    }

    fs::write(&output_path, content).map_err(crate::error::ViperError::Io)?;

    println!("Generated documentation: {}", output_path.display());

    Ok(())
}

fn extract_docstrings_simple(source: &str) -> Vec<(String, String)> {
    let mut docs = Vec::new();

    // Simple regex-like search for function definitions with docstrings
    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Look for def with docstring
        if line.starts_with("def ") && i + 1 < lines.len() {
            let next_line = lines[i + 1].trim();
            if next_line.starts_with("\"\"\"") || next_line.starts_with("'''") {
                // Extract function name
                let name_part =
                    line.trim_start_matches("def ").split('(').next().unwrap_or("unknown");
                let name = name_part.trim().to_string();

                // Extract docstring
                let mut doc = String::new();
                let mut j = i + 1;
                let closing = if next_line.starts_with("\"\"\"") { "\"\"\"" } else { "'''" };

                // Skip opening quote
                j += 1;
                while j < lines.len() {
                    let l = lines[j];
                    if l.trim().starts_with(closing) {
                        break;
                    }
                    if !doc.is_empty() {
                        doc.push('\n');
                    }
                    doc.push_str(l.trim());
                    j += 1;
                }

                if !doc.is_empty() {
                    docs.push((name, doc));
                }
                i = j;
            }
        }

        i += 1;
    }

    docs
}

pub struct DocArgs {
    pub input: String,
    pub output: String,
}

impl DocArgs {
    pub fn new(input: String, output: String) -> Self {
        Self { input, output }
    }
}
