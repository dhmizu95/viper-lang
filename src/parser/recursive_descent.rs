use crate::ast::Module;
use crate::lexer::Token;
use crate::parser::statements::StatementParser;
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
    pub fn parse(&mut self) -> Result<Module, String> {
        if self.tokens.is_empty() {
            return Ok(Module {
                statements: Vec::new(),
                span: Span::empty(0, 0),
            });
        }

        let mut stmt_parser = StatementParser::new(&self.tokens);
        let statements = stmt_parser.parse_statements()?;

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
