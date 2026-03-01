use super::*;
use crate::ast::{BinOp, Expr, Stmt};
use crate::lexer::TokenKind;

/// Parse global variable declaration: global x, y, z
/// Python syntax: global x (inside function to refer to module-level x)
pub fn parse_global_decl(parser: &mut StatementParser) -> Result<Stmt, String> {
    let span = parser.current().span;
    parser.expect(&TokenKind::Global)?;

    // Parse comma-separated list of variable names
    let mut names = vec![parser.expect_ident()?];
    while parser.match_token(&TokenKind::Comma) {
        names.push(parser.expect_ident()?);
    }

    Ok(Stmt::Global { names, span })
}

/// Parse nonlocal variable declaration: nonlocal x, y
/// Python syntax: nonlocal x (inside nested function to refer to enclosing scope x)
pub fn parse_nonlocal_decl(parser: &mut StatementParser) -> Result<Stmt, String> {
    let span = parser.current().span;
    parser.expect(&TokenKind::Nonlocal)?;

    // Parse comma-separated list of variable names
    let mut names = vec![parser.expect_ident()?];
    while parser.match_token(&TokenKind::Comma) {
        names.push(parser.expect_ident()?);
    }

    Ok(Stmt::Nonlocal { names, span })
}

/// Parse constant declaration: const PI = 3.14
pub fn parse_const_decl(parser: &mut StatementParser) -> Result<Stmt, String> {
    let span = parser.current().span;
    parser.expect(&TokenKind::Const)?;

    let name = parser.expect_ident()?;

    // Constants must have an initializer
    if !parser.match_token(&TokenKind::Eq) {
        return Err("Constant declaration must have an initializer".to_string());
    }

    let value = parse_expression(parser)?;

    Ok(Stmt::Const { name, value, span })
}

pub fn parse_assignment_or_expr(parser: &mut StatementParser) -> Result<Stmt, String> {
    // Parse the left-hand side (could be identifier, tuple, or attribute access)
    let expr = parse_primary_expr(parser)?;

    // Check for tuple unpacking: a, b = 1, 2
    let target = if parser.match_token(&TokenKind::Comma) {
        // This is a tuple pattern for unpacking
        let mut elements = vec![expr];
        loop {
            elements.push(parse_primary_expr(parser)?);
            if !parser.match_token(&TokenKind::Comma) {
                break;
            }
        }
        let last_span = parser.previous().span;
        let merged_span = elements.first().unwrap().span().merge(last_span);
        Expr::Tuple { elements, span: merged_span }
    } else {
        expr
    };

    let mut type_ann = None;
    let is_ident = matches!(target, Expr::Ident(_, _));

    // If it's an identifier, we can have a type annotation
    if is_ident {
        if parser.match_token(&TokenKind::Colon) {
            type_ann = Some(parse_type_annotation(parser)?);
        }
    }

    if parser.match_token(&TokenKind::Eq) {
        // For the value, we need to parse a full expression including function calls
        // Also handle tuple literals on the right-hand side: a, b = 1, 2
        let value = parse_value_or_tuple(parser)?;
        let span = target.span().merge(value.span());

        if type_ann.is_some() {
            if let Expr::Ident(name, _) = target {
                return Ok(Stmt::Declare {
                    name,
                    type_ann,
                    value: Some(value),
                    mutable: true, // Variables with type annotations are mutable by default (Python semantics)
                    span,
                });
            }
        }

        Ok(Stmt::Assign { target: Box::new(target), value: Box::new(value), span })
    } else if is_augmented_assign(parser) {
        let op = get_aug_assign_op(parser);
        let value = parse_value_expr(parser)?;
        let span = target.span().merge(value.span());
        if type_ann.is_some() {
            return Err("Cannot use type annotation with augmented assignment".to_string());
        }
        Ok(Stmt::AugAssign { target: Box::new(target), op, value: Box::new(value), span })
    } else {
        // If we parsed a type annotation but no assignment, it's just a declaration
        if let Some(ann) = type_ann {
            let expr_span = target.span();
            if let Expr::Ident(name, _) = target {
                return Ok(Stmt::Declare {
                    name,
                    type_ann: Some(ann),
                    value: None,
                    mutable: false,
                    span: expr_span,
                });
            }
        }

        // Continue parsing the rest of the expression using the Pratt parser
        // But stop at statement delimiters (Newline, Dedent, etc.)
        parser.expr_parser.set_pos(parser.pos);

        // Check if we're at a statement delimiter
        if matches!(parser.current().kind, TokenKind::Newline | TokenKind::Dedent | TokenKind::Eof)
        {
            // Just return the primary expression as a statement
            Ok(Stmt::Expr(target))
        } else {
            // Parse the full expression, carefully not eating the next line
            if parser.is_at_end() {
                return Ok(Stmt::Expr(target));
            }

            // Let parse_value_expr do the bounded checking
            parser.pos = parser.expr_parser.pos(); // sync pos before value parse
            let full_expr = parse_value_expr_with_left(parser, target)?;
            Ok(Stmt::Expr(full_expr))
        }
    }
}
pub fn parse_value_expr_with_left(
    parser: &mut StatementParser,
    left: Expr,
) -> Result<Expr, String> {
    parser.expr_parser.set_pos(parser.pos);

    if matches!(
        parser.current().kind,
        TokenKind::Newline
            | TokenKind::Dedent
            | TokenKind::Indent
            | TokenKind::Eof
            | TokenKind::RParen
            | TokenKind::Comma
            | TokenKind::Colon
    ) {
        return Ok(left);
    }

    // Just let Pratt parse it with the given pos
    let full_expr = parser
        .expr_parser
        .parse_expr_with_left(left, crate::parser::precedence::Precedence::MIN)?;
    parser.pos = parser.expr_parser.pos();
    Ok(full_expr)
}

