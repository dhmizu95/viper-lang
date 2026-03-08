use super::*;
use crate::ast::{Expr, Stmt};
use crate::lexer::{Token, TokenKind};
use crate::parser::expressions::PrattParser;

/// Statement parser for Viper
pub struct StatementParser<'a> {
    pub(crate) tokens: &'a [Token],
    pub(crate) pos: usize,
    pub(crate) expr_parser: PrattParser<'a>,
}

impl<'a> StatementParser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, pos: 0, expr_parser: PrattParser::new(tokens) }
    }

    pub(crate) fn current(&self) -> &Token {
        &self.tokens[self.pos]
    }

    pub(crate) fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos + 1)
    }

    pub(crate) fn previous(&self) -> &Token {
        &self.tokens[self.pos - 1]
    }

    pub(crate) fn advance(&mut self) {
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
    }

    pub(crate) fn match_token(&mut self, kind: &TokenKind) -> bool {
        if std::mem::discriminant(&self.current().kind) == std::mem::discriminant(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    pub(crate) fn expect(&mut self, kind: &TokenKind) -> Result<(), String> {
        if std::mem::discriminant(&self.current().kind) == std::mem::discriminant(kind) {
            self.advance();
            Ok(())
        } else {
            Err(format!(
                "Expected {:?}, found {:?} at line {}",
                kind,
                self.current().kind,
                self.current().span.line
            ))
        }
    }

    pub(crate) fn expect_ident(&mut self) -> Result<String, String> {
        // Accept regular identifiers
        if let TokenKind::Ident(name) = &self.current().kind {
            let name = name.clone();
            self.advance();
            Ok(name)
        // Also accept type keywords that can be used as identifiers (e.g., in imports)
        } else if matches!(self.current().kind, 
            TokenKind::Optional | TokenKind::Tuple | TokenKind::Result | TokenKind::Class
        ) {
            let name = format!("{:?}", self.current().kind);
            let name = name.trim_start_matches("TokenKind::").to_string();
            self.advance();
            Ok(name)
        } else {
            Err(format!(
                "Expected identifier, found {:?} at line {}",
                self.current().kind,
                self.current().span.line
            ))
        }
    }

    pub(crate) fn is_at_end(&self) -> bool {
        matches!(self.current().kind, TokenKind::Eof)
    }
}

/// Parse all statements until EOF
pub fn parse_statements(parser: &mut StatementParser) -> Result<Vec<Stmt>, String> {
    let mut stmts = Vec::new();

    while !parser.is_at_end() {
        // Skip newlines and dedents at top level
        while parser.match_token(&TokenKind::Newline) || parser.match_token(&TokenKind::Dedent) {
            // empty
        }

        if parser.is_at_end() || parser.match_token(&TokenKind::Eof) {
            break;
        }

        stmts.push(parse_statement(parser)?);
    }

    Ok(stmts)
}

pub fn parse_statement(parser: &mut StatementParser) -> Result<Stmt, String> {
    let token = parser.current().clone();

    match &token.kind {
        TokenKind::At => {
            // Decorator - can be followed by function or class definition
            // Peek ahead past decorators to see what follows
            let mut peek_pos = parser.pos;
            
            // Skip all @ tokens and their associated names/arguments
            while peek_pos < parser.tokens.len() {
                match &parser.tokens[peek_pos].kind {
                    TokenKind::At => {
                        peek_pos += 1;
                        // Skip decorator name and any dotted suffix
                        while peek_pos < parser.tokens.len() {
                            match &parser.tokens[peek_pos].kind {
                                TokenKind::Ident(_) | TokenKind::Dot => peek_pos += 1,
                                TokenKind::LParen => {
                                    // Skip parenthesized arguments
                                    let mut paren_depth = 1;
                                    peek_pos += 1;
                                    while peek_pos < parser.tokens.len() && paren_depth > 0 {
                                        match &parser.tokens[peek_pos].kind {
                                            TokenKind::LParen => paren_depth += 1,
                                            TokenKind::RParen => paren_depth -= 1,
                                            _ => {}
                                        }
                                        peek_pos += 1;
                                    }
                                }
                                _ => break,
                            }
                        }
                    }
                    TokenKind::Newline => peek_pos += 1,
                    _ => break,
                }
            }
            
            // Now check what token follows the decorators
            if peek_pos < parser.tokens.len() {
                match &parser.tokens[peek_pos].kind {
                    TokenKind::Class => return parse_class_def(parser),
                    TokenKind::Def => return parse_function_def(parser),
                    _ => {}
                }
            }
            
            // Default to function def for backward compatibility
            parse_function_def(parser)
        }
        TokenKind::Def => parse_function_def(parser),
        TokenKind::Extern => parse_extern_decl(parser),
        TokenKind::If => parse_if_stmt(parser),
        TokenKind::While => parse_while_stmt(parser),
        TokenKind::For => parse_for_stmt(parser),
        TokenKind::Return => parse_return_stmt(parser),
        TokenKind::Break => {
            let span = parser.current().span;
            parser.advance();
            Ok(Stmt::Break(span))
        }
        TokenKind::Continue => {
            let span = parser.current().span;
            parser.advance();
            Ok(Stmt::Continue(span))
        }
        TokenKind::Pass => {
            let span = parser.current().span;
            parser.advance();
            Ok(Stmt::Pass(span))
        }
        TokenKind::Import => parse_import(parser),
        TokenKind::From => parse_from_import(parser),
        TokenKind::Class => parse_class_def(parser),
        TokenKind::Struct => parse_struct_def(parser),
        TokenKind::Try => parse_try_stmt(parser),
        TokenKind::Sync => parse_sync_block(parser),
        TokenKind::Task => parse_task_spawn(parser),
        TokenKind::Global => parse_global_decl(parser),
        TokenKind::Nonlocal => parse_nonlocal_decl(parser),
        TokenKind::Const => parse_const_decl(parser),
        TokenKind::Async => {
            // Check if this is async for, async def, or async with
            if matches!(parser.peek(), Some(t) if matches!(t.kind, TokenKind::For)) {
                parse_async_for_stmt(parser)
            } else if matches!(parser.peek(), Some(t) if matches!(t.kind, TokenKind::With)) {
                parser.advance(); // consume 'async'
                parse_with_stmt(parser, true)
            } else {
                parse_async_function_def(parser)
            }
        }
        TokenKind::Match => parse_match_stmt(parser),
        TokenKind::Select => parse_select_stmt(parser),
        TokenKind::Unless => parse_unless_stmt(parser),
        TokenKind::Type => parse_type_alias(parser),
        TokenKind::Assert => parse_assert_stmt(parser),
        TokenKind::Del => parse_delete_stmt(parser),
        TokenKind::Raise => parse_raise_stmt(parser),
        TokenKind::With => parse_with_stmt(parser, false),
        TokenKind::Yield => parse_yield_stmt(parser),
        TokenKind::Await => {
            // Await expression as statement
            let expr = parse_expression(parser)?;
            Ok(Stmt::Expr(expr))
        }
        TokenKind::PlusPlus | TokenKind::MinusMinus => {
            // Prefix increment/decrement as statement
            let expr = parse_expression(parser)?;
            Ok(Stmt::Expr(expr))
        }
        TokenKind::Ident(_) => {
            // Could be assignment or expression
            parse_assignment_or_expr(parser)
        }
        _ => {
            // Try expression statement
            let expr = parse_expression(parser)?;

            // Check if this is a concurrency builtin call that should be a statement
            if let Expr::Call { func, args, span } = expr {
                if let Some(stmt) = transform_concurrency_call(parser, &func, args.clone(), span) {
                    return Ok(stmt);
                }
                // Otherwise, put it back as a Call expression
                Ok(Stmt::Expr(Expr::Call { func, args, span }))
            } else {
                Ok(Stmt::Expr(expr))
            }
        }
    }
}
pub fn parse_return_stmt(parser: &mut StatementParser) -> Result<Stmt, String> {
    let span = parser.current().span;
    parser.expect(&TokenKind::Return)?;

    if parser.match_token(&TokenKind::Newline)
        || parser.match_token(&TokenKind::Dedent)
        || parser.is_at_end()
    {
        return Ok(Stmt::Return { value: None, span });
    }

    let first_expr = parse_expression(parser)?;
    let value = if parser.match_token(&TokenKind::Comma) {
        let mut elements = vec![first_expr];
        loop {
            elements.push(parse_expression(parser)?);
            if !parser.match_token(&TokenKind::Comma) {
                break;
            }
        }
        Some(Expr::Tuple { elements, span })
    } else {
        Some(first_expr)
    };

    Ok(Stmt::Return { value, span })
}

pub fn parse_import(parser: &mut StatementParser) -> Result<Stmt, String> {
    let span = parser.current().span;
    parser.expect(&TokenKind::Import)?;

    let module = parser.expect_ident()?;
    let alias =
        if parser.match_token(&TokenKind::As) { Some(parser.expect_ident()?) } else { None };

    Ok(Stmt::Import { module, alias, span })
}
pub fn parse_from_import(parser: &mut StatementParser) -> Result<Stmt, String> {
    let span = parser.current().span;
    parser.expect(&TokenKind::From)?;

    // Parse dotted module name (e.g., unittest.mock)
    let mut module = parser.expect_ident()?;
    while parser.match_token(&TokenKind::Dot) {
        let suffix = parser.expect_ident()?;
        module.push('.');
        module.push_str(&suffix);
    }
    
    parser.expect(&TokenKind::Import)?;

    let mut names = Vec::new();
    loop {
        let name = parser.expect_ident()?;
        let alias =
            if parser.match_token(&TokenKind::As) { Some(parser.expect_ident()?) } else { None };
        names.push((name, alias));

        if !parser.match_token(&TokenKind::Comma) {
            break;
        }
    }

    Ok(Stmt::FromImport { module, names, span })
}
pub fn parse_expression(parser: &mut StatementParser) -> Result<Expr, String> {
    let current_pos = parser.pos;
    parser.expr_parser.set_pos(current_pos);
    let expr = parser.expr_parser.parse_expr(crate::parser::precedence::Precedence::MIN)?;
    parser.pos = parser.expr_parser.pos();
    Ok(expr)
}
pub fn parse_block(parser: &mut StatementParser) -> Result<Vec<Stmt>, String> {
    // Expect indent after colon
    if !parser.match_token(&TokenKind::Indent) {
        // Single statement on same line
        let stmt = parse_statement(parser)?;
        return Ok(vec![stmt]);
    }

    let mut stmts = Vec::new();
    loop {
        // Check for dedent without consuming it
        if matches!(parser.current().kind, TokenKind::Dedent) {
            // Consume the dedent
            parser.advance();
            break;
        }
        if parser.is_at_end() {
            break;
        }
        if parser.match_token(&TokenKind::Newline) {
            continue;
        }
        stmts.push(parse_statement(parser)?);
    }

    Ok(stmts)
}

/// Parse assert statement: assert condition or assert condition, message
pub fn parse_assert_stmt(parser: &mut StatementParser) -> Result<Stmt, String> {
    let span = parser.current().span;
    parser.expect(&TokenKind::Assert)?;

    let condition = parse_expression(parser)?;

    let message = if parser.match_token(&TokenKind::Comma) {
        Some(Box::new(parse_expression(parser)?))
    } else {
        None
    };

    Ok(Stmt::Assert { condition: Box::new(condition), message, span })
}

/// Parse delete statement: del target1, target2, ...
pub fn parse_delete_stmt(parser: &mut StatementParser) -> Result<Stmt, String> {
    let span = parser.current().span;
    parser.expect(&TokenKind::Del)?;

    let mut targets = vec![parse_expression(parser)?];
    while parser.match_token(&TokenKind::Comma) {
        targets.push(parse_expression(parser)?);
    }

    Ok(Stmt::Delete { targets, span })
}

/// Parse raise statement: raise or raise Exception() or raise Exception() from cause
pub fn parse_raise_stmt(parser: &mut StatementParser) -> Result<Stmt, String> {
    let span = parser.current().span;
    parser.expect(&TokenKind::Raise)?;

    // Check if there's an exception to raise
    if parser.match_token(&TokenKind::Newline)
        || parser.match_token(&TokenKind::Dedent)
        || parser.is_at_end()
    {
        return Ok(Stmt::Raise { exception: None, cause: None, span });
    }

    let exception = Some(Box::new(parse_expression(parser)?));

    // Check for "from" clause
    let cause = if parser.match_token(&TokenKind::From) {
        Some(Box::new(parse_expression(parser)?))
    } else {
        None
    };

    Ok(Stmt::Raise { exception, cause, span })
}

/// Parse with statement: with expr as var: body or with expr1 as v1, expr2 as v2: body
/// If is_async is true, parses: async with expr as var: body
/// Note: When is_async is true, the 'async' token has already been consumed
pub fn parse_with_stmt(parser: &mut StatementParser, is_async: bool) -> Result<Stmt, String> {
    let span = parser.current().span;
    parser.expect(&TokenKind::With)?;

    let mut items = vec![];
    loop {
        let context_expr = parse_expression(parser)?;

        let optional_vars = if parser.match_token(&TokenKind::As) {
            Some(parser.expect_ident()?)
        } else {
            None
        };

        items.push(crate::ast::WithItem { context_expr, optional_vars, span });

        if !parser.match_token(&TokenKind::Comma) {
            break;
        }
    }

    parser.expect(&TokenKind::Colon)?;
    let body = parse_block(parser)?;

    Ok(Stmt::With { items, body, is_async, span })
}

/// Parse yield statement: yield or yield expr
pub fn parse_yield_stmt(parser: &mut StatementParser) -> Result<Stmt, String> {
    let span = parser.current().span;
    parser.expect(&TokenKind::Yield)?;

    // Check if there's a value to yield
    if parser.match_token(&TokenKind::Newline)
        || parser.match_token(&TokenKind::Dedent)
        || parser.is_at_end()
    {
        return Ok(Stmt::Yield { value: None, span });
    }

    let value = Some(Box::new(parse_expression(parser)?));
    Ok(Stmt::Yield { value, span })
}
