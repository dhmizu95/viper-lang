use super::super::*;
use crate::ast::Expr;
use crate::lexer::TokenKind;
use crate::utils::Span;

/// Parse integer literal (Int or BigInt)
pub fn parse_int_literal(_parser: &mut StatementParser, n: i128, span: Span) -> Expr {
    // Check if the integer fits in i64
    if n > i64::MAX as i128 || n < i64::MIN as i128 {
        // Convert to BigInt if too large for i64
        Expr::BigInt(n.to_string(), span)
    } else {
        Expr::Int(n as i64, span)
    }
}

/// Parse BigInt literal
pub fn parse_bigint_literal(_parser: &mut StatementParser, s: String, span: Span) -> Expr {
    Expr::BigInt(s, span)
}

/// Parse float literal
pub fn parse_float_literal(_parser: &mut StatementParser, n: f64, span: Span) -> Expr {
    Expr::Float(n, span)
}

/// Parse string literal
pub fn parse_str_literal(_parser: &mut StatementParser, s: String, span: Span) -> Expr {
    Expr::Str(s, span)
}

/// Parse f-string literal with interpolated expressions
pub fn parse_fstring_literal(_parser: &mut StatementParser, s: String, span: Span) -> Expr {
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
                let mut inner_parser = crate::parser::expressions::PrattParser::new(&tokens);
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

/// Parse bytes literal
pub fn parse_bytes_literal(_parser: &mut StatementParser, b: Vec<u8>, span: Span) -> Expr {
    Expr::Bytes(b, span)
}

/// Parse boolean literal
pub fn parse_bool_literal(_parser: &mut StatementParser, b: bool, span: Span) -> Expr {
    Expr::Bool(b, span)
}

/// Parse None literal
pub fn parse_none_literal(_parser: &mut StatementParser, span: Span) -> Expr {
    Expr::None(span)
}

/// Parse super expression
pub fn parse_super_expr(parser: &mut StatementParser, span: Span) -> Expr {
    // Parse super() call - must be followed by ()
    if parser.match_token(&TokenKind::LParen) {
        // Consume the parentheses, super() takes no arguments for now
        parser.expect(&TokenKind::RParen).unwrap();
        Expr::Super(span)
    } else {
        // Just `super` without call - treat as identifier for now
        Expr::Ident("super".to_string(), span)
    }
}

/// Parse await expression
pub fn parse_await_expr(parser: &mut StatementParser, span: Span) -> crate::error::Result<Expr> {
    let future = super::parse_primary_expr(parser)?;
    Ok(Expr::Await { future: Box::new(future), span })
}
