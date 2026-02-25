use crate::ast::{BinOp, Expr, UnaryOp};
use crate::lexer::{Token, TokenKind};
use crate::parser::precedence::Precedence;
use crate::utils::Span;

/// Pratt parser for expression parsing with proper precedence
pub struct PrattParser<'a> {
    tokens: &'a [Token],
    pos: usize,
    eof_token: Token,
}

impl<'a> PrattParser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens,
            pos: 0,
            eof_token: Token::new(TokenKind::Eof, Span::empty(0, 0)),
        }
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn set_pos(&mut self, pos: usize) {
        self.pos = pos;
    }

    pub fn parse_expr_with_left(
        &mut self,
        mut left: Expr,
        min_prec: Precedence,
    ) -> Result<Expr, String> {
        // Handle postfix operators (function calls, indexing, attribute access)
        loop {
            if self.match_token(&TokenKind::LParen) {
                // Function call
                let mut args = Vec::new();
                if !self.match_token(&TokenKind::RParen) {
                    loop {
                        args.push(self.parse_expr(Precedence::MIN)?);
                        if !self.match_token(&TokenKind::Comma) {
                            break;
                        }
                    }
                }
                self.expect(&TokenKind::RParen)?;
                let call_span = left.span().merge(self.previous().span);
                left = Expr::Call {
                    func: Box::new(left),
                    args,
                    span: call_span,
                };
            } else if self.match_token(&TokenKind::LBracket) {
                // Indexing
                let index = self.parse_expr(Precedence::MIN)?;
                self.expect(&TokenKind::RBracket)?;
                let index_span = left.span().merge(self.previous().span);
                left = Expr::Index {
                    obj: Box::new(left),
                    index: Box::new(index),
                    span: index_span,
                };
            } else if self.match_token(&TokenKind::Dot) {
                // Attribute access
                if let TokenKind::Ident(attr) = &self.current().kind {
                    let attr = attr.clone();
                    self.advance();
                    let attr_span = left.span().merge(self.previous().span);
                    left = Expr::Attribute {
                        obj: Box::new(left),
                        attr,
                        span: attr_span,
                    };
                } else {
                    return Err("Expected attribute name after '.'".to_string());
                }
            } else {
                break;
            }
        }

        // Handle binary/infix operators
        loop {
            // Check for postfix operators
            if let Some(op_prec) = self.infix_precedence() {
                if op_prec < min_prec {
                    break;
                }

                // Handle right associativity
                let next_min_prec = if Precedence::EXPONENT.0 == op_prec.0 {
                    op_prec
                } else {
                    Precedence(op_prec.0 + 1)
                };

                let op = self.parse_infix_op()?;
                let right = self.parse_expr(next_min_prec)?;
                let span = left.span().merge(right.span());

                left = Expr::BinOp {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                    span,
                };
            } else {
                break;
            }
        }

        // Handle ternary expression: `then_expr if cond else else_expr`
        if matches!(self.current().kind, TokenKind::If) {
            // Look ahead to see if this is a ternary
            let saved_pos = self.pos;

            self.advance(); // consume 'if'

            // Parse condition
            match self.parse_expr(Precedence::MIN) {
                Ok(condition) => {
                    // Check for 'else'
                    if matches!(self.current().kind, TokenKind::Else) {
                        self.advance(); // consume 'else'
                                        // Parse else expression
                        match self.parse_expr(Precedence::MIN) {
                            Ok(else_expr) => {
                                // This is a valid ternary!
                                let span = left.span().merge(else_expr.span());
                                left = Expr::Conditional {
                                    condition: Box::new(condition),
                                    then_expr: Box::new(left),
                                    else_expr: Box::new(else_expr),
                                    span,
                                };
                                // Don't restore position - we successfully parsed a ternary
                            }
                            Err(_) => {
                                // Failed to parse else expression, restore position
                                self.pos = saved_pos;
                            }
                        }
                    } else {
                        // No 'else' found, this is an if statement, restore position
                        self.pos = saved_pos;
                    }
                }
                Err(_) => {
                    // Failed to parse condition, restore position
                    self.pos = saved_pos;
                }
            }
        }

        Ok(left)
    }

    /// Parse an expression with minimum precedence
    pub fn parse_expr(&mut self, min_prec: Precedence) -> Result<Expr, String> {
        // Parse prefix expression
        let left = self.parse_prefix()?;
        self.parse_expr_with_left(left, min_prec)
    }

    fn parse_prefix(&mut self) -> Result<Expr, String> {
        let token = self.current();
        let _kind = token.kind.clone();
        let span = token.span;
        let token_kind = &token.kind;

        match token_kind {
            TokenKind::Int(n) => {
                let n = *n;
                self.advance();
                Ok(Expr::Int(n, span))
            }
            TokenKind::Float(n) => {
                let n = *n;
                self.advance();
                Ok(Expr::Float(n, span))
            }
            TokenKind::Str(s) => {
                let s = s.clone();
                self.advance();
                Ok(Expr::Str(s, span))
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
                            let mut inner_parser = PrattParser::new(&tokens);
                            if let Ok(expr) = inner_parser.parse_expr(Precedence::MIN) {
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

                Ok(Expr::FString(elements, span))
            }
            TokenKind::Bool(b) => {
                let b = *b;
                self.advance();
                Ok(Expr::Bool(b, span))
            }
            TokenKind::None => {
                self.advance();
                Ok(Expr::None(span))
            }
            TokenKind::Lambda => {
                // Changed from TokenKind::Ident(name) if name == "lambda"
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
                let body = self.parse_expr(Precedence::MIN)?;
                let merged_span = span.merge(body.span());
                Ok(Expr::Lambda {
                    params,
                    body: Box::new(body),
                    span: merged_span,
                })
            }
            TokenKind::Ident(name) => {
                let name = name.clone();
                self.advance();
                Ok(Expr::Ident(name, span))
            }
            TokenKind::True => {
                self.advance();
                Ok(Expr::Bool(true, span))
            }
            TokenKind::False => {
                self.advance();
                Ok(Expr::Bool(false, span))
            }
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expr(Precedence::MIN)?;

                // Check for tuple
                if self.match_token(&TokenKind::Comma) {
                    let mut elements = vec![expr];
                    while !self.match_token(&TokenKind::RParen) {
                        elements.push(self.parse_expr(Precedence::MIN)?);
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
                Ok(expr)
            }
            TokenKind::LBracket => {
                self.advance();
                let mut elements = Vec::new();
                let mut size: Option<usize> = None;

                if !self.match_token(&TokenKind::RBracket) {
                    // Parse first element
                    elements.push(self.parse_expr(Precedence::MIN)?);

                    // Check for array repetition syntax: [value; size]
                    let is_semi = matches!(self.current().kind, TokenKind::Semi);
                    if is_semi {
                        self.advance(); // consume the semicolon
                        let size_token = self.current();
                        match &size_token.kind {
                            TokenKind::Int(n) => {
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
                            if self.match_token(&TokenKind::RBracket) {
                                break;
                            }
                            elements.push(self.parse_expr(Precedence::MIN)?);
                        }
                        self.expect(&TokenKind::RBracket)?;
                    }
                } else {
                    self.expect(&TokenKind::RBracket)?;
                }

                let last_span = self.previous().span;
                let merged_span = span.merge(last_span);

                // Use Array node for fixed-size arrays, List for dynamic lists
                if size.is_some() || !elements.is_empty() {
                    Ok(Expr::Array {
                        elements,
                        size,
                        span: merged_span,
                    })
                } else {
                    Ok(Expr::List {
                        elements,
                        span: merged_span,
                    })
                }
            }
            TokenKind::Minus => {
                self.advance();
                let operand = self.parse_expr(Precedence::UNARY)?;
                let merged_span = span.merge(operand.span());
                Ok(Expr::UnaryOp {
                    op: UnaryOp::Neg,
                    operand: Box::new(operand),
                    span: merged_span,
                })
            }
            TokenKind::Not => {
                self.advance();
                let operand = self.parse_expr(Precedence::UNARY)?;
                let merged_span = span.merge(operand.span());
                Ok(Expr::UnaryOp {
                    op: UnaryOp::Not,
                    operand: Box::new(operand),
                    span: merged_span,
                })
            }
            TokenKind::Await => {
                self.advance();
                let future = self.parse_expr(Precedence::UNARY)?;
                let merged_span = span.merge(future.span());
                Ok(Expr::Await {
                    future: Box::new(future),
                    span: merged_span,
                })
            }
            TokenKind::Plus => {
                self.advance();
                let operand = self.parse_expr(Precedence::UNARY)?;
                let merged_span = span.merge(operand.span());
                Ok(Expr::UnaryOp {
                    op: UnaryOp::Pos,
                    operand: Box::new(operand),
                    span: merged_span,
                })
            }
            TokenKind::Tilde => {
                self.advance();
                let operand = self.parse_expr(Precedence::UNARY)?;
                let merged_span = span.merge(operand.span());
                Ok(Expr::UnaryOp {
                    op: UnaryOp::Invert,
                    operand: Box::new(operand),
                    span: merged_span,
                })
            }
            _ => Err(format!("Unexpected token in expression: {:?}", token.kind)),
        }
    }

    fn parse_infix_op(&mut self) -> Result<BinOp, String> {
        let kind = self.current().kind.clone();
        self.advance();

        let op = match kind {
            TokenKind::Plus => BinOp::Add,
            TokenKind::Minus => BinOp::Sub,
            TokenKind::Star => BinOp::Mul,
            TokenKind::Slash => BinOp::Div,
            TokenKind::Percent => BinOp::Mod,
            TokenKind::DoubleSlash => BinOp::FloorDiv,
            TokenKind::DoubleStar => BinOp::Pow,
            TokenKind::Eq | TokenKind::EqEq => BinOp::Eq,
            TokenKind::NotEq => BinOp::NotEq,
            TokenKind::Lt => BinOp::Lt,
            TokenKind::LtEq => BinOp::LtEq,
            TokenKind::Gt => BinOp::Gt,
            TokenKind::GtEq => BinOp::GtEq,
            TokenKind::And => BinOp::And,
            TokenKind::Or => BinOp::Or,
            TokenKind::Ampersand => BinOp::BitAnd,
            TokenKind::Pipe => BinOp::BitOr,
            TokenKind::Caret => BinOp::BitXor,
            TokenKind::LtLt => BinOp::LShift,
            TokenKind::GtGt => BinOp::RShift,
            TokenKind::Is => BinOp::Is,
            TokenKind::IsNot => BinOp::IsNot,
            TokenKind::In => BinOp::In,
            TokenKind::NotIn => BinOp::NotIn,
            _ => return Err("Unknown infix operator".to_string()),
        };

        Ok(op)
    }

    fn infix_precedence(&self) -> Option<Precedence> {
        let token = self.peek();

        let prec = match &token.kind {
            TokenKind::Or => Precedence::OR,
            TokenKind::And => Precedence::AND,
            TokenKind::Eq
            | TokenKind::EqEq
            | TokenKind::NotEq
            | TokenKind::Lt
            | TokenKind::LtEq
            | TokenKind::Gt
            | TokenKind::GtEq
            | TokenKind::Is
            | TokenKind::IsNot
            | TokenKind::In
            | TokenKind::NotIn => Precedence::COMPARISON,
            TokenKind::Pipe => Precedence::BITWISE_OR,
            TokenKind::Caret => Precedence::BITWISE_XOR,
            TokenKind::Ampersand => Precedence::BITWISE_AND,
            TokenKind::LtLt | TokenKind::GtGt => Precedence::BITWISE_SHIFT,
            TokenKind::Plus | TokenKind::Minus => Precedence::SUM,
            TokenKind::Star | TokenKind::Slash | TokenKind::Percent | TokenKind::DoubleSlash => {
                Precedence::PRODUCT
            }
            TokenKind::DoubleStar => Precedence::EXPONENT,
            _ => return None,
        };

        Some(prec)
    }

    fn current(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&self.eof_token)
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&self.eof_token)
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
        if self.pos >= self.tokens.len() {
            return false;
        }
        if std::mem::discriminant(&self.current().kind) == std::mem::discriminant(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, kind: &TokenKind) -> Result<(), String> {
        if self.pos >= self.tokens.len() {
            return Err(format!("Expected {:?}, but reached end of tokens", kind));
        }
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
}
