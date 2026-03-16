use super::super::*;
use crate::ast::Expr;
use crate::lexer::TokenKind;
use crate::utils::Span;

/// Parse identifier expression with postfix operators (attribute, call, index/slice)
pub fn parse_ident_expr(
    parser: &mut StatementParser,
    name: String,
    span: Span,
) -> crate::error::Result<Expr> {
    parser.advance();
    // Check for attribute access or function call
    let mut expr = Expr::Ident(name, span);

    loop {
        if parser.match_token(&TokenKind::Dot) {
            let attr = parser.expect_ident()?;
            let attr_span = parser.previous().span;
            expr = Expr::Attribute { obj: Box::new(expr), attr, span: span.merge(attr_span) };
        } else if parser.match_token(&TokenKind::LParen) {
            let mut args = Vec::new();
            let mut keywords = Vec::new();
            let mut seen_keyword = false;
            if !parser.match_token(&TokenKind::RParen) {
                loop {
                    let is_keyword = matches!(parser.current().kind, TokenKind::Ident(_))
                        && parser.peek().map_or(false, |t| matches!(t.kind, TokenKind::Eq));
                    if is_keyword {
                        let keyword_name = parser.expect_ident()?;
                        parser.expect(&TokenKind::Eq)?;
                        let keyword_value = parse_expression(parser)?;
                        keywords.push((keyword_name, keyword_value));
                        seen_keyword = true;
                    } else {
                        if seen_keyword {
                            return crate::parser::parse_error(
                                "Positional arguments cannot follow keyword arguments".to_string(),
                            );
                        }
                        args.push(parse_expression(parser)?);
                    }
                    if !parser.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
                parser.expect(&TokenKind::RParen)?;
            }
            // RParen was already consumed by match_token if it matched
            let call_span = span.merge(parser.previous().span);
            expr = Expr::Call { func: Box::new(expr), args, keywords, span: call_span };
        } else if parser.match_token(&TokenKind::LBracket) {
            // Parse slice or index
            let is_slice = super::is_slice_pattern(parser);

            if is_slice {
                expr = parse_slice_expr(parser, expr, span)?;
            } else {
                // Regular indexing
                let index = parse_expression(parser)?;
                parser.expect(&TokenKind::RBracket)?;
                let index_span = span.merge(parser.previous().span);
                expr =
                    Expr::Index { obj: Box::new(expr), index: Box::new(index), span: index_span };
            }
        } else {
            break;
        }
    }

    Ok(expr)
}

/// Parse send keyword as identifier with postfix operators
pub fn parse_send_expr(parser: &mut StatementParser, span: Span) -> crate::error::Result<Expr> {
    parser.advance();
    // Treat as identifier "send" for function call syntax
    let mut expr = Expr::Ident("send".to_string(), span);

    loop {
        if parser.match_token(&TokenKind::Dot) {
            let attr = parser.expect_ident()?;
            let attr_span = parser.previous().span;
            expr = Expr::Attribute { obj: Box::new(expr), attr, span: span.merge(attr_span) };
        } else if parser.match_token(&TokenKind::LParen) {
            let mut args = Vec::new();
            let mut keywords = Vec::new();
            let mut seen_keyword = false;
            if !parser.match_token(&TokenKind::RParen) {
                loop {
                    let is_keyword = matches!(parser.current().kind, TokenKind::Ident(_))
                        && parser.peek().map_or(false, |t| matches!(t.kind, TokenKind::Eq));
                    if is_keyword {
                        let keyword_name = parser.expect_ident()?;
                        parser.expect(&TokenKind::Eq)?;
                        let keyword_value = parse_expression(parser)?;
                        keywords.push((keyword_name, keyword_value));
                        seen_keyword = true;
                    } else {
                        if seen_keyword {
                            return crate::parser::parse_error(
                                "Positional arguments cannot follow keyword arguments".to_string(),
                            );
                        }
                        args.push(parse_expression(parser)?);
                    }
                    if !parser.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
                parser.expect(&TokenKind::RParen)?;
            }
            let call_span = span.merge(parser.previous().span);
            expr = Expr::Call { func: Box::new(expr), args, keywords, span: call_span };
        } else if parser.match_token(&TokenKind::LBracket) {
            let is_slice = super::is_slice_pattern(parser);

            if is_slice {
                expr = parse_slice_expr(parser, expr, span)?;
            } else {
                let index = parse_expression(parser)?;
                parser.expect(&TokenKind::RBracket)?;
                let index_span = span.merge(parser.previous().span);
                expr =
                    Expr::Index { obj: Box::new(expr), index: Box::new(index), span: index_span };
            }
        } else {
            break;
        }
    }

    Ok(expr)
}

/// Parse recv keyword as identifier with postfix operators
pub fn parse_recv_expr(parser: &mut StatementParser, span: Span) -> crate::error::Result<Expr> {
    parser.advance();
    // Treat as identifier "recv" for function call syntax
    let mut expr = Expr::Ident("recv".to_string(), span);

    loop {
        if parser.match_token(&TokenKind::Dot) {
            let attr = parser.expect_ident()?;
            let attr_span = parser.previous().span;
            expr = Expr::Attribute { obj: Box::new(expr), attr, span: span.merge(attr_span) };
        } else if parser.match_token(&TokenKind::LParen) {
            let mut args = Vec::new();
            let mut keywords = Vec::new();
            let mut seen_keyword = false;
            if !parser.match_token(&TokenKind::RParen) {
                loop {
                    let is_keyword = matches!(parser.current().kind, TokenKind::Ident(_))
                        && parser.peek().map_or(false, |t| matches!(t.kind, TokenKind::Eq));
                    if is_keyword {
                        let keyword_name = parser.expect_ident()?;
                        parser.expect(&TokenKind::Eq)?;
                        let keyword_value = parse_expression(parser)?;
                        keywords.push((keyword_name, keyword_value));
                        seen_keyword = true;
                    } else {
                        if seen_keyword {
                            return crate::parser::parse_error(
                                "Positional arguments cannot follow keyword arguments".to_string(),
                            );
                        }
                        args.push(parse_expression(parser)?);
                    }
                    if !parser.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
                parser.expect(&TokenKind::RParen)?;
            }
            let call_span = span.merge(parser.previous().span);
            expr = Expr::Call { func: Box::new(expr), args, keywords, span: call_span };
        } else if parser.match_token(&TokenKind::LBracket) {
            let is_slice = super::is_slice_pattern(parser);

            if is_slice {
                expr = parse_slice_expr(parser, expr, span)?;
            } else {
                let index = parse_expression(parser)?;
                parser.expect(&TokenKind::RBracket)?;
                let index_span = span.merge(parser.previous().span);
                expr =
                    Expr::Index { obj: Box::new(expr), index: Box::new(index), span: index_span };
            }
        } else {
            break;
        }
    }

    Ok(expr)
}

/// Parse slice expression: [:], [start:], [:end], [start:end], [::step], etc.
fn parse_slice_expr(
    parser: &mut StatementParser,
    obj: Expr,
    span: Span,
) -> crate::error::Result<Expr> {
    let mut start: Option<Box<Expr>> = None;
    let mut end: Option<Box<Expr>> = None;
    let mut step: Option<Box<Expr>> = None;

    // Parse start (optional)
    if !matches!(parser.current().kind, TokenKind::Colon) {
        start = Some(Box::new(parse_expression(parser)?));
    }

    // Expect first colon
    parser.expect(&TokenKind::Colon)?;

    // Parse end (optional)
    if !matches!(parser.current().kind, TokenKind::RBracket)
        && !matches!(parser.current().kind, TokenKind::Colon)
    {
        end = Some(Box::new(parse_expression(parser)?));
    }

    // Check for step
    if matches!(parser.current().kind, TokenKind::Colon) {
        parser.expect(&TokenKind::Colon)?;
        // Parse step (optional)
        if !matches!(parser.current().kind, TokenKind::RBracket) {
            step = Some(Box::new(parse_expression(parser)?));
        }
    }

    parser.expect(&TokenKind::RBracket)?;
    let index_span = span.merge(parser.previous().span);

    Ok(Expr::Slice { obj: Box::new(obj), start, end, step, span: index_span })
}
