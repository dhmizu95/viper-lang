use super::*;
use crate::ast::{Param, Stmt, Type};
use crate::lexer::TokenKind;

pub fn parse_function_def(parser: &mut StatementParser) -> Result<Stmt, String> {
    let start_span = parser.current().span;
    parser.expect(&TokenKind::Def)?;

    let name_token = parser.expect_ident()?;

    // Parse generic type parameters: [T, U, ...]
    let mut type_params = Vec::new();
    if parser.match_token(&TokenKind::LBracket) {
        loop {
            if matches!(parser.current().kind, TokenKind::Ident(_)) {
                type_params.push(parser.expect_ident()?);
            } else {
                break;
            }
            if !parser.match_token(&TokenKind::Comma) {
                break;
            }
        }
        parser.expect(&TokenKind::RBracket)?;
    }

    parser.expect(&TokenKind::LParen)?;

    let mut params = Vec::new();
    if !matches!(parser.current().kind, TokenKind::RParen) {
        loop {
            let param = parse_param(parser)?;
            params.push(param);
            if !parser.match_token(&TokenKind::Comma) {
                break;
            }
        }
    }
    parser.expect(&TokenKind::RParen)?;

    let return_type = if parser.match_token(&TokenKind::Arrow) {
        Some(parse_type_annotation(parser)?)
    } else {
        None
    };

    parser.expect(&TokenKind::Colon)?;
    let body = parse_block(parser)?;

    let span = start_span.merge(parser.previous().span);

    Ok(Stmt::Function {
        name: name_token,
        type_params,
        params,
        return_type,
        body,
        span,
        is_async: false,
    })
}

pub fn parse_extern_decl(parser: &mut StatementParser) -> Result<Stmt, String> {
    let start_span = parser.current().span;
    parser.expect(&TokenKind::Extern)?;

    if let TokenKind::Str(_) = &parser.current().kind {
        parser.advance();
    }

    parser.expect(&TokenKind::Def)?;

    let name_token = parser.expect_ident()?;
    parser.expect(&TokenKind::LParen)?;

    let mut params = Vec::new();
    if !matches!(parser.current().kind, TokenKind::RParen) {
        loop {
            let param = parse_param(parser)?;
            params.push(param);
            if !parser.match_token(&TokenKind::Comma) {
                break;
            }
        }
    }
    parser.expect(&TokenKind::RParen)?;

    let return_type = if parser.match_token(&TokenKind::Arrow) {
        Some(parse_type_annotation(parser)?)
    } else {
        None
    };

    let span = start_span.merge(parser.previous().span);

    Ok(Stmt::Extern { name: name_token, params, return_type, span })
}
pub fn parse_async_function_def(parser: &mut StatementParser) -> Result<Stmt, String> {
    let start_span = parser.current().span;
    parser.expect(&TokenKind::Async)?;
    parser.expect(&TokenKind::Def)?;

    let name_token = parser.expect_ident()?;
    parser.expect(&TokenKind::LParen)?;

    let mut params = Vec::new();
    if !matches!(parser.current().kind, TokenKind::RParen) {
        loop {
            let param = parse_param(parser)?;
            params.push(param);
            if !parser.match_token(&TokenKind::Comma) {
                break;
            }
        }
    }
    parser.expect(&TokenKind::RParen)?;

    let return_type = if parser.match_token(&TokenKind::Arrow) {
        Some(parse_type_annotation(parser)?)
    } else {
        None
    };

    parser.expect(&TokenKind::Colon)?;
    let body = parse_block(parser)?;

    let span = start_span.merge(parser.previous().span);

    Ok(Stmt::Function {
        name: name_token,
        type_params: vec![],
        params,
        return_type,
        body,
        span,
        is_async: true,
    })
}

