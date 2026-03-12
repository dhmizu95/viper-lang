use super::super::*;
use crate::ast::{Expr, Stmt};
use crate::lexer::TokenKind;
use crate::utils::Span;

/// Parse lambda/fn expression: fn(x, y) : body or lambda x, y: body
pub fn parse_lambda_expr(parser: &mut StatementParser, span: Span) -> crate::error::Result<Expr> {
    let mut params = Vec::new();

    // Handle optional parentheses around parameters: fn(x, y: expr) or fn(x: expr)
    let paren_params = parser.match_token(&TokenKind::LParen);

    if !matches!(parser.current().kind, TokenKind::Colon) {
        loop {
            if let TokenKind::Ident(param_name) = &parser.current().kind {
                params.push(param_name.clone());
                parser.advance();
            } else if paren_params && matches!(parser.current().kind, TokenKind::RParen) {
                // Empty parameter list like fn(): expr
                break;
            } else {
                return crate::parser::parse_error("Expected parameter name in lambda".to_string());
            }

            if parser.match_token(&TokenKind::Comma) {
                // Check if there's another parameter or closing paren
                if paren_params && matches!(parser.current().kind, TokenKind::RParen) {
                    break;
                }
                continue;
            } else if paren_params && parser.match_token(&TokenKind::Colon) {
                // Shorthand syntax: fn(x, y: body)
                let body = parse_expression(parser)?;
                parser.expect(&TokenKind::RParen)?;
                let merged_span = span.merge(body.span());
                return Ok(Expr::Lambda {
                    params,
                    body: Box::new(body),
                    span: merged_span,
                });
            } else {
                break;
            }
        }
    }

    // If we opened a paren, expect closing paren before the colon
    if paren_params {
        parser.expect(&TokenKind::RParen)?;
    }

    parser.expect(&TokenKind::Colon)?;
    let body = parse_expression(parser)?;
    let merged_span = span.merge(body.span());
    Ok(Expr::Lambda {
        params,
        body: Box::new(body),
        span: merged_span,
    })
}

/// Parse type alias: type Name = Type
pub fn parse_type_alias(parser: &mut StatementParser) -> crate::error::Result<Stmt> {
    let span = parser.current().span;
    parser.expect(&TokenKind::Type)?;

    let name = parser.expect_ident()?;

    parser.expect(&TokenKind::Eq)?;

    let type_def = parse_type_annotation(parser)?;

    Ok(Stmt::TypeAlias { name, type_def, span })
}
