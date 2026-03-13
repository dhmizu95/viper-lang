use super::super::*;
use crate::ast::Expr;
use crate::lexer::TokenKind;
use crate::utils::Span;

/// Parse tuple literal (including empty tuple)
pub fn parse_tuple_literal(parser: &mut StatementParser, span: Span) -> crate::error::Result<Expr> {
    parser.advance(); // consume LParen

    // Check for empty tuple
    if parser.match_token(&TokenKind::RParen) {
        // Empty tuple: ()
        return Ok(Expr::Tuple { elements: vec![], span });
    }

    let expr = parse_expression(parser)?;

    // Check for tuple (has trailing comma)
    if parser.match_token(&TokenKind::Comma) {
        let mut elements = vec![expr];
        // Check for trailing comma (single-element tuple): (x,)
        if parser.match_token(&TokenKind::RParen) {
            // Single-element tuple with trailing comma
            let merged_span = span.merge(parser.previous().span);
            return Ok(Expr::Tuple { elements, span: merged_span });
        }
        // More elements follow - parse them
        loop {
            elements.push(parse_expression(parser)?);
            if parser.match_token(&TokenKind::Comma) {
                // Check for trailing comma after multiple elements
                if parser.match_token(&TokenKind::RParen) {
                    let merged_span = span.merge(parser.previous().span);
                    return Ok(Expr::Tuple { elements, span: merged_span });
                }
                // Continue parsing more elements
            } else {
                break;
            }
        }
        parser.expect(&TokenKind::RParen)?;
        let merged_span = span.merge(parser.previous().span);
        return Ok(Expr::Tuple { elements, span: merged_span });
    }

    parser.expect(&TokenKind::RParen)?;
    Ok(expr)
}

/// Parse list, array, or list comprehension
pub fn parse_list_or_array(parser: &mut StatementParser, span: Span) -> crate::error::Result<Expr> {
    parser.advance(); // consume LBracket
    let mut elements = Vec::new();
    let mut size: Option<usize> = None;

    // Check for empty list without consuming the RBracket
    if !matches!(parser.current().kind, TokenKind::RBracket) {
        // Parse first element
        let first_elem = parse_expression(parser)?;

        // Check for list comprehension: [expr for var in iter]
        if matches!(parser.current().kind, TokenKind::For) {
            return super::comprehensions::parse_list_comprehension(parser, first_elem, span);
        }

        // Not a list comprehension, treat as array/list
        elements.push(first_elem);

        // Check for array repetition syntax: [value; size]
        if matches!(parser.current().kind, TokenKind::Semi) {
            parser.advance(); // consume the semicolon
            let size_token = parser.current();
            match &size_token.kind {
                TokenKind::Int(n) => {
                    if *n < 0 || *n > usize::MAX as i128 {
                        return crate::parser::parse_error(format!(
                            "Array size must be a positive usize: {}",
                            n
                        ));
                    }
                    size = Some(*n as usize);
                    parser.advance();
                }
                _ => {
                    return crate::parser::parse_error(format!(
                        "Expected integer size for array, found {:?}",
                        size_token.kind
                    ))
                }
            }
            parser.expect(&TokenKind::RBracket)?;
        } else {
            // Regular list/array: parse remaining elements
            while parser.match_token(&TokenKind::Comma) {
                if matches!(parser.current().kind, TokenKind::RBracket) {
                    break;
                }
                elements.push(parse_expression(parser)?);
            }
            parser.expect(&TokenKind::RBracket)?;
        }
    } else {
        parser.expect(&TokenKind::RBracket)?;
    }

    let list_span = span.merge(parser.previous().span);

    // Use Array node for fixed-size arrays, List for dynamic lists
    if size.is_some() {
        Ok(Expr::Array { elements, size, span: list_span })
    } else {
        Ok(Expr::List { elements, span: list_span })
    }
}

/// Parse dict literal
pub fn parse_dict_literal(parser: &mut StatementParser, span: Span) -> crate::error::Result<Expr> {
    parser.advance(); // consume LBrace
    let mut pairs = Vec::new();

    // Handle empty dict: {}
    if parser.match_token(&TokenKind::RBrace) {
        let last_span = parser.previous().span;
        let merged_span = span.merge(last_span);
        return Ok(Expr::Dict { pairs, span: merged_span });
    }

    // Parse key-value pairs
    loop {
        let key = parse_expression(parser)?;
        parser.expect(&TokenKind::Colon)?;
        let value = parse_expression(parser)?;
        pairs.push((key, value));

        if !parser.match_token(&TokenKind::Comma) {
            break;
        }

        // Handle trailing comma: {key: value,}
        if parser.match_token(&TokenKind::RBrace) {
            break;
        }
    }

    parser.expect(&TokenKind::RBrace)?;
    let last_span = parser.previous().span;
    let merged_span = span.merge(last_span);

    Ok(Expr::Dict { pairs, span: merged_span })
}
