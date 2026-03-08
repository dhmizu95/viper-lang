use super::*;
use crate::ast::{
    ExceptHandler, Expr, MatchCase, MatchPattern, SelectCase, SelectCaseKind, Stmt, Type, UnaryOp,
};
use crate::lexer::TokenKind;

pub fn parse_unless_stmt(parser: &mut StatementParser) -> Result<Stmt, String> {
    let start_span = parser.current().span;
    parser.expect(&TokenKind::Unless)?;

    let condition = parse_expression(parser)?;

    // Negate the condition
    let negated_condition =
        Expr::UnaryOp { op: UnaryOp::Not, operand: Box::new(condition), span: start_span };

    parser.expect(&TokenKind::Colon)?;

    let body = parse_block(parser)?;

    let span = start_span.merge(parser.previous().span);

    Ok(Stmt::If { condition: negated_condition, body, elif_blocks: vec![], else_body: None, span })
}

pub fn parse_if_stmt(parser: &mut StatementParser) -> Result<Stmt, String> {
    let start_span = parser.current().span;
    parser.expect(&TokenKind::If)?;

    let condition = parse_expression(parser)?;
    parser.expect(&TokenKind::Colon)?;
    let body = parse_block(parser)?;

    let mut elif_blocks = Vec::new();
    while parser.match_token(&TokenKind::Elif) {
        let elif_cond = parse_expression(parser)?;
        parser.expect(&TokenKind::Colon)?;
        let elif_body = parse_block(parser)?;
        elif_blocks.push((elif_cond, elif_body));
    }

    let else_body = if parser.match_token(&TokenKind::Else) {
        parser.expect(&TokenKind::Colon)?;
        Some(parse_block(parser)?)
    } else {
        None
    };

    let span = start_span.merge(parser.previous().span);

    Ok(Stmt::If { condition, body, elif_blocks, else_body, span })
}

pub fn parse_while_stmt(parser: &mut StatementParser) -> Result<Stmt, String> {
    let start_span = parser.current().span;
    parser.expect(&TokenKind::While)?;

    let condition = parse_expression(parser)?;
    parser.expect(&TokenKind::Colon)?;
    let body = parse_block(parser)?;

    let else_body = if parser.match_token(&TokenKind::Else) {
        parser.expect(&TokenKind::Colon)?;
        Some(parse_block(parser)?)
    } else {
        None
    };

    let span = start_span.merge(parser.previous().span);

    Ok(Stmt::While { condition, body, else_body, span })
}

pub fn parse_for_stmt(parser: &mut StatementParser) -> Result<Stmt, String> {
    let start_span = parser.current().span;
    parser.expect(&TokenKind::For)?;

    let target = if matches!(parser.current().kind, TokenKind::Ident(_))
        && matches!(parser.peek().map(|t| &t.kind), Some(TokenKind::In))
    {
        let name = parser.expect_ident()?;
        Ok(Expr::Ident(name, start_span.merge(parser.previous().span)))
    } else {
        parse_expression(parser)
    }?;
    parser.expect(&TokenKind::In)?;
    let iter = parse_expression(parser)?;
    parser.expect(&TokenKind::Colon)?;
    let body = parse_block(parser)?;

    let else_body = if parser.match_token(&TokenKind::Else) {
        parser.expect(&TokenKind::Colon)?;
        Some(parse_block(parser)?)
    } else {
        None
    };

    let span = start_span.merge(parser.previous().span);

    Ok(Stmt::For {
        target: Box::new(target),
        iter: Box::new(iter),
        body,
        else_body,
        is_async: false,
        span,
    })
}

