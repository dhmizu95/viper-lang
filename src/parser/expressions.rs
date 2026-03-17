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
        Self { tokens, pos: 0, eof_token: Token::new(TokenKind::Eof, Span::empty(0, 0)) }
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
    ) -> crate::error::Result<Expr> {
        // Handle postfix operators (function calls, indexing, attribute access)
        loop {
            // Stop at statement boundaries (newlines, dedents, etc.)
            // Check if we're at a dedent or if the next token is on a different line
            if matches!(self.current().kind, TokenKind::Dedent) {
                break;
            }
            // Check if the current token is on a different line than the left expression
            // This handles Python-style statement boundaries without explicit Newline tokens
            if self.current().span.line > left.span().line {
                break;
            }

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
                    self.expect(&TokenKind::RParen)?;
                }
                let call_span = left.span().merge(self.previous().span);
                left = Expr::Call { func: Box::new(left), args, span: call_span };
            } else if self.match_token(&TokenKind::LBracket) {
                // Indexing or slicing
                let _index_span_start = self.current().span;

                // Check if this is a slice by looking for ':' before ']'
                let is_slice = self.is_slice_pattern();

                if is_slice {
                    // Parse slice: [:], [start:], [:end], [start:end], [::step], etc.
                    let mut start: Option<Box<Expr>> = None;
                    let mut end: Option<Box<Expr>> = None;
                    let mut step: Option<Box<Expr>> = None;

                    // Parse start (optional)
                    if !matches!(self.current().kind, TokenKind::Colon) {
                        start = Some(Box::new(self.parse_expr(Precedence::MIN)?));
                    }

                    // Expect first colon
                    self.expect(&TokenKind::Colon)?;

                    // Parse end (optional)
                    if !matches!(self.current().kind, TokenKind::RBracket)
                        && !matches!(self.current().kind, TokenKind::Colon)
                    {
                        end = Some(Box::new(self.parse_expr(Precedence::MIN)?));
                    }

                    // Check for step
                    if matches!(self.current().kind, TokenKind::Colon) {
                        self.expect(&TokenKind::Colon)?;
                        // Parse step (optional)
                        if !matches!(self.current().kind, TokenKind::RBracket) {
                            step = Some(Box::new(self.parse_expr(Precedence::MIN)?));
                        }
                    }

                    self.expect(&TokenKind::RBracket)?;
                    let index_span = left.span().merge(self.previous().span);
                    left = Expr::Slice { obj: Box::new(left), start, end, step, span: index_span };
                } else {
                    // Regular indexing
                    let index = self.parse_expr(Precedence::MIN)?;
                    self.expect(&TokenKind::RBracket)?;
                    let index_span = left.span().merge(self.previous().span);
                    left = Expr::Index {
                        obj: Box::new(left),
                        index: Box::new(index),
                        span: index_span,
                    };
                }
            } else if self.match_token(&TokenKind::Dot) {
                // Attribute access
                if let TokenKind::Ident(attr) = &self.current().kind {
                    let attr = attr.clone();
                    self.advance();
                    let attr_span = left.span().merge(self.previous().span);
                    left = Expr::Attribute { obj: Box::new(left), attr, span: attr_span };
                } else {
                    return crate::parser::parse_error(
                        "Expected attribute name after '.'".to_string(),
                    );
                }
            } else if self.match_token(&TokenKind::PlusPlus) {
                // Postfix increment: x++
                let inc_span = left.span().merge(self.previous().span);
                left = Expr::UnaryOp {
                    op: UnaryOp::PostIncrement,
                    operand: Box::new(left),
                    span: inc_span,
                };
            } else if self.match_token(&TokenKind::MinusMinus) {
                // Postfix decrement: x--
                let dec_span = left.span().merge(self.previous().span);
                left = Expr::UnaryOp {
                    op: UnaryOp::PostDecrement,
                    operand: Box::new(left),
                    span: dec_span,
                };
            } else if self.match_token(&TokenKind::Question) {
                // Error propagation operator: expr?
                // Unwraps Result<T, E>, returns early on error
                let unwrap_span = left.span().merge(self.previous().span);
                left = Expr::UnaryOp {
                    op: UnaryOp::Unwrap,
                    operand: Box::new(left),
                    span: unwrap_span,
                };
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

                // Check for walrus operator (:=) - assignment expression
                if matches!(self.peek().kind, TokenKind::ColonEq) {
                    self.advance(); // consume :=

                    // Left side must be an identifier
                    let target_span = left.span();
                    let target = match &left {
                        Expr::Ident(name, _) => name.clone(),
                        _ => {
                            return crate::parser::parse_error(
                                "Walrus operator requires an identifier on the left side"
                                    .to_string(),
                            )
                        }
                    };

                    // Parse the value expression
                    let value = self.parse_expr(op_prec)?;
                    let span = target_span.merge(value.span());

                    // Create assignment expression: target := value
                    left = Expr::AssignmentExpr {
                        target: Box::new(Expr::Ident(target, target_span)),
                        value: Box::new(value),
                        span,
                    };
                    continue;
                }

                // Check for pipeline operator specially
                if matches!(self.peek().kind, TokenKind::Pipeline) {
                    self.advance(); // consume |>
                                    // Use one higher precedence for left-associativity
                    let right = self.parse_expr(Precedence(Precedence::PIPELINE.0 + 1))?;
                    // Transform: left |> right  =>  right(left)
                    let span = left.span().merge(right.span());
                    left = Expr::Call { func: Box::new(right), args: vec![left], span };
                    continue;
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

                left = Expr::BinOp { left: Box::new(left), op, right: Box::new(right), span };
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
    pub fn parse_expr(&mut self, min_prec: Precedence) -> crate::error::Result<Expr> {
        // Parse prefix expression
        let left = self.parse_prefix()?;
        self.parse_expr_with_left(left, min_prec)
    }

    fn parse_prefix(&mut self) -> crate::error::Result<Expr> {
        let token = self.current();
        let _kind = token.kind.clone();
        let span = token.span;
        let token_kind = &token.kind;

        match token_kind {
            TokenKind::Int(n) => {
                let n = *n;
                self.advance();
                // Check if the integer fits in i64
                if n > i64::MAX as i128 || n < i64::MIN as i128 {
                    // Convert to BigInt if too large for i64
                    Ok(Expr::BigInt(n.to_string(), span))
                } else {
                    Ok(Expr::Int(n as i64, span))
                }
            }
            TokenKind::BigInt(s) => {
                let s = s.clone();
                self.advance();
                Ok(Expr::BigInt(s, span))
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

                        // Parse inner expression and optional format spec
                        let mut inner_expr_str = String::new();
                        let mut format_spec: Option<String> = None;
                        let mut found_colon = false;
                        
                        while let Some(&next_c) = chars.peek() {
                            if next_c == '}' {
                                chars.next(); // consume '}'
                                break;
                            } else if next_c == ':' && !found_colon {
                                // Format spec separator
                                found_colon = true;
                                chars.next(); // consume ':'
                                // Rest is format spec
                                let mut spec = String::new();
                                while let Some(&spec_c) = chars.peek() {
                                    if spec_c == '}' {
                                        break;
                                    }
                                    spec.push(chars.next().unwrap());
                                }
                                format_spec = Some(spec);
                            } else {
                                inner_expr_str.push(chars.next().unwrap());
                            }
                        }

                        // Tokenize and parse inner expression
                        let mut inner_lexer = crate::lexer::Lexer::new(&inner_expr_str);
                        if let Ok(tokens) = inner_lexer.tokenize() {
                            let mut inner_parser = PrattParser::new(&tokens);
                            if let Ok(expr) = inner_parser.parse_expr(Precedence::MIN) {
                                // Create FStringElement with format spec
                                elements.push(Expr::FStringElement {
                                    expr: Box::new(expr),
                                    format_spec,
                                    span,
                                });
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
            TokenKind::Bytes(b) => {
                let b = b.clone();
                self.advance();
                Ok(Expr::Bytes(b, span))
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
            TokenKind::Lambda | TokenKind::Fn => {
                self.advance();
                let mut params = Vec::new();

                // Handle optional parentheses around parameters: fn(x, y: expr) or fn(x: expr)
                let paren_params = self.match_token(&TokenKind::LParen);

                if !matches!(self.current().kind, TokenKind::Colon) {
                    loop {
                        if let TokenKind::Ident(param_name) = &self.current().kind {
                            params.push(param_name.clone());
                            self.advance();
                        } else if paren_params && matches!(self.current().kind, TokenKind::RParen) {
                            // Empty parameter list like fn(): expr
                            break;
                        } else {
                            return crate::parser::parse_error(
                                "Expected parameter name in lambda".to_string(),
                            );
                        }

                        if self.match_token(&TokenKind::Comma) {
                            // Check if there's another parameter or closing paren
                            if paren_params && matches!(self.current().kind, TokenKind::RParen) {
                                break;
                            }
                            continue;
                        } else if paren_params && self.match_token(&TokenKind::Colon) {
                            // Shorthand syntax: fn(x, y: body)
                            let body = self.parse_expr(Precedence::MIN)?;
                            self.expect(&TokenKind::RParen)?;
                            let merged_span = span.merge(body.span());
                            return Ok(Expr::Lambda {
                                params,
                                body: Box::new(body),
                                span: merged_span,
                            });
                        } else {
                            break;
                        }
                    }
                }

                // If we opened a paren, expect closing paren before the colon
                if paren_params {
                    self.expect(&TokenKind::RParen)?;
                }

                self.expect(&TokenKind::Colon)?;
                let body = self.parse_expr(Precedence::MIN)?;
                let merged_span = span.merge(body.span());
                Ok(Expr::Lambda { params, body: Box::new(body), span: merged_span })
            }
            TokenKind::Ident(name) => {
                let name = name.clone();
                self.advance();
                Ok(Expr::Ident(name, span))
            }
            // Handle send/recv as identifiers when used as function names
            TokenKind::Send => {
                self.advance();
                Ok(Expr::Ident("send".to_string(), span))
            }
            TokenKind::Recv => {
                self.advance();
                Ok(Expr::Ident("recv".to_string(), span))
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

                // Check for empty tuple
                if self.match_token(&TokenKind::RParen) {
                    return Ok(Expr::Tuple { elements: vec![], span });
                }

                let expr = self.parse_expr(Precedence::MIN)?;

                // Check for tuple (including single-element tuple with trailing comma)
                if self.match_token(&TokenKind::Comma) {
                    let mut elements = vec![expr];
                    // Check for trailing comma (single-element tuple): (x,)
                    if self.match_token(&TokenKind::RParen) {
                        // Single-element tuple with trailing comma
                        let merged_span = span.merge(self.previous().span);
                        return Ok(Expr::Tuple { elements, span: merged_span });
                    }
                    // More elements follow - parse them
                    loop {
                        elements.push(self.parse_expr(Precedence::MIN)?);
                        if self.match_token(&TokenKind::Comma) {
                            // Check for trailing comma after multiple elements
                            if self.match_token(&TokenKind::RParen) {
                                let merged_span = span.merge(self.previous().span);
                                return Ok(Expr::Tuple { elements, span: merged_span });
                            }
                            // Continue parsing more elements
                        } else {
                            break;
                        }
                    }
                    self.expect(&TokenKind::RParen)?;
                    let merged_span = span.merge(self.previous().span);
                    return Ok(Expr::Tuple { elements, span: merged_span });
                }

                self.expect(&TokenKind::RParen)?;
                Ok(expr)
            }
            TokenKind::LBracket => {
                self.advance();
                let mut elements = Vec::new();
                let mut size: Option<usize> = None;

                // Handle empty list: []
                if self.match_token(&TokenKind::RBracket) {
                    let last_span = self.previous().span;
                    let merged_span = span.merge(last_span);
                    return Ok(Expr::List { elements, span: merged_span });
                }

                // Parse first element
                let first_elem = self.parse_expr(Precedence::MIN)?;

                // Check for list comprehension: [expr for var in iter]
                if matches!(self.current().kind, TokenKind::For) {
                    // This is a list comprehension
                    self.advance(); // consume 'for'

                    // Parse the target (can be identifier or tuple for unpacking)
                    let mut target = {
                        // Parse as identifier first
                        if let TokenKind::Ident(name) = &self.current().kind {
                            let name = name.clone();
                            self.advance();
                            Expr::Ident(name, span)
                        } else {
                            return crate::parser::parse_error(
                                "Expected variable name in list comprehension".to_string(),
                            );
                        }
                    };

                    // Check for tuple unpacking: for i, is_prime in ...
                    if matches!(self.current().kind, TokenKind::Comma) {
                        let mut elements = vec![target];
                        loop {
                            self.advance(); // consume comma
                            if let TokenKind::Ident(name) = &self.current().kind {
                                let name = name.clone();
                                self.advance();
                                elements.push(Expr::Ident(name, span));
                            } else {
                                return crate::parser::parse_error(
                                    "Expected variable name in tuple unpacking".to_string(),
                                );
                            }
                            if !matches!(self.current().kind, TokenKind::Comma) {
                                break;
                            }
                        }
                        let last_span = self.previous().span;
                        let merged_span = elements.first().unwrap().span().merge(last_span);
                        target = Expr::Tuple { elements, span: merged_span };
                    }

                    // Expect 'in' keyword
                    self.expect(&TokenKind::In)?;

                    // Parse the iterable
                    let iter = self.parse_expr(Precedence::MIN)?;

                    // Parse optional if clauses
                    let mut ifs = Vec::new();
                    while self.match_token(&TokenKind::If) {
                        ifs.push(self.parse_expr(Precedence::MIN)?);
                    }

                    // Expect closing bracket
                    self.expect(&TokenKind::RBracket)?;

                    let last_span = self.previous().span;
                    let merged_span = span.merge(last_span);

                    return Ok(Expr::ListComprehension {
                        element: Box::new(first_elem),
                        target: Box::new(target),
                        iter: Box::new(iter),
                        ifs,
                        span: merged_span,
                    });
                }

                // Not a list comprehension, treat as array/list
                elements.push(first_elem);

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
                            return crate::parser::parse_error(format!(
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

                let last_span = self.previous().span;
                let merged_span = span.merge(last_span);

                // Use Array node for fixed-size arrays, List for dynamic lists
                if size.is_some() {
                    Ok(Expr::Array { elements, size, span: merged_span })
                } else {
                    Ok(Expr::List { elements, span: merged_span })
                }
            }
            TokenKind::LBrace => {
                self.advance();
                let mut pairs = Vec::new();

                // Handle empty dict: {}
                if self.match_token(&TokenKind::RBrace) {
                    let last_span = self.previous().span;
                    let merged_span = span.merge(last_span);
                    return Ok(Expr::Dict { pairs, span: merged_span });
                }

                // Parse key-value pairs
                loop {
                    let key = self.parse_expr(Precedence::MIN)?;
                    self.expect(&TokenKind::Colon)?;
                    let value = self.parse_expr(Precedence::MIN)?;
                    pairs.push((key, value));

                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }

                    // Handle trailing comma: {key: value,}
                    if self.match_token(&TokenKind::RBrace) {
                        break;
                    }
                }

                self.expect(&TokenKind::RBrace)?;
                let last_span = self.previous().span;
                let merged_span = span.merge(last_span);

                Ok(Expr::Dict { pairs, span: merged_span })
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
                Ok(Expr::Await { future: Box::new(future), span: merged_span })
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
            TokenKind::PlusPlus => {
                // Prefix increment: ++x
                self.advance();
                let operand = self.parse_expr(Precedence::UNARY)?;
                let merged_span = span.merge(operand.span());
                Ok(Expr::UnaryOp {
                    op: UnaryOp::PreIncrement,
                    operand: Box::new(operand),
                    span: merged_span,
                })
            }
            TokenKind::MinusMinus => {
                // Prefix decrement: --x
                self.advance();
                let operand = self.parse_expr(Precedence::UNARY)?;
                let merged_span = span.merge(operand.span());
                Ok(Expr::UnaryOp {
                    op: UnaryOp::PreDecrement,
                    operand: Box::new(operand),
                    span: merged_span,
                })
            }
            _ => crate::parser::parse_error(format!(
                "Unexpected token in expression: {:?}",
                token.kind
            )),
        }
    }

    fn parse_infix_op(&mut self) -> crate::error::Result<BinOp> {
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
            TokenKind::DoubleQuestion => BinOp::NullCoalesce,
            _ => return crate::parser::parse_error("Unknown infix operator".to_string()),
        };

        Ok(op)
    }

    fn infix_precedence(&self) -> Option<Precedence> {
        let token = self.peek();

        let prec = match &token.kind {
            TokenKind::Or => Precedence::OR,
            TokenKind::And | TokenKind::DoubleQuestion => Precedence::AND,
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
            TokenKind::Pipeline => Precedence::PIPELINE,
            TokenKind::ColonEq => Precedence::ASSIGNMENT, // Walrus operator
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

    fn expect(&mut self, kind: &TokenKind) -> crate::error::Result<()> {
        if self.pos >= self.tokens.len() {
            return crate::parser::parse_error(format!(
                "Expected {:?}, but reached end of tokens",
                kind
            ));
        }
        if std::mem::discriminant(&self.current().kind) == std::mem::discriminant(kind) {
            self.advance();
            Ok(())
        } else {
            crate::parser::parse_error(format!(
                "Expressions: Expected {:?}, found {:?}",
                kind,
                self.current().kind
            ))
        }
    }

    /// Check if the bracket contents match a slice pattern (contains ':' before ']')
    /// Handles: [:], [start:], [:end], [start:end], [::step], etc.
    fn is_slice_pattern(&self) -> bool {
        // Look ahead through tokens to find ':' or ']'
        // We need to handle nested brackets/parens
        let mut pos = self.pos;
        let mut bracket_depth = 1;

        while pos < self.tokens.len() {
            match &self.tokens[pos].kind {
                TokenKind::Colon if bracket_depth == 1 => return true,
                TokenKind::RBracket | TokenKind::RParen => {
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
}
