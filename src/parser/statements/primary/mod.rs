//! Primary expression parsing module
//!
//! This module handles parsing of primary expressions including:
//! - Literals (int, float, string, bool, None, etc.)
//! - Container literals (list, dict, tuple, array)
//! - Comprehensions
//! - Call expressions and attribute access
//! - Unary operators
//! - Special forms (lambda, type alias)

pub mod calls;
pub mod comprehensions;
pub mod containers;
pub mod literals;
pub mod operators;
pub mod special;

use super::*;
use crate::ast::{BinOp, Expr, Stmt};
use crate::lexer::TokenKind;

pub use calls::*;
pub use comprehensions::*;
pub use containers::*;
pub use literals::*;
pub use operators::*;
pub use special::*;

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

        Ok(Stmt::Assign {
            target: Box::new(target),
            value: Box::new(value),
            span,
        })
    } else if is_augmented_assign(parser) {
        let op = get_aug_assign_op(parser);
        let value = parse_value_expr(parser)?;
        let span = target.span().merge(value.span());
        if type_ann.is_some() {
            return Err("Cannot use type annotation with augmented assignment".to_string());
        }
        Ok(Stmt::AugAssign {
            target: Box::new(target),
            op,
            value: Box::new(value),
            span,
        })
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
        if matches!(
            parser.current().kind,
            TokenKind::Newline | TokenKind::Dedent | TokenKind::Eof
        ) {
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

pub fn parse_value_expr_with_left(parser: &mut StatementParser, left: Expr) -> Result<Expr, String> {
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
            literals::parse_int_literal(parser, n, span)
        }
        TokenKind::BigInt(s) => {
            let s = s.clone();
            parser.advance();
            literals::parse_bigint_literal(parser, s, span)
        }
        TokenKind::Float(n) => {
            let n = *n;
            parser.advance();
            literals::parse_float_literal(parser, n, span)
        }
        TokenKind::Str(s) => {
            let s = s.clone();
            parser.advance();
            literals::parse_str_literal(parser, s, span)
        }
        TokenKind::FString(s) => {
            let s = s.clone();
            parser.advance();
            literals::parse_fstring_literal(parser, s, span)
        }
        TokenKind::Bytes(b) => {
            let b = b.clone();
            parser.advance();
            literals::parse_bytes_literal(parser, b, span)
        }
        TokenKind::Bool(b) => {
            let b = *b;
            parser.advance();
            literals::parse_bool_literal(parser, b, span)
        }
        TokenKind::True => {
            parser.advance();
            literals::parse_bool_literal(parser, true, span)
        }
        TokenKind::False => {
            parser.advance();
            literals::parse_bool_literal(parser, false, span)
        }
        TokenKind::None => {
            parser.advance();
            literals::parse_none_literal(parser, span)
        }
        TokenKind::Super => {
            parser.advance();
            literals::parse_super_expr(parser, span)
        }
        TokenKind::Await => {
            parser.advance();
            return literals::parse_await_expr(parser, span);
        }
        TokenKind::Ident(name) => {
            let name = name.clone();
            return calls::parse_ident_expr(parser, name, span);
        }
        // Handle send/recv as identifiers when used as function names
        TokenKind::Send => {
            return calls::parse_send_expr(parser, span);
        }
        TokenKind::Recv => {
            return calls::parse_recv_expr(parser, span);
        }
        TokenKind::LParen => {
            return containers::parse_tuple_literal(parser, span);
        }
        TokenKind::LBracket => {
            return containers::parse_list_or_array(parser, span);
        }
        TokenKind::LBrace => {
            return containers::parse_dict_literal(parser, span);
        }
        TokenKind::Minus => {
            return operators::parse_neg_expr(parser, span);
        }
        TokenKind::Not => {
            return operators::parse_not_expr(parser, span);
        }
        TokenKind::Tilde => {
            return operators::parse_invert_expr(parser, span);
        }
        TokenKind::Plus => {
            return operators::parse_pos_expr(parser, span);
        }
        TokenKind::PlusPlus => {
            return operators::parse_pre_inc_expr(parser, span);
        }
        TokenKind::MinusMinus => {
            return operators::parse_pre_dec_expr(parser, span);
        }
        TokenKind::Lambda | TokenKind::Fn => {
            parser.advance();
            return special::parse_lambda_expr(parser, span);
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
