fn main() {
    let source = r#"def main():
    x = 3 + 4
"#;

    println!("Source:\n{}\n---\n", source);

    let mut lexer = viper_lang::lexer::Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();

    println!("Tokens:");
    for (i, token) in tokens.iter().enumerate() {
        println!("{:3}: {:?}", i, token.kind);
    }

    println!("\n---\nParsing...");
    let mut parser = viper_lang::parser::Parser::new(tokens);
    match parser.parse() {
        Ok(ast) => {
            println!("Parsed {} statements", ast.statements.len());
            for (i, stmt) in ast.statements.iter().enumerate() {
                println!("  Stmt {}: {:?}", i, stmt);
            }
        }
        Err(e) => println!("Parse error: {}", e),
    }
}
