use thiserror::Error;
use crate::utils::Span;

#[derive(Error, Debug)]
pub enum ViperError {
    #[error("Lexical error: {0} at line {1}, col {2}")]
    Lexical(String, usize, usize),

    #[error("Parse error: {0} at {1}")]
    Parser(String, Span),

    #[error("Type error: {0} at {1}")]
    TypeError(String, Span),

    #[error("Codegen error: {0}")]
    Codegen(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Driver error: {0}")]
    Driver(String),

    #[error("CLI error: {0}")]
    Cli(String),
}

pub type Result<T> = std::result::Result<T, ViperError>;
