use std::fs;

fn main() {
    let source = fs::read_to_string("test_factorial.vp").unwrap();
    println!("Source:\n{}\n", source);
    
    // Simple test - just show the tokens
    println!("Testing lexer...");
}
