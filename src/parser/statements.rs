use crate::ast::{
    BinOp, ExceptHandler, Expr, MatchCase, MatchPattern, Param, SelectCase, SelectCaseKind, Stmt,
    Type, UnaryOp,
};
use crate::lexer::{Token, TokenKind};
use crate::parser::expressions::PrattParser;
use crate::utils::Span;

/// Statement parser for Viper
pub struct StatementParser<'a> {
    tokens: &'a [Token],
    pos: usize,
    expr_parser: PrattParser<'a>,
}

impl<'a> StatementParser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            expr_parser: PrattParser::new(tokens),
            tokens,
            pos: 0,
        }
    }

    /// Parse all statements until EOF
    pub fn parse_statements(&mut self) -> Result<Vec<Stmt>, String> {
        let mut stmts = Vec::new();

        while !self.is_at_end() {
            // Skip newlines and dedents at top level
            while self.match_token(&TokenKind::Newline) || self.match_token(&TokenKind::Dedent) {
                // empty
            }

            if self.is_at_end() || self.match_token(&TokenKind::Eof) {
                break;
            }

            stmts.push(self.parse_statement()?);
        }

        Ok(stmts)
    }

    fn parse_statement(&mut self) -> Result<Stmt, String> {
        let token = self.current().clone();

        match &token.kind {
            TokenKind::Def => self.parse_function_def(),
            TokenKind::Extern => self.parse_extern_decl(),
            TokenKind::If => self.parse_if_stmt(),
            TokenKind::While => self.parse_while_stmt(),
            TokenKind::For => self.parse_for_stmt(),
            TokenKind::Return => self.parse_return_stmt(),
            TokenKind::Break => {
                let span = self.current().span;
                self.advance();
                Ok(Stmt::Break(span))
            }
            TokenKind::Continue => {
                let span = self.current().span;
                self.advance();
                Ok(Stmt::Continue(span))
            }
            TokenKind::Pass => {
                let span = self.current().span;
                self.advance();
                Ok(Stmt::Pass(span))
            }
            TokenKind::Import => self.parse_import(),
            TokenKind::From => self.parse_from_import(),
            TokenKind::Class => self.parse_class_def(),
            TokenKind::Struct => self.parse_struct_def(),
            TokenKind::Try => self.parse_try_stmt(),
            TokenKind::Sync => self.parse_sync_block(),
            TokenKind::Task => self.parse_task_spawn(),
            TokenKind::Mut => self.parse_mutable_decl(),
            TokenKind::Async => {
                // Check if this is async for or async def
                if matches!(self.peek(), Some(t) if matches!(t.kind, TokenKind::For)) {
                    self.parse_async_for_stmt()
                } else {
                    self.parse_async_function_def()
                }
            }
            TokenKind::Match => self.parse_match_stmt(),
            TokenKind::Select => self.parse_select_stmt(),
            TokenKind::Unless => self.parse_unless_stmt(),
            TokenKind::Await => {
                // Await expression as statement
                let expr = self.parse_expression()?;
                Ok(Stmt::Expr(expr))
            }
            TokenKind::Ident(_) => {
                // Could be assignment or expression
                self.parse_assignment_or_expr()
            }
            _ => {
                // Try expression statement
                let expr = self.parse_expression()?;

                // Check if this is a concurrency builtin call that should be a statement
                if let Expr::Call { func, args, span } = expr {
                    if let Some(stmt) = self.transform_concurrency_call(&func, args.clone(), span) {
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

    fn parse_function_def(&mut self) -> Result<Stmt, String> {
        let start_span = self.current().span;
        self.expect(&TokenKind::Def)?;

        let name_token = self.expect_ident()?;

        // Parse generic type parameters: [T, U, ...]
        let mut type_params = Vec::new();
        if self.match_token(&TokenKind::LBracket) {
            loop {
                if matches!(self.current().kind, TokenKind::Ident(_)) {
                    type_params.push(self.expect_ident()?);
                } else {
                    break;
                }
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
            self.expect(&TokenKind::RBracket)?;
        }

        self.expect(&TokenKind::LParen)?;

        let mut params = Vec::new();
        if !matches!(self.current().kind, TokenKind::RParen) {
            loop {
                let param = self.parse_param()?;
                params.push(param);
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
        }
        self.expect(&TokenKind::RParen)?;

        let return_type = if self.match_token(&TokenKind::Arrow) {
            Some(self.parse_type_annotation()?)
        } else {
            None
        };

        self.expect(&TokenKind::Colon)?;
        let body = self.parse_block()?;

        let span = start_span.merge(self.previous().span);

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

    fn parse_extern_decl(&mut self) -> Result<Stmt, String> {
        let start_span = self.current().span;
        self.expect(&TokenKind::Extern)?;

        if let TokenKind::Str(_) = &self.current().kind {
            self.advance();
        }

        self.expect(&TokenKind::Def)?;

        let name_token = self.expect_ident()?;
        self.expect(&TokenKind::LParen)?;

        let mut params = Vec::new();
        if !matches!(self.current().kind, TokenKind::RParen) {
            loop {
                let param = self.parse_param()?;
                params.push(param);
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
        }
        self.expect(&TokenKind::RParen)?;

        let return_type = if self.match_token(&TokenKind::Arrow) {
            Some(self.parse_type_annotation()?)
        } else {
            None
        };

        let span = start_span.merge(self.previous().span);

        Ok(Stmt::Extern {
            name: name_token,
            params,
            return_type,
            span,
        })
    }

    fn parse_async_function_def(&mut self) -> Result<Stmt, String> {
        let start_span = self.current().span;
        self.expect(&TokenKind::Async)?;
        self.expect(&TokenKind::Def)?;

        let name_token = self.expect_ident()?;
        self.expect(&TokenKind::LParen)?;

        let mut params = Vec::new();
        if !matches!(self.current().kind, TokenKind::RParen) {
            loop {
                let param = self.parse_param()?;
                params.push(param);
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
        }
        self.expect(&TokenKind::RParen)?;

        let return_type = if self.match_token(&TokenKind::Arrow) {
            Some(self.parse_type_annotation()?)
        } else {
            None
        };

        self.expect(&TokenKind::Colon)?;
        let body = self.parse_block()?;

        let span = start_span.merge(self.previous().span);

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

    fn parse_param(&mut self) -> Result<Param, String> {
        let span = self.current().span;
        let name = self.expect_ident()?;

        let type_ann = if self.match_token(&TokenKind::Colon) {
            Some(self.parse_type_annotation()?)
        } else {
            None
        };

        let default = if self.match_token(&TokenKind::Eq) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        Ok(Param {
            name,
            type_ann,
            default,
            span,
        })
    }

    fn parse_type_annotation(&mut self) -> Result<Type, String> {
        // Handle array type: [type; size]
        if self.match_token(&TokenKind::LBracket) {
            let elem_type = self.parse_type_annotation()?;
            self.expect(&TokenKind::Semi)?;
            let size_token = self.current();
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
            self.advance();
            self.expect(&TokenKind::RBracket)?;
            return Ok(Type::Array(Box::new(elem_type), size));
        }

        let token = self.current();
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
                    self.advance();
                    if !self.match_token(&TokenKind::LBracket) {
                        return Err("Expected '[' after Chan".to_string());
                    }
                    let elem_type = self.parse_type_annotation()?;
                    self.expect(&TokenKind::RBracket)?;
                    return Ok(Type::Chan(Box::new(elem_type)));
                }
                "WaitGroup" => {
                    self.advance();
                    return Ok(Type::WaitGroup);
                }
                "Callable" => {
                    self.advance();
                    return Ok(Type::Fn(vec![], Box::new(Type::I64)));
                }
                _ => Type::Var(name.clone()),
            },
            _ => return Err(format!("Expected type name, found {:?}", token.kind)),
        };
        self.advance();
        Ok(ty)
    }

    fn parse_unless_stmt(&mut self) -> Result<Stmt, String> {
        let start_span = self.current().span;
        self.expect(&TokenKind::Unless)?;

        let condition = self.parse_expression()?;

        // Negate the condition
        let negated_condition = Expr::UnaryOp {
            op: UnaryOp::Not,
            operand: Box::new(condition),
            span: start_span,
        };

        self.expect(&TokenKind::Colon)?;

        let body = self.parse_block()?;

        let span = start_span.merge(self.previous().span);

        Ok(Stmt::If {
            condition: negated_condition,
            body,
            elif_blocks: vec![],
            else_body: None,
            span,
        })
    }

    fn parse_if_stmt(&mut self) -> Result<Stmt, String> {
        let start_span = self.current().span;
        self.expect(&TokenKind::If)?;

        let condition = self.parse_expression()?;
        self.expect(&TokenKind::Colon)?;
        let body = self.parse_block()?;

        let mut elif_blocks = Vec::new();
        while self.match_token(&TokenKind::Elif) {
            let elif_cond = self.parse_expression()?;
            self.expect(&TokenKind::Colon)?;
            let elif_body = self.parse_block()?;
            elif_blocks.push((elif_cond, elif_body));
        }

        let else_body = if self.match_token(&TokenKind::Else) {
            self.expect(&TokenKind::Colon)?;
            Some(self.parse_block()?)
        } else {
            None
        };

        let span = start_span.merge(self.previous().span);

        Ok(Stmt::If {
            condition,
            body,
            elif_blocks,
            else_body,
            span,
        })
    }

    fn parse_while_stmt(&mut self) -> Result<Stmt, String> {
        let start_span = self.current().span;
        self.expect(&TokenKind::While)?;

        let condition = self.parse_expression()?;
        self.expect(&TokenKind::Colon)?;
        let body = self.parse_block()?;

        let else_body = if self.match_token(&TokenKind::Else) {
            self.expect(&TokenKind::Colon)?;
            Some(self.parse_block()?)
        } else {
            None
        };

        let span = start_span.merge(self.previous().span);

        Ok(Stmt::While {
            condition,
            body,
            else_body,
            span,
        })
    }

    fn parse_for_stmt(&mut self) -> Result<Stmt, String> {
        let start_span = self.current().span;
        self.expect(&TokenKind::For)?;

        let target = if matches!(self.current().kind, TokenKind::Ident(_))
            && matches!(self.peek().map(|t| &t.kind), Some(TokenKind::In))
        {
            let name = self.expect_ident()?;
            Ok(Expr::Ident(name, start_span.merge(self.previous().span)))
        } else {
            self.parse_expression()
        }?;
        self.expect(&TokenKind::In)?;
        let iter = self.parse_expression()?;
        self.expect(&TokenKind::Colon)?;
        let body = self.parse_block()?;

        let else_body = if self.match_token(&TokenKind::Else) {
            self.expect(&TokenKind::Colon)?;
            Some(self.parse_block()?)
        } else {
            None
        };

        let span = start_span.merge(self.previous().span);

        Ok(Stmt::For {
            target: Box::new(target),
            iter: Box::new(iter),
            body,
            else_body,
            is_async: false,
            span,
        })
    }

    fn parse_async_for_stmt(&mut self) -> Result<Stmt, String> {
        let start_span = self.current().span;
        self.expect(&TokenKind::Async)?;
        self.expect(&TokenKind::For)?;

        let target = if matches!(self.current().kind, TokenKind::Ident(_))
            && matches!(self.peek().map(|t| &t.kind), Some(TokenKind::In))
        {
            let name = self.expect_ident()?;
            Ok(Expr::Ident(name, start_span.merge(self.previous().span)))
        } else {
            self.parse_expression()
        }?;
        self.expect(&TokenKind::In)?;
        let iter = self.parse_expression()?;
        self.expect(&TokenKind::Colon)?;
        let body = self.parse_block()?;

        let else_body = if self.match_token(&TokenKind::Else) {
            self.expect(&TokenKind::Colon)?;
            Some(self.parse_block()?)
        } else {
            None
        };

        let span = start_span.merge(self.previous().span);

        Ok(Stmt::For {
            target: Box::new(target),
            iter: Box::new(iter),
            body,
            else_body,
            is_async: true,
            span,
        })
    }

    fn parse_return_stmt(&mut self) -> Result<Stmt, String> {
        let span = self.current().span;
        self.expect(&TokenKind::Return)?;

        let value = if self.match_token(&TokenKind::Newline)
            || self.match_token(&TokenKind::Dedent)
            || self.is_at_end()
        {
            None
        } else {
            Some(self.parse_expression()?)
        };

        Ok(Stmt::Return { value, span })
    }

    fn parse_import(&mut self) -> Result<Stmt, String> {
        let span = self.current().span;
        self.expect(&TokenKind::Import)?;

        let module = self.expect_ident()?;
        let alias = if self.match_token(&TokenKind::As) {
            Some(self.expect_ident()?)
        } else {
            None
        };

        Ok(Stmt::Import {
            module,
            alias,
            span,
        })
    }

    fn parse_from_import(&mut self) -> Result<Stmt, String> {
        let span = self.current().span;
        self.expect(&TokenKind::From)?;

        let module = self.expect_ident()?;
        self.expect(&TokenKind::Import)?;

        let mut names = Vec::new();
        loop {
            let name = self.expect_ident()?;
            let alias = if self.match_token(&TokenKind::As) {
                Some(self.expect_ident()?)
            } else {
                None
            };
            names.push((name, alias));

            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }

        Ok(Stmt::FromImport {
            module,
            names,
            span,
        })
    }

    fn parse_class_def(&mut self) -> Result<Stmt, String> {
        let start_span = self.current().span;
        self.expect(&TokenKind::Class)?;

        let name = self.expect_ident()?;

        let mut bases = Vec::new();
        if self.match_token(&TokenKind::LParen) {
            if !self.match_token(&TokenKind::RParen) {
                loop {
                    bases.push(self.parse_expression()?);
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
            }
            self.expect(&TokenKind::RParen)?;
        }

        self.expect(&TokenKind::Colon)?;
        let body = self.parse_block()?;

        let span = start_span.merge(self.previous().span);

        Ok(Stmt::Class {
            name,
            bases,
            body,
            span,
        })
    }

    fn parse_struct_def(&mut self) -> Result<Stmt, String> {
        let start_span = self.current().span;
        self.expect(&TokenKind::Struct)?;

        let name = self.expect_ident()?;

        self.expect(&TokenKind::Colon)?;

        let mut fields = Vec::new();
        if self.match_token(&TokenKind::Indent) {
            loop {
                if matches!(self.current().kind, TokenKind::Dedent) {
                    self.advance();
                    break;
                }
                if self.is_at_end() {
                    break;
                }
                if self.match_token(&TokenKind::Newline) {
                    continue;
                }

                let field_name = self.expect_ident()?;
                self.expect(&TokenKind::Colon)?;
                let field_type = self.parse_type_annotation()?;

                fields.push((field_name, field_type));

                self.match_token(&TokenKind::Newline);
            }
        }

        let span = start_span.merge(self.previous().span);

        Ok(Stmt::Struct { name, fields, span })
    }

    fn parse_try_stmt(&mut self) -> Result<Stmt, String> {
        let start_span = self.current().span;
        self.expect(&TokenKind::Try)?;
        self.expect(&TokenKind::Colon)?;
        let body = self.parse_block()?;

        let mut handlers = Vec::new();
        while self.match_token(&TokenKind::Except) {
            let handler_span = self.previous().span;

            let type_ann = if matches!(self.current().kind, TokenKind::Ident(_)) {
                let ty = self.parse_type_annotation()?;
                Some(ty)
            } else {
                None
            };

            let name = if self.match_token(&TokenKind::As) {
                Some(self.expect_ident()?)
            } else {
                None
            };

            self.expect(&TokenKind::Colon)?;
            let handler_body = self.parse_block()?;

            handlers.push(ExceptHandler {
                type_ann,
                name,
                body: handler_body,
                span: handler_span,
            });
        }

        let else_body = if self.match_token(&TokenKind::Else) {
            self.expect(&TokenKind::Colon)?;
            Some(self.parse_block()?)
        } else {
            None
        };

        let finally_body = if self.match_token(&TokenKind::Finally) {
            self.expect(&TokenKind::Colon)?;
            Some(self.parse_block()?)
        } else {
            None
        };

        let span = start_span.merge(self.previous().span);

        Ok(Stmt::Try {
            body,
            handlers,
            else_body,
            finally_body,
            span,
        })
    }

    fn parse_sync_block(&mut self) -> Result<Stmt, String> {
        let span = self.current().span;
        self.expect(&TokenKind::Sync)?;
        self.expect(&TokenKind::Colon)?;
        let body = self.parse_block()?;

        Ok(Stmt::Sync { body, span })
    }

    fn parse_task_spawn(&mut self) -> Result<Stmt, String> {
        let span = self.current().span;
        self.expect(&TokenKind::Task)?;
        let call = self.parse_expression()?;

        Ok(Stmt::Task { call, span })
    }

    fn parse_mutable_decl(&mut self) -> Result<Stmt, String> {
        let span = self.current().span;
        self.expect(&TokenKind::Mut)?;

        let name = self.expect_ident()?;

        let type_ann = if self.match_token(&TokenKind::Colon) {
            Some(self.parse_type_annotation()?)
        } else {
            None
        };

        let value = if self.match_token(&TokenKind::Eq) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        Ok(Stmt::Declare {
            name,
            type_ann,
            value,
            mutable: true,
            span,
        })
    }

    fn parse_assignment_or_expr(&mut self) -> Result<Stmt, String> {
        // Parse the left-hand side (should be an identifier or attribute access)
        let expr = self.parse_primary_expr()?;

        if self.match_token(&TokenKind::Eq) {
            // For the value, we need to parse a full expression including function calls
            let value = self.parse_value_expr()?;
            let span = expr.span().merge(value.span());
            Ok(Stmt::Assign {
                target: Box::new(expr),
                value: Box::new(value),
                span,
            })
        } else if self.is_augmented_assign() {
            let op = self.get_aug_assign_op();
            let value = self.parse_value_expr()?;
            let span = expr.span().merge(value.span());
            Ok(Stmt::AugAssign {
                target: Box::new(expr),
                op,
                value: Box::new(value),
                span,
            })
        } else {
            // Continue parsing the rest of the expression using the Pratt parser
            // But stop at statement delimiters (Newline, Dedent, etc.)
            self.expr_parser.set_pos(self.pos);

            // Check if we're at a statement delimiter
            if matches!(
                self.current().kind,
                TokenKind::Newline | TokenKind::Dedent | TokenKind::Eof
            ) {
                // Just return the primary expression as a statement
                Ok(Stmt::Expr(expr))
            } else {
                // Parse the full expression, carefully not eating the next line
                if self.is_at_end() {
                    return Ok(Stmt::Expr(expr));
                }

                // Let parse_value_expr do the bounded checking
                self.pos = self.expr_parser.pos(); // sync pos before value parse
                let full_expr = self.parse_value_expr_with_left(expr)?;
                Ok(Stmt::Expr(full_expr))
            }
        }
    }

    fn parse_value_expr_with_left(&mut self, left: Expr) -> Result<Expr, String> {
        self.expr_parser.set_pos(self.pos);

        if matches!(
            self.current().kind,
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
        let full_expr = self
            .expr_parser
            .parse_expr_with_left(left, crate::parser::precedence::Precedence::MIN)?;
        self.pos = self.expr_parser.pos();
        Ok(full_expr)
    }

    fn parse_value_expr(&mut self) -> Result<Expr, String> {
        // Parse a value expression (primary + postfix operators + binary operators)
        let expr = self.parse_primary_expr()?;

        // parse_primary_expr already handles function calls, so expr should be complete
        // Now check for binary operators
        self.expr_parser.set_pos(self.pos);

        // Check if we're at a statement boundary
        // This includes: delimiters, keywords that start statements, or indentation changes
        if matches!(
            self.current().kind,
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
            self.current().kind,
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
                | TokenKind::Mut
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
        if matches!(self.current().kind, TokenKind::Ident(_)) {
            // Peek at the next token to see if this Ident is followed by a postfix operator
            if let Some(next_token) = self.tokens.get(self.pos + 1) {
                let is_postfix = matches!(
                    next_token.kind,
                    TokenKind::LParen | TokenKind::LBracket | TokenKind::Dot
                );
                if !is_postfix {
                    // This Ident is likely the start of a new statement
                    return Ok(expr);
                }
            }
        }

        // Try to parse binary operators using the already parsed left side
        let full_expr = self
            .expr_parser
            .parse_expr_with_left(expr, crate::parser::precedence::Precedence::MIN)?;
        self.pos = self.expr_parser.pos();
        Ok(full_expr)
    }

    fn parse_primary_expr(&mut self) -> Result<Expr, String> {
        // Parse only primary expressions (identifiers, literals, etc.) without operators
        let token = self.current();
        let span = token.span;

        let expr = match &token.kind {
            TokenKind::Int(n) => {
                let n = *n;
                self.advance();
                // Check if the integer fits in i64
                if n > i64::MAX as i128 || n < i64::MIN as i128 {
                    return Err(format!("Integer literal too large for i64: {}", n));
                }
                Expr::Int(n as i64, span)
            }
            TokenKind::Float(n) => {
                let n = *n;
                self.advance();
                Expr::Float(n, span)
            }
            TokenKind::Str(s) => {
                let s = s.clone();
                self.advance();
                Expr::Str(s, span)
            }
            TokenKind::FString(s) => {
                let s = s.clone();
                self.advance();

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
            TokenKind::Bool(b) => {
                let b = *b;
                self.advance();
                Expr::Bool(b, span)
            }
            TokenKind::True => {
                self.advance();
                Expr::Bool(true, span)
            }
            TokenKind::False => {
                self.advance();
                Expr::Bool(false, span)
            }
            TokenKind::None => {
                self.advance();
                Expr::None(span)
            }
            TokenKind::Await => {
                self.advance();
                let future = self.parse_primary_expr()?;
                Expr::Await {
                    future: Box::new(future),
                    span,
                }
            }
            TokenKind::Ident(name) => {
                let name = name.clone();
                self.advance();
                // Check for attribute access or function call
                let mut expr = Expr::Ident(name, span);

                loop {
                    if self.match_token(&TokenKind::Dot) {
                        let attr = self.expect_ident()?;
                        let attr_span = self.previous().span;
                        expr = Expr::Attribute {
                            obj: Box::new(expr),
                            attr,
                            span: span.merge(attr_span),
                        };
                    } else if self.match_token(&TokenKind::LParen) {
                        let mut args = Vec::new();
                        if !self.match_token(&TokenKind::RParen) {
                            loop {
                                args.push(self.parse_expression()?);
                                if !self.match_token(&TokenKind::Comma) {
                                    break;
                                }
                            }
                            self.expect(&TokenKind::RParen)?;
                        }
                        // RParen was already consumed by match_token if it matched
                        let call_span = span.merge(self.previous().span);
                        expr = Expr::Call {
                            func: Box::new(expr),
                            args,
                            span: call_span,
                        };
                    } else if self.match_token(&TokenKind::LBracket) {
                        let index = self.parse_expression()?;
                        self.expect(&TokenKind::RBracket)?;
                        let index_span = span.merge(self.previous().span);
                        expr = Expr::Index {
                            obj: Box::new(expr),
                            index: Box::new(index),
                            span: index_span,
                        };
                    } else {
                        break;
                    }
                }

                expr
            }

            TokenKind::LParen => {
                self.advance();

                // Check for empty tuple or just parenthesized expression
                if self.match_token(&TokenKind::RParen) {
                    // Empty tuple - treat as None for now
                    return Ok(Expr::None(span));
                }

                let expr = self.parse_expression()?;

                // Check for tuple (has trailing comma)
                if self.match_token(&TokenKind::Comma) {
                    let mut elements = vec![expr];
                    while !self.match_token(&TokenKind::RParen) {
                        elements.push(self.parse_expression()?);
                        if !self.match_token(&TokenKind::Comma) {
                            break;
                        }
                    }
                    self.expect(&TokenKind::RParen)?;
                    let last_span = self.previous().span;
                    let merged_span = span.merge(last_span);
                    return Ok(Expr::Tuple {
                        elements,
                        span: merged_span,
                    });
                }

                self.expect(&TokenKind::RParen)?;
                expr
            }
            TokenKind::LBracket => {
                self.advance();
                let mut elements = Vec::new();
                let mut size: Option<usize> = None;

                // Check for empty list without consuming the RBracket
                if !matches!(self.current().kind, TokenKind::RBracket) {
                    // Parse first element
                    let first_elem = self.parse_expression()?;

                    // Check for list comprehension: [expr for var in iter]
                    if matches!(self.current().kind, TokenKind::For) {
                        // This is a list comprehension
                        self.advance(); // consume 'for'

                        // Parse the variable name
                        let var = if let TokenKind::Ident(name) = &self.current().kind {
                            let name = name.clone();
                            self.advance();
                            name
                        } else {
                            return Err("Expected variable name in list comprehension".to_string());
                        };

                        // Expect 'in' keyword
                        self.expect(&TokenKind::In)?;

                        // Parse the iterable
                        let iter = self.parse_expression()?;

                        // Expect closing bracket
                        self.expect(&TokenKind::RBracket)?;

                        let list_span = span.merge(self.previous().span);

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
                    if matches!(self.current().kind, TokenKind::Semi) {
                        self.advance(); // consume the semicolon
                        let size_token = self.current();
                        match &size_token.kind {
                            TokenKind::Int(n) => {
                                if *n < 0 || *n > usize::MAX as i128 {
                                    return Err(format!(
                                        "Array size must be a positive usize: {}",
                                        n
                                    ));
                                }
                                size = Some(*n as usize);
                                self.advance();
                            }
                            _ => {
                                return Err(format!(
                                    "Expected integer size for array, found {:?}",
                                    size_token.kind
                                ))
                            }
                        }
                        self.expect(&TokenKind::RBracket)?;
                    } else {
                        // Regular list/array: parse remaining elements
                        while self.match_token(&TokenKind::Comma) {
                            if matches!(self.current().kind, TokenKind::RBracket) {
                                break;
                            }
                            elements.push(self.parse_expression()?);
                        }
                        self.expect(&TokenKind::RBracket)?;
                    }
                } else {
                    self.expect(&TokenKind::RBracket)?;
                }

                let list_span = span.merge(self.previous().span);

                // Use Array node for fixed-size arrays, List for dynamic lists
                if size.is_some() || !elements.is_empty() {
                    Expr::Array {
                        elements,
                        size,
                        span: list_span,
                    }
                } else {
                    Expr::List {
                        elements,
                        span: list_span,
                    }
                }
            }
            TokenKind::Minus => {
                self.advance();
                let operand = self.parse_primary_expr()?;
                let neg_span = span.merge(operand.span());
                Expr::UnaryOp {
                    op: crate::ast::UnaryOp::Neg,
                    operand: Box::new(operand),
                    span: neg_span,
                }
            }
            TokenKind::Not => {
                self.advance();
                let operand = self.parse_primary_expr()?;
                let not_span = span.merge(operand.span());
                Expr::UnaryOp {
                    op: crate::ast::UnaryOp::Not,
                    operand: Box::new(operand),
                    span: not_span,
                }
            }
            TokenKind::Plus => {
                self.advance();
                let operand = self.parse_primary_expr()?;
                let plus_span = span.merge(operand.span());
                Expr::UnaryOp {
                    op: crate::ast::UnaryOp::Pos,
                    operand: Box::new(operand),
                    span: plus_span,
                }
            }
            TokenKind::Lambda | TokenKind::Fn => {
                self.advance();
                let mut params = Vec::new();
                if !matches!(self.current().kind, TokenKind::Colon) {
                    loop {
                        if let TokenKind::Ident(param_name) = &self.current().kind {
                            params.push(param_name.clone());
                            self.advance();
                        } else {
                            return Err("Expected parameter name in lambda".to_string());
                        }

                        if self.match_token(&TokenKind::Comma) {
                            continue;
                        } else {
                            break;
                        }
                    }
                }
                self.expect(&TokenKind::Colon)?;
                let body = self.parse_expression()?;
                let merged_span = span.merge(body.span());
                Expr::Lambda {
                    params,
                    body: Box::new(body),
                    span: merged_span,
                }
            }
            _ => return Err(format!("Unexpected token in expression: {:?}", token.kind)),
        };

        Ok(expr)
    }

    fn is_augmented_assign(&self) -> bool {
        matches!(
            self.current().kind,
            TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Star
                | TokenKind::Slash
                | TokenKind::Percent
                | TokenKind::DoubleStar
        ) && self
            .peek()
            .map_or(false, |t| matches!(t.kind, TokenKind::Eq))
    }

    fn get_aug_assign_op(&mut self) -> BinOp {
        self.advance(); // skip operator
        self.advance(); // skip =

        match &self.previous().kind {
            TokenKind::Plus => BinOp::Add,
            TokenKind::Minus => BinOp::Sub,
            TokenKind::Star => BinOp::Mul,
            TokenKind::Slash => BinOp::Div,
            TokenKind::Percent => BinOp::Mod,
            TokenKind::DoubleStar => BinOp::Pow,
            _ => BinOp::Add, // default
        }
    }

    fn parse_expression(&mut self) -> Result<Expr, String> {
        let current_pos = self.pos;
        self.expr_parser.set_pos(current_pos);
        let expr = self
            .expr_parser
            .parse_expr(crate::parser::precedence::Precedence::MIN)?;
        self.pos = self.expr_parser.pos();
        Ok(expr)
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, String> {
        // Expect indent after colon
        if !self.match_token(&TokenKind::Indent) {
            // Single statement on same line
            let stmt = self.parse_statement()?;
            return Ok(vec![stmt]);
        }

        let mut stmts = Vec::new();
        loop {
            // Check for dedent without consuming it
            if matches!(self.current().kind, TokenKind::Dedent) {
                // Consume the dedent
                self.advance();
                break;
            }
            if self.is_at_end() {
                break;
            }
            if self.match_token(&TokenKind::Newline) {
                continue;
            }
            stmts.push(self.parse_statement()?);
        }

        Ok(stmts)
    }

    fn current(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos + 1)
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.pos - 1]
    }

    fn advance(&mut self) {
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
    }

    fn match_token(&mut self, kind: &TokenKind) -> bool {
        if std::mem::discriminant(&self.current().kind) == std::mem::discriminant(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Transform concurrency builtin calls into appropriate AST nodes
    fn transform_concurrency_call(&self, func: &Expr, args: Vec<Expr>, span: Span) -> Option<Stmt> {
        if let Expr::Ident(name, _) = func {
            match name.as_str() {
                "chan" => {
                    if args.len() == 1 {
                        return Some(Stmt::Chan {
                            size: args.into_iter().next().unwrap(),
                            span,
                        });
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

    fn parse_match_stmt(&mut self) -> Result<Stmt, String> {
        let start_span = self.current().span;
        self.expect(&TokenKind::Match)?;

        let subject = self.parse_expression()?;

        self.expect(&TokenKind::Colon)?;

        // Allow either newline or immediate case
        let mut cases = Vec::new();

        // Check if we have case directly or need newline/indent
        if !matches!(self.current().kind, TokenKind::Case) {
            // Allow Newline or Indent
            if matches!(self.current().kind, TokenKind::Newline) {
                self.advance();
            }
            // Skip optional indent
            if matches!(self.current().kind, TokenKind::Indent) {
                self.expect(&TokenKind::Indent)?;
            }
        }

        while matches!(self.current().kind, TokenKind::Case) {
            let case_span = self.current().span;
            self.expect(&TokenKind::Case)?;

            let pattern = self.parse_match_pattern()?;

            let guard = if self.match_token(&TokenKind::If) {
                Some(self.parse_expression()?)
            } else {
                None
            };

            self.expect(&TokenKind::Colon)?;

            // Parse case body - can be single statement or indented block
            let mut body = Vec::new();

            // Skip optional newline after colon
            self.match_token(&TokenKind::Newline);

            // Check for indented block
            if matches!(self.current().kind, TokenKind::Indent) {
                self.expect(&TokenKind::Indent)?;
                loop {
                    if matches!(self.current().kind, TokenKind::Dedent) {
                        self.advance();
                        break;
                    }
                    if matches!(self.current().kind, TokenKind::Case) {
                        break;
                    }
                    if self.is_at_end() {
                        break;
                    }
                    if self.match_token(&TokenKind::Newline) {
                        continue;
                    }
                    body.push(self.parse_statement()?);
                }
            } else if !matches!(self.current().kind, TokenKind::Case) {
                // Single statement on same line
                body.push(self.parse_statement()?);
            }

            cases.push(MatchCase {
                pattern,
                guard,
                body,
                span: case_span.merge(self.previous().span),
            });
        }

        let span = start_span.merge(self.previous().span);

        Ok(Stmt::Match {
            subject: Box::new(subject),
            cases,
            span,
        })
    }

    fn parse_select_stmt(&mut self) -> Result<Stmt, String> {
        let start_span = self.current().span;
        self.expect(&TokenKind::Select)?;
        self.expect(&TokenKind::Colon)?;

        let mut cases = Vec::new();

        // Handle optional newline
        if matches!(self.current().kind, TokenKind::Newline) {
            self.expect(&TokenKind::Newline)?;
            if matches!(self.current().kind, TokenKind::Indent) {
                self.expect(&TokenKind::Indent)?;
            }
        }

        while matches!(self.current().kind, TokenKind::Case) {
            let case_span = self.current().span;
            self.expect(&TokenKind::Case)?;

            // Parse the case kind: recv, send, or default
            let kind = if self.match_token(&TokenKind::Recv) {
                self.expect(&TokenKind::LParen)?;
                let chan = self.parse_expression()?;
                self.expect(&TokenKind::RParen)?;
                let var = if self.match_token(&TokenKind::Eq) {
                    Some(self.expect_ident()?)
                } else {
                    None
                };
                SelectCaseKind::Recv {
                    chan: Box::new(chan),
                    var,
                }
            } else if self.match_token(&TokenKind::Send) {
                self.expect(&TokenKind::LParen)?;
                let chan = self.parse_expression()?;
                self.expect(&TokenKind::Comma)?;
                let value = self.parse_expression()?;
                self.expect(&TokenKind::RParen)?;
                SelectCaseKind::Send {
                    chan: Box::new(chan),
                    value: Box::new(value),
                }
            } else if let TokenKind::Ident(name) = &self.current().kind {
                if name == "default" {
                    self.advance();
                    SelectCaseKind::Default
                } else {
                    return Err("Expected 'recv', 'send', or 'default' in select case".to_string());
                }
            } else {
                return Err("Expected 'recv', 'send', or 'default' in select case".to_string());
            };

            self.expect(&TokenKind::Colon)?;

            // Parse body
            self.match_token(&TokenKind::Newline);
            let mut body = Vec::new();
            if matches!(self.current().kind, TokenKind::Indent) {
                self.expect(&TokenKind::Indent)?;
                loop {
                    if matches!(self.current().kind, TokenKind::Dedent) {
                        self.advance();
                        break;
                    }
                    if matches!(self.current().kind, TokenKind::Case) {
                        break;
                    }
                    if self.is_at_end() {
                        break;
                    }
                    if self.match_token(&TokenKind::Newline) {
                        continue;
                    }
                    body.push(self.parse_statement()?);
                }
            } else if !matches!(self.current().kind, TokenKind::Case) {
                body.push(self.parse_statement()?);
            }

            cases.push(SelectCase {
                kind,
                body,
                span: case_span,
            });
        }

        let span = start_span.merge(self.previous().span);
        Ok(Stmt::Select { cases, span })
    }

    fn parse_match_pattern(&mut self) -> Result<MatchPattern, String> {
        let token = self.current().clone();

        match token.kind {
            TokenKind::Underscore => {
                self.advance();
                Ok(MatchPattern::Wildcard)
            }
            TokenKind::Int(ref n) => {
                let span = token.span;
                self.advance();
                // Check if the integer fits in i64
                if *n > i64::MAX as i128 || *n < i64::MIN as i128 {
                    return Err(format!("Integer literal too large for i64: {}", n));
                }
                Ok(MatchPattern::Constant(Expr::Int(*n as i64, span)))
            }
            TokenKind::Str(s) => {
                let span = token.span;
                self.advance();
                Ok(MatchPattern::Constant(Expr::Str(s, span)))
            }
            TokenKind::Bool(b) => {
                let span = token.span;
                self.advance();
                Ok(MatchPattern::Constant(Expr::Bool(b, span)))
            }
            TokenKind::None => {
                let span = token.span;
                self.advance();
                Ok(MatchPattern::Constant(Expr::None(span)))
            }
            TokenKind::LBracket => self.parse_match_list_pattern(),
            TokenKind::LParen => self.parse_match_tuple_pattern(),
            TokenKind::Ident(name) => {
                self.advance();
                let next_token = self.current();
                if matches!(next_token.kind, TokenKind::LParen) {
                    self.parse_match_type_pattern(&name)
                } else {
                    Ok(MatchPattern::Variable(name))
                }
            }
            _ => Err(format!("Unexpected token in pattern: {:?}", token.kind)),
        }
    }

    fn parse_match_list_pattern(&mut self) -> Result<MatchPattern, String> {
        self.expect(&TokenKind::LBracket)?;

        let mut elements = Vec::new();
        let mut rest = None;

        if !matches!(self.current().kind, TokenKind::RBracket) {
            loop {
                if matches!(self.current().kind, TokenKind::DotDot) {
                    self.expect(&TokenKind::DotDot)?;
                    rest = Some(self.expect_ident()?);
                    break;
                }
                elements.push(self.parse_match_pattern()?);
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
        }

        self.expect(&TokenKind::RBracket)?;

        Ok(MatchPattern::List { elements, rest })
    }

    fn parse_match_tuple_pattern(&mut self) -> Result<MatchPattern, String> {
        self.expect(&TokenKind::LParen)?;

        let mut elements = Vec::new();
        if !matches!(self.current().kind, TokenKind::RParen) {
            loop {
                elements.push(self.parse_match_pattern()?);
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
        }

        self.expect(&TokenKind::RParen)?;

        Ok(MatchPattern::Tuple(elements))
    }

    fn parse_match_type_pattern(&mut self, type_name: &str) -> Result<MatchPattern, String> {
        self.expect(&TokenKind::LParen)?;

        let binding = if matches!(self.current().kind, TokenKind::Ident(_))
            && !matches!(
                self.peek().map(|t| &t.kind),
                Some(TokenKind::Comma) | Some(TokenKind::RParen)
            ) {
            Some(self.expect_ident()?)
        } else {
            None
        };

        self.expect(&TokenKind::RParen)?;

        Ok(MatchPattern::TypeCheck {
            type_name: type_name.to_string(),
            binding,
        })
    }

    fn expect(&mut self, kind: &TokenKind) -> Result<(), String> {
        if std::mem::discriminant(&self.current().kind) == std::mem::discriminant(kind) {
            self.advance();
            Ok(())
        } else {
            Err(format!(
                "Expected {:?}, found {:?}",
                kind,
                self.current().kind
            ))
        }
    }

    fn expect_ident(&mut self) -> Result<String, String> {
        if let TokenKind::Ident(name) = &self.current().kind {
            let name = name.clone();
            self.advance();
            Ok(name)
        } else {
            Err(format!(
                "Expected identifier, found {:?}",
                self.current().kind
            ))
        }
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len() - 1
    }
}
