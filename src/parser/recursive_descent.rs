use crate::ast::Module;
use crate::error::{Result, ViperError};
use crate::lexer::Token;
use crate::parser::statements::{parse_statements, StatementParser};
use crate::utils::Span;

/// Main parser that coordinates statement parsing
pub struct Parser {
    tokens: Vec<Token>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens }
    }

    /// Parse the token stream into an AST
    pub fn parse(&mut self) -> Result<Module> {
        self.parse_raw().map_err(ViperError::driver)
    }

    fn parse_raw(&mut self) -> std::result::Result<Module, String> {
        if self.tokens.is_empty() {
            return Ok(Module { statements: Vec::new(), span: Span::empty(0, 0) });
        }

        let mut stmt_parser = StatementParser::new(&self.tokens);
        let statements = parse_statements(&mut stmt_parser)?;

        let span = if statements.is_empty() {
            Span::empty(0, 0)
        } else {
            let first_span = statements.first().unwrap().span();
            let last_span = statements.last().unwrap().span();
            first_span.merge(last_span)
        };

        Ok(Module { statements, span })
    }
}
