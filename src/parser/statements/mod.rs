use crate::ast::{
    BinOp, ExceptHandler, Expr, MatchCase, MatchPattern, Param, SelectCase, SelectCaseKind, Stmt,
    Type, UnaryOp,
};
use crate::lexer::{Token, TokenKind};
use crate::parser::expressions::PrattParser;
use crate::utils::Span;

pub mod core;
pub mod definitions;
pub mod control_flow;
pub mod concurrency;
pub mod primary;

pub use core::*;
pub use definitions::*;
pub use control_flow::*;
pub use concurrency::*;
pub use primary::*;

/// Statement parser for Viper
pub struct StatementParser<'a> {
    pub(crate) tokens: &'a [Token],
    pub(crate) pos: usize,
    pub(crate) expr_parser: PrattParser<'a>,
}

impl<'a> StatementParser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens,
            pos: 0,
            expr_parser: PrattParser::new(tokens),
        }
    }

    pub(crate) fn current(&self) -> &Token {
        &self.tokens[self.pos]
    }

    pub(crate) fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos + 1)
    }

    pub(crate) fn previous(&self) -> &Token {
        &self.tokens[self.pos - 1]
    }

    pub(crate) fn advance(&mut self) {
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
    }

    pub(crate) fn match_token(&mut self, kind: &TokenKind) -> bool {
        if std::mem::discriminant(&self.current().kind) == std::mem::discriminant(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    pub(crate) fn expect(&mut self, kind: &TokenKind) -> Result<(), String> {
        if std::mem::discriminant(&self.current().kind) == std::mem::discriminant(kind) {
            self.advance();
            Ok(())
        } else {
            Err(format!(
                "Expected {:?}, found {:?} at line {}",
                kind,
                self.current().kind,
                self.current().span.line
            ))
        }
    }

    pub(crate) fn expect_ident(&mut self) -> Result<String, String> {
        if let TokenKind::Ident(name) = &self.current().kind {
            let name = name.clone();
            self.advance();
            Ok(name)
        } else {
            Err(format!(
                "Expected identifier, found {:?} at line {}",
                self.current().kind,
                self.current().span.line
            ))
        }
    }

    pub(crate) fn is_at_end(&self) -> bool {
        matches!(self.current().kind, TokenKind::Eof)
    }
}