pub fn parse_value_expr(parser: &mut StatementParser) -> Result<Expr, String> {
    // Parse a value expression (primary + postfix operators + binary operators)
    let expr = parse_primary_expr(parser)?;

    // parse_primary_expr already handles function calls, so expr should be complete
    // Now check for binary operators
    parser.expr_parser.set_pos(parser.pos);

    // Check if we're at a statement boundary
    // This includes: delimiters, keywords that start statements, or indentation changes
    if matches!(
        parser.current().kind,
        TokenKind::Newline
            | TokenKind::Dedent
            | TokenKind::Indent
            | TokenKind::Eof
            | TokenKind::RParen
            | TokenKind::Comma
    ) {
        return Ok(expr);
    }

    // Also check for keywords that start new statements
    // Note: We exclude 'Else' and 'Elif' here to allow ternary expressions
    // The ternary pattern is: <expr> if <cond> else <expr>
    // When parsing the value after '=', we need to allow 'if...else' for ternary
    if matches!(
        parser.current().kind,
        TokenKind::While
            | TokenKind::For
            | TokenKind::Def
            | TokenKind::Return
            | TokenKind::Break
            | TokenKind::Continue
            | TokenKind::Pass
            | TokenKind::Import
            | TokenKind::From
            | TokenKind::Class
            | TokenKind::Try
            | TokenKind::Except
            | TokenKind::Finally
            | TokenKind::Sync
            | TokenKind::Task
    ) {
        return Ok(expr);
    }

    // Special handling for 'if' - only treat as statement boundary if it's
    // at the start of a line (not part of a ternary)
    // For ternary: <value> if <cond> else <value>
    // The 'if' comes after a value, not at statement start
    // So we don't add 'if' to the boundary check here

    // Also check for identifiers that could start a new statement (like function calls as statements)
    // If the current token is an Ident and it's NOT followed by a postfix operator (paren/bracket/dot),
    // then it's likely the start of a new statement
    if matches!(parser.current().kind, TokenKind::Ident(_)) {
        // Peek at the next token to see if this Ident is followed by a postfix operator
        if let Some(next_token) = parser.tokens.get(parser.pos + 1) {
            let is_postfix =
                matches!(next_token.kind, TokenKind::LParen | TokenKind::LBracket | TokenKind::Dot);
            if !is_postfix {
                // This Ident is likely the start of a new statement
                return Ok(expr);
            }
        }
    }

    // Try to parse binary operators using the already parsed left side
    let full_expr = parser
        .expr_parser
        .parse_expr_with_left(expr, crate::parser::precedence::Precedence::MIN)?;
    parser.pos = parser.expr_parser.pos();
    Ok(full_expr)
}

