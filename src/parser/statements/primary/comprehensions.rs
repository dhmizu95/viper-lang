use super::super::*;
use crate::ast::Expr;
use crate::lexer::TokenKind;
use crate::utils::Span;

/// Parse list comprehension: [expr for var in iter]
pub fn parse_list_comprehension(
    parser: &mut StatementParser,
    element: Expr,
    span: Span,
) -> crate::error::Result<Expr> {
    // This is a list comprehension
    parser.advance(); // consume 'for'

    // Parse the variable name
    let var = if let TokenKind::Ident(name) = &parser.current().kind {
        let name = name.clone();
        parser.advance();
        name
    } else {
        return crate::parser::parse_error("Expected variable name in list comprehension".to_string());
    };

    // Expect 'in' keyword
    parser.expect(&TokenKind::In)?;

    // Parse the iterable
    let iter = parse_expression(parser)?;

    // Expect closing bracket
    parser.expect(&TokenKind::RBracket)?;

    let list_span = span.merge(parser.previous().span);

    Ok(Expr::ListComprehension {
        element: Box::new(element),
        var,
        iter: Box::new(iter),
        span: list_span,
    })
}