pub fn parse_param(parser: &mut StatementParser) -> Result<Param, String> {
    let span = parser.current().span;
    let name = parser.expect_ident()?;

    let type_ann = if parser.match_token(&TokenKind::Colon) {
        Some(parse_type_annotation(parser)?)
    } else {
        None
    };

    let default =
        if parser.match_token(&TokenKind::Eq) { Some(parse_expression(parser)?) } else { None };

    Ok(Param { name, type_ann, default, span })
}
pub fn parse_type_annotation(parser: &mut StatementParser) -> Result<Type, String> {
    // Handle array type: [type; size]
    if parser.match_token(&TokenKind::LBracket) {
        let elem_type = parse_type_annotation(parser)?;
        parser.expect(&TokenKind::Semi)?;
        let size_token = parser.current();
        let size = match &size_token.kind {
            TokenKind::Int(n) => {
                if *n < 0 || *n > usize::MAX as i128 {
                    return Err(format!("Array size must be a positive usize: {}", n));
                }
                *n as usize
            }
            TokenKind::BigInt(_) => {
                return Err("Array size cannot be a BigInt".to_string());
            }
            _ => {
                return Err(format!(
                    "Expected integer size for array type, found {:?}",
                    size_token.kind
                ))
            }
        };
        parser.advance();
        parser.expect(&TokenKind::RBracket)?;
        return Ok(Type::Array(Box::new(elem_type), size));
    }

    // Handle tuple type: tuple[type1, type2, ...]
    if parser.match_token(&TokenKind::Tuple) {
        parser.expect(&TokenKind::LBracket)?;
        let mut types = Vec::new();
        if !parser.match_token(&TokenKind::RBracket) {
            loop {
                types.push(parse_type_annotation(parser)?);
                if !parser.match_token(&TokenKind::Comma) {
                    break;
                }
            }
        }
        parser.expect(&TokenKind::RBracket)?;
        return Ok(Type::Tuple(types));
    }

    // Handle Optional type: Optional[type]
    if parser.match_token(&TokenKind::Optional) {
        parser.expect(&TokenKind::LBracket)?;
        let inner_type = parse_type_annotation(parser)?;
        parser.expect(&TokenKind::RBracket)?;
        return Ok(Type::Optional(Box::new(inner_type)));
    }

    let token = parser.current();
    let ty = match &token.kind {
        TokenKind::Ident(name) => match name.as_str() {
            "i8" => Type::I8,
            "i16" => Type::I16,
            "i32" => Type::I32,
            "i64" => Type::I64,
            "f32" => Type::F32,
            "f64" => Type::F64,
            "bool" => Type::Bool,
            "str" => Type::Str,
            "Chan" | "chan" => {
                // Handle Chan[T] syntax
                parser.advance();
                if !parser.match_token(&TokenKind::LBracket) {
                    return Err("Expected '[' after Chan".to_string());
                }
                let elem_type = parse_type_annotation(parser)?;
                parser.expect(&TokenKind::RBracket)?;
                return Ok(Type::Chan(Box::new(elem_type)));
            }
            "WaitGroup" => {
                parser.advance();
                return Ok(Type::WaitGroup);
            }
            "Callable" => {
                parser.advance();
                return Ok(Type::Fn(vec![], Box::new(Type::I64)));
            }
            _ => Type::Var(name.clone()),
        },
        TokenKind::None | TokenKind::Void => Type::None,
        _ => return Err(format!("Expected type name, found {:?}", token.kind)),
    };
    parser.advance();

    // Handle Optional suffix: T?
    if parser.match_token(&TokenKind::Question) {
        return Ok(Type::Optional(Box::new(ty)));
    }

    Ok(ty)
}
pub fn parse_class_def(parser: &mut StatementParser) -> Result<Stmt, String> {
    let start_span = parser.current().span;
    parser.expect(&TokenKind::Class)?;

    let name = parser.expect_ident()?;

    let mut bases = Vec::new();
    if parser.match_token(&TokenKind::LParen) {
        if !parser.match_token(&TokenKind::RParen) {
            loop {
                bases.push(parse_expression(parser)?);
                if !parser.match_token(&TokenKind::Comma) {
                    break;
                }
            }
        }
        parser.expect(&TokenKind::RParen)?;
    }

    parser.expect(&TokenKind::Colon)?;
    let body = parse_block(parser)?;

    let span = start_span.merge(parser.previous().span);

    Ok(Stmt::Class { name, bases, body, span })
}
pub fn parse_struct_def(parser: &mut StatementParser) -> Result<Stmt, String> {
    let start_span = parser.current().span;
    parser.expect(&TokenKind::Struct)?;

    let name = parser.expect_ident()?;

    parser.expect(&TokenKind::Colon)?;

    let mut fields = Vec::new();
    if parser.match_token(&TokenKind::Indent) {
        loop {
            if matches!(parser.current().kind, TokenKind::Dedent) {
                parser.advance();
                break;
            }
            if parser.is_at_end() {
                break;
            }
            if parser.match_token(&TokenKind::Newline) {
                continue;
            }

            let field_name = parser.expect_ident()?;
            parser.expect(&TokenKind::Colon)?;
            let field_type = parse_type_annotation(parser)?;

            fields.push((field_name, field_type));

            parser.match_token(&TokenKind::Newline);
        }
    }

    let span = start_span.merge(parser.previous().span);

    Ok(Stmt::Struct { name, fields, span })
}
