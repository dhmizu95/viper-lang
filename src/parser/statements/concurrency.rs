use super::*;
use crate::ast::{Expr, Stmt};
use crate::lexer::TokenKind;
use crate::utils::Span;

pub fn parse_sync_block(parser: &mut StatementParser) -> crate::error::Result<Stmt> {
    let span = parser.current().span;
    parser.expect(&TokenKind::Sync)?;
    parser.expect(&TokenKind::Colon)?;
    let body = parse_block(parser)?;

    Ok(Stmt::Sync { body, span })
}

pub fn parse_task_spawn(parser: &mut StatementParser) -> crate::error::Result<Stmt> {
    let span = parser.current().span;
    parser.expect(&TokenKind::Task)?;
    let call = parse_expression(parser)?;

    Ok(Stmt::Task { call, span })
}
/// Transform concurrency builtin calls into appropriate AST nodes
pub fn transform_concurrency_call(
    _parser: &mut StatementParser,
    func: &Expr,
    args: Vec<Expr>,
    keywords: Vec<(String, Expr)>,
    span: Span,
) -> Option<Stmt> {
    if !keywords.is_empty() {
        return None;
    }
    if let Expr::Ident(name, _) = func {
        match name.as_str() {
            "chan" => {
                if args.len() == 1 {
                    return Some(Stmt::Chan { size: args.into_iter().next().unwrap(), span });
                }
            }
            "send" => {
                if args.len() == 2 {
                    let mut args_iter = args.into_iter();
                    return Some(Stmt::Send {
                        chan: Box::new(args_iter.next().unwrap()),
                        value: Box::new(args_iter.next().unwrap()),
                        span,
                    });
                }
            }
            "recv" => {
                if args.len() == 1 {
                    return Some(Stmt::Recv {
                        chan: Box::new(args.into_iter().next().unwrap()),
                        span,
                    });
                }
            }
            "WaitGroup" => {
                if args.is_empty() {
                    return Some(Stmt::WaitGroup { span });
                }
            }
            "add" => {
                if args.len() == 2 {
                    let mut args_iter = args.into_iter();
                    return Some(Stmt::WgAdd {
                        wg: Box::new(args_iter.next().unwrap()),
                        n: Box::new(args_iter.next().unwrap()),
                        span,
                    });
                }
            }
            "done" => {
                if args.len() == 1 {
                    return Some(Stmt::WgDone {
                        wg: Box::new(args.into_iter().next().unwrap()),
                        span,
                    });
                }
            }
            "wait" => {
                if args.len() == 1 {
                    return Some(Stmt::WgWait {
                        wg: Box::new(args.into_iter().next().unwrap()),
                        span,
                    });
                }
            }
            _ => {}
        }
    }
    None
}