/// Parse a value expression or tuple of values
/// Used for assignment right-hand sides: a, b = 1, 2
pub fn parse_value_or_tuple(parser: &mut StatementParser) -> Result<Expr, String> {
    let first = parse_primary_expr(parser)?;

    // Check for comma - indicates a tuple
    if parser.match_token(&TokenKind::Comma) {
        let first_span = first.span();
        let mut elements = vec![first];
        loop {
            elements.push(parse_primary_expr(parser)?);
            if !parser.match_token(&TokenKind::Comma) {
                break;
            }
        }
        let last_span = parser.previous().span;
        let merged_span = first_span.merge(last_span);
        return Ok(Expr::Tuple { elements, span: merged_span });
    }

    // Otherwise parse as a regular value expression
    parse_value_expr_with_left(parser, first)
}

pub fn parse_primary_expr(parser: &mut StatementParser) -> Result<Expr, String> {
    // Parse only primary expressions (identifiers, literals, etc.) without operators
    let token = parser.current();
    let span = token.span;

    let expr = match &token.kind {
        TokenKind::Int(n) => {
            let n = *n;
            parser.advance();
            // Check if the integer fits in i64
            if n > i64::MAX as i128 || n < i64::MIN as i128 {
                // Convert to BigInt if too large for i64
                Expr::BigInt(n.to_string(), span)
            } else {
                Expr::Int(n as i64, span)
            }
        }
        TokenKind::BigInt(s) => {
            let s = s.clone();
            parser.advance();
            Expr::BigInt(s, span)
        }
        TokenKind::Float(n) => {
            let n = *n;
            parser.advance();
            Expr::Float(n, span)
        }
        TokenKind::Str(s) => {
            let s = s.clone();
            parser.advance();
            Expr::Str(s, span)
        }
        TokenKind::FString(s) => {
            let s = s.clone();
            parser.advance();

            let mut elements = Vec::new();
            let mut current_lit = String::new();
            let mut chars = s.chars().peekable();

            while let Some(c) = chars.next() {
                if c == '{' {
                    if !current_lit.is_empty() {
                        elements.push(Expr::Str(current_lit.clone(), span));
                        current_lit.clear();
                    }

                    let mut inner_expr_str = String::new();
                    while let Some(&next_c) = chars.peek() {
                        if next_c == '}' {
                            chars.next(); // consume '}'
                            break;
                        } else {
                            inner_expr_str.push(chars.next().unwrap());
                        }
                    }

                    // Tokenize and parse inner expression
                    let mut inner_lexer = crate::lexer::Lexer::new(&inner_expr_str);
                    if let Ok(tokens) = inner_lexer.tokenize() {
                        let mut inner_parser =
                            crate::parser::expressions::PrattParser::new(&tokens);
                        if let Ok(expr) =
                            inner_parser.parse_expr(crate::parser::precedence::Precedence::MIN)
                        {
                            elements.push(expr);
                        }
                    }
                } else {
                    current_lit.push(c);
                }
            }

            if !current_lit.is_empty() {
                elements.push(Expr::Str(current_lit, span));
            }

            Expr::FString(elements, span)
        }
        TokenKind::Bytes(b) => {
            let b = b.clone();
            parser.advance();
            Expr::Bytes(b, span)
        }
        TokenKind::Bool(b) => {
            let b = *b;
            parser.advance();
            Expr::Bool(b, span)
        }
        TokenKind::True => {
            parser.advance();
            Expr::Bool(true, span)
        }
        TokenKind::False => {
            parser.advance();
            Expr::Bool(false, span)
        }
        TokenKind::None => {
            parser.advance();
            Expr::None(span)
        }
        TokenKind::Super => {
            parser.advance();
            // Parse super() call - must be followed by ()
            if parser.match_token(&TokenKind::LParen) {
                // Consume the parentheses, super() takes no arguments for now
                parser.expect(&TokenKind::RParen)?;
                Expr::Super(span)
            } else {
                // Just `super` without call - treat as identifier for now
                Expr::Ident("super".to_string(), span)
            }
        }
        TokenKind::Await => {
            parser.advance();
            let future = parse_primary_expr(parser)?;
            Expr::Await { future: Box::new(future), span }
        }
        TokenKind::Ident(name) => {
            let name = name.clone();
            parser.advance();
            // Check for attribute access or function call
            let mut expr = Expr::Ident(name, span);

            loop {
                if parser.match_token(&TokenKind::Dot) {
                    let attr = parser.expect_ident()?;
                    let attr_span = parser.previous().span;
                    expr =
                        Expr::Attribute { obj: Box::new(expr), attr, span: span.merge(attr_span) };
                } else if parser.match_token(&TokenKind::LParen) {
                    let mut args = Vec::new();
                    if !parser.match_token(&TokenKind::RParen) {
                        loop {
                            args.push(parse_expression(parser)?);
                            if !parser.match_token(&TokenKind::Comma) {
                                break;
                            }
                        }
                        parser.expect(&TokenKind::RParen)?;
                    }
                    // RParen was already consumed by match_token if it matched
                    let call_span = span.merge(parser.previous().span);
                    expr = Expr::Call { func: Box::new(expr), args, span: call_span };
                } else if parser.match_token(&TokenKind::LBracket) {
                    // Parse slice or index
                    // Look ahead to check for ':' which indicates a slice
                    // We need to peek past the first expression to see if there's a colon
                    let is_slice = is_slice_pattern(parser);

                    if is_slice {
                        // Parse slice: [:], [start:], [:end], [start:end], [::step], etc.
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
                        expr =
                            Expr::Slice { obj: Box::new(expr), start, end, step, span: index_span };
                    } else {
                        // Regular indexing
                        let index = parse_expression(parser)?;
                        parser.expect(&TokenKind::RBracket)?;
                        let index_span = span.merge(parser.previous().span);
                        expr = Expr::Index {
                            obj: Box::new(expr),
                            index: Box::new(index),
                            span: index_span,
                        };
                    }
                } else {
                    break;
                }
            }

            expr
        }
        // Handle send/recv as identifiers when used as function names
        TokenKind::Send => {
            parser.advance();
            // Treat as identifier "send" for function call syntax
            let mut expr = Expr::Ident("send".to_string(), span);

            loop {
                if parser.match_token(&TokenKind::Dot) {
                    let attr = parser.expect_ident()?;
                    let attr_span = parser.previous().span;
                    expr =
                        Expr::Attribute { obj: Box::new(expr), attr, span: span.merge(attr_span) };
                } else if parser.match_token(&TokenKind::LParen) {
                    let mut args = Vec::new();
                    if !parser.match_token(&TokenKind::RParen) {
                        loop {
                            args.push(parse_expression(parser)?);
                            if !parser.match_token(&TokenKind::Comma) {
                                break;
                            }
                        }
                        parser.expect(&TokenKind::RParen)?;
                    }
                    // RParen was already consumed by match_token if it matched
                    let call_span = span.merge(parser.previous().span);
                    expr = Expr::Call { func: Box::new(expr), args, span: call_span };
                } else if parser.match_token(&TokenKind::LBracket) {
                    // Parse slice or index
                    // Look ahead to check for ':' which indicates a slice
                    // We need to peek past the first expression to see if there's a colon
                    let is_slice = is_slice_pattern(parser);

                    if is_slice {
                        // Parse slice: [:], [start:], [:end], [start:end], [::step], etc.
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
                        expr =
                            Expr::Slice { obj: Box::new(expr), start, end, step, span: index_span };
                    } else {
                        // Regular indexing
                        let index = parse_expression(parser)?;
                        parser.expect(&TokenKind::RBracket)?;
                        let index_span = span.merge(parser.previous().span);
                        expr = Expr::Index {
                            obj: Box::new(expr),
                            index: Box::new(index),
                            span: index_span,
                        };
                    }
                } else {
                    break;
                }
            }

            expr
        }
        TokenKind::Recv => {
            parser.advance();
            // Treat as identifier "recv" for function call syntax
            let mut expr = Expr::Ident("recv".to_string(), span);

            loop {
                if parser.match_token(&TokenKind::Dot) {
                    let attr = parser.expect_ident()?;
                    let attr_span = parser.previous().span;
                    expr =
                        Expr::Attribute { obj: Box::new(expr), attr, span: span.merge(attr_span) };
                } else if parser.match_token(&TokenKind::LParen) {
                    let mut args = Vec::new();
                    if !parser.match_token(&TokenKind::RParen) {
                        loop {
                            args.push(parse_expression(parser)?);
                            if !parser.match_token(&TokenKind::Comma) {
                                break;
                            }
                        }
                        parser.expect(&TokenKind::RParen)?;
                    }
                    // RParen was already consumed by match_token if it matched
                    let call_span = span.merge(parser.previous().span);
                    expr = Expr::Call { func: Box::new(expr), args, span: call_span };
                } else if parser.match_token(&TokenKind::LBracket) {
                    // Parse slice or index
                    // Look ahead to check for ':' which indicates a slice
                    // We need to peek past the first expression to see if there's a colon
                    let is_slice = is_slice_pattern(parser);

                    if is_slice {
                        // Parse slice: [:], [start:], [:end], [start:end], [::step], etc.
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
                        expr =
                            Expr::Slice { obj: Box::new(expr), start, end, step, span: index_span };
                    } else {
                        // Regular indexing
                        let index = parse_expression(parser)?;
                        parser.expect(&TokenKind::RBracket)?;
                        let index_span = span.merge(parser.previous().span);
                        expr = Expr::Index {
                            obj: Box::new(expr),
                            index: Box::new(index),
                            span: index_span,
                        };
                    }
                } else {
                    break;
                }
            }

            expr
        }

        TokenKind::LParen => {
            parser.advance();

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
            expr
        }
        TokenKind::LBracket => {
            parser.advance();
            let mut elements = Vec::new();
            let mut size: Option<usize> = None;

            // Check for empty list without consuming the RBracket
            if !matches!(parser.current().kind, TokenKind::RBracket) {
                // Parse first element
                let first_elem = parse_expression(parser)?;

                // Check for list comprehension: [expr for var in iter]
                if matches!(parser.current().kind, TokenKind::For) {
                    // This is a list comprehension
                    parser.advance(); // consume 'for'

                    // Parse the variable name
                    let var = if let TokenKind::Ident(name) = &parser.current().kind {
                        let name = name.clone();
                        parser.advance();
                        name
                    } else {
                        return Err("Expected variable name in list comprehension".to_string());
                    };

                    // Expect 'in' keyword
                    parser.expect(&TokenKind::In)?;

                    // Parse the iterable
                    let iter = parse_expression(parser)?;

                    // Expect closing bracket
                    parser.expect(&TokenKind::RBracket)?;

                    let list_span = span.merge(parser.previous().span);

                    return Ok(Expr::ListComprehension {
                        element: Box::new(first_elem),
                        var,
                        iter: Box::new(iter),
                        span: list_span,
                    });
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
                                return Err(format!("Array size must be a positive usize: {}", n));
                            }
                            size = Some(*n as usize);
                            parser.advance();
                        }
                        _ => {
                            return Err(format!(
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
                Expr::Array { elements, size, span: list_span }
            } else {
                Expr::List { elements, span: list_span }
            }
        }
        TokenKind::LBrace => {
            parser.advance();
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

            Expr::Dict { pairs, span: merged_span }
        }
        TokenKind::Minus => {
            parser.advance();
            let operand = parse_primary_expr(parser)?;
            let neg_span = span.merge(operand.span());
            Expr::UnaryOp {
                op: crate::ast::UnaryOp::Neg,
                operand: Box::new(operand),
                span: neg_span,
            }
        }
        TokenKind::Not => {
            parser.advance();
            let operand = parse_primary_expr(parser)?;
            let not_span = span.merge(operand.span());
            Expr::UnaryOp {
                op: crate::ast::UnaryOp::Not,
                operand: Box::new(operand),
                span: not_span,
            }
        }
        TokenKind::Tilde => {
            parser.advance();
            let operand = parse_primary_expr(parser)?;
            let tilde_span = span.merge(operand.span());
            Expr::UnaryOp {
                op: crate::ast::UnaryOp::Invert,
                operand: Box::new(operand),
                span: tilde_span,
            }
        }
        TokenKind::Plus => {
            parser.advance();
            let operand = parse_primary_expr(parser)?;
            let plus_span = span.merge(operand.span());
            Expr::UnaryOp {
                op: crate::ast::UnaryOp::Pos,
                operand: Box::new(operand),
                span: plus_span,
            }
        }
        TokenKind::PlusPlus => {
            parser.advance();
            let operand = parse_primary_expr(parser)?;
            let inc_span = span.merge(operand.span());
            Expr::UnaryOp {
                op: crate::ast::UnaryOp::PreIncrement,
                operand: Box::new(operand),
                span: inc_span,
            }
        }
        TokenKind::MinusMinus => {
            parser.advance();
            let operand = parse_primary_expr(parser)?;
            let dec_span = span.merge(operand.span());
            Expr::UnaryOp {
                op: crate::ast::UnaryOp::PreDecrement,
                operand: Box::new(operand),
                span: dec_span,
            }
        }
        TokenKind::Lambda | TokenKind::Fn => {
            parser.advance();
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
                        return Err("Expected parameter name in lambda".to_string());
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
            Expr::Lambda { params, body: Box::new(body), span: merged_span }
        }
        _ => return Err(format!("Unexpected token in expression: {:?}", token.kind)),
    };

    Ok(expr)
}
pub fn is_augmented_assign(parser: &mut StatementParser) -> bool {
    matches!(
        parser.current().kind,
        TokenKind::PlusEq
            | TokenKind::MinusEq
            | TokenKind::StarEq
            | TokenKind::SlashEq
            | TokenKind::PercentEq
            | TokenKind::DoubleStarEq
            | TokenKind::DoubleSlashEq
    )
}

pub fn get_aug_assign_op(parser: &mut StatementParser) -> BinOp {
    let op = match &parser.current().kind {
        TokenKind::PlusEq => BinOp::Add,
        TokenKind::MinusEq => BinOp::Sub,
        TokenKind::StarEq => BinOp::Mul,
        TokenKind::SlashEq => BinOp::Div,
        TokenKind::PercentEq => BinOp::Mod,
        TokenKind::DoubleStarEq => BinOp::Pow,
        TokenKind::DoubleSlashEq => BinOp::FloorDiv,
        _ => BinOp::Add, // default
    };
    parser.advance(); // skip the augmented assignment token

    op
}
/// Check if the bracket contents match a slice pattern (contains ':' before ']')
/// Handles: [:], [start:], [:end], [start:end], [::step], etc.
pub fn is_slice_pattern(parser: &mut StatementParser) -> bool {
    // Look ahead through tokens to find ':' or ']'
    // We need to handle nested brackets/parens
    let mut pos = parser.pos;
    let mut bracket_depth = 1;

    while pos < parser.tokens.len() {
        match &parser.tokens[pos].kind {
            TokenKind::Colon if bracket_depth == 1 => return true,
            TokenKind::RBracket => {
                bracket_depth -= 1;
                if bracket_depth == 0 {
                    return false;
                }
            }
            TokenKind::LBracket | TokenKind::LParen => {
                bracket_depth += 1;
            }
            TokenKind::Eof => return false,
            _ => {}
        }
        pos += 1;
    }

    false
}

/// Parse type alias: type Name = Type
pub fn parse_type_alias(parser: &mut StatementParser) -> Result<Stmt, String> {
    let span = parser.current().span;
    parser.expect(&TokenKind::Type)?;

    let name = parser.expect_ident()?;

    parser.expect(&TokenKind::Eq)?;

    let type_def = parse_type_annotation(parser)?;

    Ok(Stmt::TypeAlias { name, type_def, span })
}
