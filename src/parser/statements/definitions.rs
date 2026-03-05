use super::*;
use crate::ast::{Decorator, Param, Stmt, Type};
use crate::lexer::TokenKind;

/// Parse decorators before a function or class definition
pub fn parse_decorators(parser: &mut StatementParser) -> Result<Vec<Decorator>, String> {
    let mut decorators = Vec::new();

    while parser.match_token(&TokenKind::At) {
        let start_span = parser.previous().span;

        // Parse decorator name (may include dots like @property.setter)
        let mut name = parser.expect_ident()?;
        
        // Check for dotted name (e.g., property.setter)
        while parser.match_token(&TokenKind::Dot) {
            let suffix = parser.expect_ident()?;
            name.push('.');
            name.push_str(&suffix);
        }

        // Parse optional arguments
        let mut args = Vec::new();
        let mut keywords = Vec::new();

        if parser.match_token(&TokenKind::LParen) {
            if !parser.match_token(&TokenKind::RParen) {
                loop {
                    // Check if this is a keyword argument
                    if parser.peek().map_or(false, |t| matches!(t.kind, TokenKind::Eq)) {
                        let keyword_name = parser.expect_ident()?;
                        parser.expect(&TokenKind::Eq)?;
                        let keyword_value = parse_expression(parser)?;
                        keywords.push((keyword_name, keyword_value));
                    } else {
                        args.push(parse_expression(parser)?);
                    }

                    if !parser.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
            }
            parser.expect(&TokenKind::RParen)?;
        }

        let span = start_span.merge(parser.previous().span);
        decorators.push(Decorator { name, args, keywords, span });

        // Skip newlines after decorator until we hit non-newline token
        while parser.match_token(&TokenKind::Newline) {
            parser.advance();  // Actually consume the newline
        }
    }

    Ok(decorators)
}

pub fn parse_function_def(parser: &mut StatementParser) -> Result<Stmt, String> {
    // Parse decorators first
    let decorators = parse_decorators(parser)?;
    
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
        decorators,
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
    // Parse decorators first
    let decorators = parse_decorators(parser)?;
    
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
        decorators,
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
    // Parse the base type first
    let base_type = parse_base_type(parser)?;

    // Check for union type: T | U | V ...
    if parser.match_token(&TokenKind::Pipe) {
        let mut variants = vec![base_type];
        loop {
            let variant = parse_base_type(parser)?;
            variants.push(variant);
            
            if !parser.match_token(&TokenKind::Pipe) {
                break;
            }
        }
        return Ok(Type::Union(variants));
    }

    Ok(base_type)
}

/// Parse a single type (without union handling)
fn parse_base_type(parser: &mut StatementParser) -> Result<Type, String> {
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

    // Handle Result type: Result[Ok, Err]
    if parser.match_token(&TokenKind::Result) {
        // match_token already advances past Result
        parser.expect(&TokenKind::LBracket)?;
        let ok_type = parse_type_annotation(parser)?;
        parser.expect(&TokenKind::Comma)?;
        let err_type = parse_type_annotation(parser)?;
        parser.expect(&TokenKind::RBracket)?;
        return Ok(Type::Result(Box::new(ok_type), Box::new(err_type)));
    }

    let token = parser.current();
    let ty = match &token.kind {
        TokenKind::Ident(name) => match name.as_str() {
            // Generic types with parameters
            "list" => {
                // list[T] syntax
                parser.advance();
                if !parser.match_token(&TokenKind::LBracket) {
                    return Err("Expected '[' after list".to_string());
                }
                let elem_type = parse_type_annotation(parser)?;
                parser.expect(&TokenKind::RBracket)?;
                return Ok(Type::List(Box::new(elem_type)));
            }
            "dict" => {
                // dict[K, V] syntax
                parser.advance();
                if !parser.match_token(&TokenKind::LBracket) {
                    return Err("Expected '[' after dict".to_string());
                }
                let key_type = parse_type_annotation(parser)?;
                parser.expect(&TokenKind::Comma)?;
                let value_type = parse_type_annotation(parser)?;
                parser.expect(&TokenKind::RBracket)?;
                return Ok(Type::Dict(Box::new(key_type), Box::new(value_type)));
            }
            "tuple" => {
                // tuple[T1, T2, ...] syntax
                parser.advance();
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
            // Python-style aliases
            "int" => Type::Int,   // Python int -> auto-promoting tagged integer
            "float" => Type::F64, // Python float -> Viper f64
            // Viper native types
            "i8" => Type::I8,
            "i16" => Type::I16,
            "i32" => Type::I32,
            "f32" => Type::F32,
            "bool" => Type::Bool,
            "str" => Type::Str,
            "WaitGroup" => {
                parser.advance();
                return Ok(Type::WaitGroup);
            }
            "Callable" => {
                parser.advance();
                return Ok(Type::Fn(vec![], Box::new(Type::I64)));
            }
            // Generic type application for user-defined types: MyType[T, U]
            _ => {
                let type_name = name.clone();
                parser.advance();
                
                // Check for generic application: Type[T, U, ...]
                if parser.match_token(&TokenKind::LBracket) {
                    let mut type_args = Vec::new();
                    loop {
                        type_args.push(parse_type_annotation(parser)?);
                        if !parser.match_token(&TokenKind::Comma) {
                            break;
                        }
                    }
                    parser.expect(&TokenKind::RBracket)?;
                    return Ok(Type::GenericApp {
                        name: type_name,
                        type_args,
                    });
                }
                
                // Simple type variable or named type
                Type::Var(type_name)
            }
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
    // Parse decorators first
    let decorators = parse_decorators(parser)?;
    
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

    // Extract fields and methods from the class body
    let mut fields = Vec::new();
    let mut methods = Vec::new();
    
    for stmt in &body {
        match stmt {
            Stmt::Assign { target, .. } => {
                // Class variable
                if let crate::ast::Expr::Ident(field_name, _) = target.as_ref() {
                    fields.push((field_name.clone(), None, true));
                }
            }
            Stmt::Declare { name: field_name, type_ann, .. } => {
                // Typed class variable
                fields.push((field_name.clone(), type_ann.clone(), true));
            }
            Stmt::Function { name: method_name, .. } => {
                methods.push(method_name.clone());
            }
            _ => {}
        }
    }

    Ok(Stmt::Class { name, bases, body, span, decorators, fields, methods })
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
