use super::super::*;
use crate::ast::Expr;
use crate::lexer::TokenKind;
use crate::utils::Span;

/// Parse list comprehension: [expr for var in iter] or [expr for var1, var2 in iter if cond]
pub fn parse_list_comprehension(
    parser: &mut StatementParser,
    element: Expr,
    span: Span,
) -> crate::error::Result<Expr> {
    // This is a list comprehension
    parser.advance(); // consume 'for'

    // Parse the target (can be identifier or tuple for unpacking)
    let mut target = parse_primary_expr(parser)?;
    
    // Check for tuple unpacking: for i, is_prime in ...
    if parser.match_token(&TokenKind::Comma) {
        let mut elements = vec![target];
        loop {
            elements.push(parse_primary_expr(parser)?);
            if !parser.match_token(&TokenKind::Comma) {
                break;
            }
        }
        let last_span = parser.previous().span;
        let merged_span = elements.first().unwrap().span().merge(last_span);
        target = Expr::Tuple { elements, span: merged_span };
    }

    // Expect 'in' keyword
    parser.expect(&TokenKind::In)?;

    // Parse the iterable
    let iter = parse_expression(parser)?;

    // Parse optional if clauses
    let mut ifs = Vec::new();
    while parser.match_token(&TokenKind::If) {
        ifs.push(parse_expression(parser)?);
    }

    // Expect closing bracket
    parser.expect(&TokenKind::RBracket)?;

    let list_span = span.merge(parser.previous().span);

    Ok(Expr::ListComprehension {
        element: Box::new(element),
        target: Box::new(target),
        iter: Box::new(iter),
        ifs,
        span: list_span,
    })
}
