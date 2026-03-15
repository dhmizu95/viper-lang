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
            if !parser.match_token(&TokenKind::RParen) {
                loop {
                    // Check if this is a keyword argument
                    let saved_pos = parser.pos;
                    let mut is_keyword = false;

                    if let TokenKind::Ident(name) = &parser.current().kind {
                        let name = name.clone();
                        parser.advance();
                        if parser.match_token(&TokenKind::Eq)
                            || parser.match_token(&TokenKind::EqEq)
                        {
                            let value = parse_expression(parser)?;
                            keywords.push((name, value));
                            is_keyword = true;
                        } else {
                            parser.pos = saved_pos;
                        }
                    }

                    if !is_keyword {
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
            if !parser.match_token(&TokenKind::RParen) {
                loop {
                    // Check if this is a keyword argument
                    let saved_pos = parser.pos;
                    let mut is_keyword = false;

                    if let TokenKind::Ident(name) = &parser.current().kind {
                        let name = name.clone();
                        parser.advance();
                        if parser.match_token(&TokenKind::Eq)
                            || parser.match_token(&TokenKind::EqEq)
                        {
                            let value = parse_expression(parser)?;
                            keywords.push((name, value));
                            is_keyword = true;
                        } else {
                            parser.pos = saved_pos;
                        }
                    }

                    if !is_keyword {
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
            if !parser.match_token(&TokenKind::RParen) {
                loop {
                    // Check if this is a keyword argument
                    let saved_pos = parser.pos;
                    let mut is_keyword = false;

                    if let TokenKind::Ident(name) = &parser.current().kind {
                        let name = name.clone();
                        parser.advance();
                        if parser.match_token(&TokenKind::Eq)
                            || parser.match_token(&TokenKind::EqEq)
                        {
                            let value = parse_expression(parser)?;
                            keywords.push((name, value));
                            is_keyword = true;
                        } else {
                            parser.pos = saved_pos;
                        }
                    }

                    if !is_keyword {
                        args.push(parse_expression(parser)?);
                    }
                    if !parser.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
                parser.expect(&TokenKind::RParen)?;
            }
            let call_span = span.merge(parser.previous().span);
            expr = Expr::Call { func: Box::new(expr), args, keywords: Vec::new(), span: call_span };
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