pub fn parse_async_for_stmt(parser: &mut StatementParser) -> Result<Stmt, String> {
    let start_span = parser.current().span;
    parser.expect(&TokenKind::Async)?;
    parser.expect(&TokenKind::For)?;

    let target = if matches!(parser.current().kind, TokenKind::Ident(_))
        && matches!(parser.peek().map(|t| &t.kind), Some(TokenKind::In))
    {
        let name = parser.expect_ident()?;
        Ok(Expr::Ident(name, start_span.merge(parser.previous().span)))
    } else {
        parse_expression(parser)
    }?;
    parser.expect(&TokenKind::In)?;
    let iter = parse_expression(parser)?;
    parser.expect(&TokenKind::Colon)?;
    let body = parse_block(parser)?;

    let else_body = if parser.match_token(&TokenKind::Else) {
        parser.expect(&TokenKind::Colon)?;
        Some(parse_block(parser)?)
    } else {
        None
    };

    let span = start_span.merge(parser.previous().span);

    Ok(Stmt::For {
        target: Box::new(target),
        iter: Box::new(iter),
        body,
        else_body,
        is_async: true,
        span,
    })
}
pub fn parse_try_stmt(parser: &mut StatementParser) -> Result<Stmt, String> {
    let start_span = parser.current().span;
    parser.expect(&TokenKind::Try)?;
    parser.expect(&TokenKind::Colon)?;
    let body = parse_block(parser)?;

    let mut handlers = Vec::new();
    while parser.match_token(&TokenKind::Except) {
        let handler_span = parser.previous().span;

        // Parse exception type - can be an identifier (Exception class) or type annotation
        let type_ann = if matches!(parser.current().kind, TokenKind::Ident(_)) {
            // For exception handlers, just capture the type name as a Type::Var
            let name = parser.expect_ident()?;
            Some(Type::Var(name))
        } else {
            None
        };

        let name =
            if parser.match_token(&TokenKind::As) { Some(parser.expect_ident()?) } else { None };

        parser.expect(&TokenKind::Colon)?;
        let handler_body = parse_block(parser)?;

        handlers.push(ExceptHandler { type_ann, name, body: handler_body, span: handler_span });
    }

    let else_body = if parser.match_token(&TokenKind::Else) {
        parser.expect(&TokenKind::Colon)?;
        Some(parse_block(parser)?)
    } else {
        None
    };

    let finally_body = if parser.match_token(&TokenKind::Finally) {
        parser.expect(&TokenKind::Colon)?;
        Some(parse_block(parser)?)
    } else {
        None
    };

    let span = start_span.merge(parser.previous().span);

    Ok(Stmt::Try { body, handlers, else_body, finally_body, span })
}
pub fn parse_match_stmt(parser: &mut StatementParser) -> Result<Stmt, String> {
    let start_span = parser.current().span;
    parser.expect(&TokenKind::Match)?;

    let subject = parse_expression(parser)?;

    parser.expect(&TokenKind::Colon)?;

    // Allow either newline or immediate case
    let mut cases = Vec::new();

    // Check if we have case directly or need newline/indent
    if !matches!(parser.current().kind, TokenKind::Case) {
        // Allow Newline or Indent
        if matches!(parser.current().kind, TokenKind::Newline) {
            parser.advance();
        }
        // Skip optional indent
        if matches!(parser.current().kind, TokenKind::Indent) {
            parser.expect(&TokenKind::Indent)?;
        }
    }

    while matches!(parser.current().kind, TokenKind::Case) {
        let case_span = parser.current().span;
        parser.expect(&TokenKind::Case)?;

        let pattern = parse_match_pattern(parser)?;

        let guard =
            if parser.match_token(&TokenKind::If) { Some(parse_expression(parser)?) } else { None };

        parser.expect(&TokenKind::Colon)?;

        // Parse case body - can be single statement or indented block
        let mut body = Vec::new();

        // Skip optional newline after colon
        parser.match_token(&TokenKind::Newline);

        // Check for indented block
        if matches!(parser.current().kind, TokenKind::Indent) {
            parser.expect(&TokenKind::Indent)?;
            loop {
                if matches!(parser.current().kind, TokenKind::Dedent) {
                    parser.advance();
                    break;
                }
                if matches!(parser.current().kind, TokenKind::Case) {
                    break;
                }
                if parser.is_at_end() {
                    break;
                }
                if parser.match_token(&TokenKind::Newline) {
                    continue;
                }
                body.push(parse_statement(parser)?);
            }
        } else if !matches!(parser.current().kind, TokenKind::Case) {
            // Single statement on same line
            body.push(parse_statement(parser)?);
        }

        cases.push(MatchCase {
            pattern,
            guard,
            body,
            span: case_span.merge(parser.previous().span),
        });
    }

    let span = start_span.merge(parser.previous().span);

    Ok(Stmt::Match { subject: Box::new(subject), cases, span })
}

