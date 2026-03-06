use super::super::*;
use crate::ast::{Expr, UnaryOp};
use crate::utils::Span;

/// Parse unary minus expression: -expr
pub fn parse_neg_expr(parser: &mut StatementParser, span: Span) -> Result<Expr, String> {
    parser.advance();
    let operand = super::parse_primary_expr(parser)?;
    let neg_span = span.merge(operand.span());
    Ok(Expr::UnaryOp {
        op: UnaryOp::Neg,
        operand: Box::new(operand),
        span: neg_span,
    })
}

/// Parse unary not expression: not expr
pub fn parse_not_expr(parser: &mut StatementParser, span: Span) -> Result<Expr, String> {
    parser.advance();
    let operand = super::parse_primary_expr(parser)?;
    let not_span = span.merge(operand.span());
    Ok(Expr::UnaryOp {
        op: UnaryOp::Not,
        operand: Box::new(operand),
        span: not_span,
    })
}

/// Parse unary invert expression: ~expr
pub fn parse_invert_expr(parser: &mut StatementParser, span: Span) -> Result<Expr, String> {
    parser.advance();
    let operand = super::parse_primary_expr(parser)?;
    let tilde_span = span.merge(operand.span());
    Ok(Expr::UnaryOp {
        op: UnaryOp::Invert,
        operand: Box::new(operand),
        span: tilde_span,
    })
}

/// Parse unary plus expression: +expr
pub fn parse_pos_expr(parser: &mut StatementParser, span: Span) -> Result<Expr, String> {
    parser.advance();
    let operand = super::parse_primary_expr(parser)?;
    let plus_span = span.merge(operand.span());
    Ok(Expr::UnaryOp {
        op: UnaryOp::Pos,
        operand: Box::new(operand),
        span: plus_span,
    })
}

/// Parse pre-increment expression: ++expr
pub fn parse_pre_inc_expr(parser: &mut StatementParser, span: Span) -> Result<Expr, String> {
    parser.advance();
    let operand = super::parse_primary_expr(parser)?;
    let inc_span = span.merge(operand.span());
    Ok(Expr::UnaryOp {
        op: UnaryOp::PreIncrement,
        operand: Box::new(operand),
        span: inc_span,
    })
}

/// Parse pre-decrement expression: --expr
pub fn parse_pre_dec_expr(parser: &mut StatementParser, span: Span) -> Result<Expr, String> {
    parser.advance();
    let operand = super::parse_primary_expr(parser)?;
    let dec_span = span.merge(operand.span());
    Ok(Expr::UnaryOp {
        op: UnaryOp::PreDecrement,
        operand: Box::new(operand),
        span: dec_span,
    })
}
