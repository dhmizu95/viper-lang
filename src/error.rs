use crate::utils::Span;
use thiserror::Error;

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

impl ViperError {
    pub fn lexical(message: impl Into<String>, line: usize, column: usize) -> Self {
        Self::Lexical(message.into(), line, column)
    }

    pub fn parser(message: impl Into<String>, span: Span) -> Self {
        Self::Parser(message.into(), span)
    }

    pub fn type_error(message: impl Into<String>, span: Span) -> Self {
        Self::TypeError(message.into(), span)
    }

    pub fn codegen(message: impl Into<String>) -> Self {
        Self::Codegen(message.into())
    }

    pub fn driver(message: impl Into<String>) -> Self {
        Self::Driver(message.into())
    }

    pub fn cli(message: impl Into<String>) -> Self {
        Self::Cli(message.into())
    }
}

impl From<String> for ViperError {
    fn from(value: String) -> Self {
        ViperError::Driver(value)
    }
}

impl From<&str> for ViperError {
    fn from(value: &str) -> Self {
        ViperError::Driver(value.to_string())
    }
}