pub fn parse_select_stmt(parser: &mut StatementParser) -> Result<Stmt, String> {
    let start_span = parser.current().span;
    parser.expect(&TokenKind::Select)?;
    parser.expect(&TokenKind::Colon)?;

    let mut cases = Vec::new();

    // Handle optional newline
    if matches!(parser.current().kind, TokenKind::Newline) {
        parser.expect(&TokenKind::Newline)?;
        if matches!(parser.current().kind, TokenKind::Indent) {
            parser.expect(&TokenKind::Indent)?;
        }
    }

    while matches!(parser.current().kind, TokenKind::Case) {
        let case_span = parser.current().span;
        parser.expect(&TokenKind::Case)?;

        // Parse the case kind: recv, send, or default
        let kind = if parser.match_token(&TokenKind::Recv) {
            parser.expect(&TokenKind::LParen)?;
            let chan = parse_expression(parser)?;
            parser.expect(&TokenKind::RParen)?;
            let var = if parser.match_token(&TokenKind::Eq) {
                Some(parser.expect_ident()?)
            } else {
                None
            };
            SelectCaseKind::Recv { chan: Box::new(chan), var }
        } else if parser.match_token(&TokenKind::Send) {
            parser.expect(&TokenKind::LParen)?;
            let chan = parse_expression(parser)?;
            parser.expect(&TokenKind::Comma)?;
            let value = parse_expression(parser)?;
            parser.expect(&TokenKind::RParen)?;
            SelectCaseKind::Send { chan: Box::new(chan), value: Box::new(value) }
        } else if let TokenKind::Ident(name) = &parser.current().kind {
            if name == "default" {
                parser.advance();
                SelectCaseKind::Default
            } else {
                return Err("Expected 'recv', 'send', or 'default' in select case".to_string());
            }
        } else {
            return Err("Expected 'recv', 'send', or 'default' in select case".to_string());
        };

        parser.expect(&TokenKind::Colon)?;

        // Parse body
        parser.match_token(&TokenKind::Newline);
        let mut body = Vec::new();
        if matches!(parser.current().kind, TokenKind::Indent) {
            parser.expect(&TokenKind::Indent)?;
            loop {
                if matches!(parser.current().kind, TokenKind::Dedent) {
                    parser.advance();
                    break;
                }
                if matches!(parser.current().kind, TokenKind::Case) {
                    break;
                }
                if parser.is_at_end() {
                    break;
                }
                if parser.match_token(&TokenKind::Newline) {
                    continue;
                }
                body.push(parse_statement(parser)?);
            }
        } else if !matches!(parser.current().kind, TokenKind::Case) {
            body.push(parse_statement(parser)?);
        }

        cases.push(SelectCase { kind, body, span: case_span });
    }

    let span = start_span.merge(parser.previous().span);
    Ok(Stmt::Select { cases, span })
}

