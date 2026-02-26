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
        if let TokenKind::Ident(name) = &self.current().kind {
            let name = name.clone();
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
        TokenKind::Mut => parse_mutable_decl(parser),
        TokenKind::Async => {
            // Check if this is async for or async def
            if matches!(parser.peek(), Some(t) if matches!(t.kind, TokenKind::For)) {
                parse_async_for_stmt(parser)
            } else {
                parse_async_function_def(parser)
            }
        }
        TokenKind::Match => parse_match_stmt(parser),
        TokenKind::Select => parse_select_stmt(parser),
        TokenKind::Unless => parse_unless_stmt(parser),
        TokenKind::Type => parse_type_alias(parser),
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

    let module = parser.expect_ident()?;
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