pub fn parse_match_pattern(parser: &mut StatementParser) -> Result<MatchPattern, String> {
    let token = parser.current().clone();

    match token.kind {
        TokenKind::Underscore => {
            parser.advance();
            Ok(MatchPattern::Wildcard)
        }
        TokenKind::Int(ref n) => {
            let span = token.span;
            parser.advance();
            // Check if the integer fits in i64
            if *n > i64::MAX as i128 || *n < i64::MIN as i128 {
                // Convert to BigInt pattern if too large for i64
                Ok(MatchPattern::Constant(Expr::BigInt(n.to_string(), span)))
            } else {
                Ok(MatchPattern::Constant(Expr::Int(*n as i64, span)))
            }
        }
        TokenKind::BigInt(ref s) => {
            let span = token.span;
            parser.advance();
            Ok(MatchPattern::Constant(Expr::BigInt(s.clone(), span)))
        }
        TokenKind::Str(s) => {
            let span = token.span;
            parser.advance();
            Ok(MatchPattern::Constant(Expr::Str(s, span)))
        }
        TokenKind::Bytes(b) => {
            let span = token.span;
            parser.advance();
            Ok(MatchPattern::Constant(Expr::Bytes(b, span)))
        }
        TokenKind::Bool(b) => {
            let span = token.span;
            parser.advance();
            Ok(MatchPattern::Constant(Expr::Bool(b, span)))
        }
        TokenKind::None => {
            let span = token.span;
            parser.advance();
            Ok(MatchPattern::Constant(Expr::None(span)))
        }
        TokenKind::LBracket => parse_match_list_pattern(parser),
        TokenKind::LParen => parse_match_tuple_pattern(parser),
        TokenKind::Ident(name) => {
            parser.advance();
            let next_token = parser.current();
            if matches!(next_token.kind, TokenKind::LParen) {
                parse_match_type_pattern(parser, &name)
            } else {
                Ok(MatchPattern::Variable(name))
            }
        }
        _ => Err(format!("Unexpected token in pattern: {:?}", token.kind)),
    }
}

pub fn parse_match_list_pattern(parser: &mut StatementParser) -> Result<MatchPattern, String> {
    parser.expect(&TokenKind::LBracket)?;

    let mut elements = Vec::new();
    let mut rest = None;

    if !matches!(parser.current().kind, TokenKind::RBracket) {
        loop {
            if matches!(parser.current().kind, TokenKind::DotDot) {
                parser.expect(&TokenKind::DotDot)?;
                rest = Some(parser.expect_ident()?);
                break;
            }
            elements.push(parse_match_pattern(parser)?);
            if !parser.match_token(&TokenKind::Comma) {
                break;
            }
        }
    }

    parser.expect(&TokenKind::RBracket)?;

    Ok(MatchPattern::List { elements, rest })
}

pub fn parse_match_tuple_pattern(parser: &mut StatementParser) -> Result<MatchPattern, String> {
    parser.expect(&TokenKind::LParen)?;

    let mut elements = Vec::new();
    if !matches!(parser.current().kind, TokenKind::RParen) {
        loop {
            elements.push(parse_match_pattern(parser)?);
            if !parser.match_token(&TokenKind::Comma) {
                break;
            }
        }
    }

    parser.expect(&TokenKind::RParen)?;

    Ok(MatchPattern::Tuple(elements))
}

pub fn parse_match_type_pattern(
    parser: &mut StatementParser,
    type_name: &str,
) -> Result<MatchPattern, String> {
    parser.expect(&TokenKind::LParen)?;

    // Check if there's a binding identifier
    // The binding is captured when followed by RParen (single pattern) or Comma (in tuple)
    let binding = if matches!(parser.current().kind, TokenKind::Ident(_)) {
        let next_is_end = matches!(
            parser.peek().map(|t| &t.kind),
            Some(TokenKind::RParen) | Some(TokenKind::Comma)
        );
        if next_is_end {
            Some(parser.expect_ident()?)
        } else {
            None
        }
    } else {
        None
    };

    parser.expect(&TokenKind::RParen)?;

    Ok(MatchPattern::TypeCheck { type_name: type_name.to_string(), binding })
}
