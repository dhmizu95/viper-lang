#![allow(dead_code)]
use crate::lexer::indent_stack::{IndentChange, IndentStack};
use crate::lexer::tokens::{Token, TokenKind};
use crate::utils::Span;

/// The lexer scans source code and produces a stream of tokens
pub struct Lexer<'a> {
    source: &'a str,
    chars: std::iter::Peekable<std::str::Chars<'a>>,
    pos: usize,
    line: usize,
    column: usize,
    start_of_line: bool,
    indent_stack: IndentStack,
    pending_dedents: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.chars().peekable(),
            pos: 0,
            line: 1,
            column: 1,
            start_of_line: true,
            indent_stack: IndentStack::new(),
            pending_dedents: 0,
        }
    }

    /// Tokenize the entire source, returning all tokens
    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();

        loop {
            match self.next_token() {
                Ok(token) => {
                    let is_eof = matches!(token.kind, TokenKind::Eof);
                    tokens.push(token);
                    if is_eof {
                        break;
                    }
                }
                Err(e) => return Err(e),
            }
        }

        Ok(tokens)
    }

    /// Get the next token from the source
    pub fn next_token(&mut self) -> Result<Token, String> {
        'retry: loop {
            // Emit pending dedents first
            if self.pending_dedents > 0 {
                self.pending_dedents -= 1;
                let span = Span::new(self.pos, self.pos, self.line, self.column);
                return Ok(Token::new(TokenKind::Dedent, span));
            }

            // Handle indentation at start of line BEFORE consuming whitespace
            if self.start_of_line {
                self.start_of_line = false;

                // Count indentation (spaces/tabs at the start of line)
                let indent_start_pos = self.pos;
                let indent_start_line = self.line;
                let indent_start_column = self.column;

                let mut indent = 0;
                while let Some(&c) = self.chars.peek() {
                    if c == ' ' {
                        indent += 1;
                        self.advance();
                    } else if c == '\t' {
                        indent = (indent / 4 + 1) * 4; // Tab = 4 spaces
                        self.advance();
                    } else if c == '\n' {
                        // Empty line or line with only whitespace - skip it
                        self.advance();
                        self.start_of_line = true;
                        continue 'retry;
                    } else if c == '#' {
                        // Comment line - skip it
                        while let Some(&c) = self.chars.peek() {
                            if c == '\n' {
                                break;
                            }
                            self.advance();
                        }
                        // After comment, we'll see \n which will be handled on next iteration
                        continue 'retry;
                    } else {
                        break;
                    }
                }

                match self.indent_stack.process_indent(indent) {
                    IndentChange::Indent => {
                        let span = Span::new(
                            indent_start_pos,
                            indent_start_pos,
                            indent_start_line,
                            indent_start_column,
                        );
                        return Ok(Token::new(TokenKind::Indent, span));
                    }
                    IndentChange::DedentCount(count) => {
                        // Emit one dedent now, rest will be emitted on subsequent calls
                        self.pending_dedents = count - 1;
                        let span = Span::new(
                            indent_start_pos,
                            indent_start_pos,
                            indent_start_line,
                            indent_start_column,
                        );
                        return Ok(Token::new(TokenKind::Dedent, span));
                    }
                    IndentChange::Dedent => {
                        let span = Span::new(
                            indent_start_pos,
                            indent_start_pos,
                            indent_start_line,
                            indent_start_column,
                        );
                        return Ok(Token::new(TokenKind::Dedent, span));
                    }
                    IndentChange::Error(msg) => return Err(msg),
                    IndentChange::None => {}
                }
            }

            // Skip whitespace (not at start of line - already handled above)
            loop {
                match self.chars.peek() {
                    Some(&c) if c == ' ' || c == '\t' => {
                        self.advance();
                    }
                    Some(&c) if c == '\n' => {
                        self.advance();
                        self.start_of_line = true;
                        // After newline, restart to handle indentation
                        continue 'retry;
                    }
                    Some(&c) if c == '#' => {
                        // Skip comments
                        while let Some(&c) = self.chars.peek() {
                            if c == '\n' {
                                break;
                            }
                            self.advance();
                        }
                    }
                    _ => break,
                }
            }

            // Check for EOF
            if self.chars.peek().is_none() {
                // Emit remaining dedents
                if self.indent_stack.depth() > 0 {
                    self.indent_stack.reset();
                    let span = Span::new(self.pos, self.pos, self.line, self.column);
                    return Ok(Token::eof(span));
                }
                let span = Span::new(self.pos, self.pos, self.line, self.column);
                return Ok(Token::eof(span));
            }

            let start_pos = self.pos;
            let start_line = self.line;
            let start_column = self.column;

            let c = self.advance();

            let kind = match c {
                // Single-character tokens
                '(' => TokenKind::LParen,
                ')' => TokenKind::RParen,
                '[' => TokenKind::LBracket,
                ']' => TokenKind::RBracket,
                '{' => TokenKind::LBrace,
                '}' => TokenKind::RBrace,
                ',' => TokenKind::Comma,
                ':' => TokenKind::Colon,
                '.' => TokenKind::Dot,
                '@' => TokenKind::At,
                '+' => TokenKind::Plus,
                '-' => {
                    if self.peek() == Some('>') {
                        self.advance();
                        TokenKind::Arrow
                    } else {
                        TokenKind::Minus
                    }
                }
                '%' => TokenKind::Percent,

                // Potentially double-character tokens
                '*' => {
                    if self.peek() == Some('*') {
                        self.advance();
                        TokenKind::DoubleStar
                    } else {
                        TokenKind::Star
                    }
                }
                '/' => {
                    if self.peek() == Some('/') {
                        self.advance();
                        TokenKind::DoubleSlash
                    } else {
                        TokenKind::Slash
                    }
                }
                '=' => {
                    if self.peek() == Some('=') {
                        self.advance();
                        TokenKind::EqEq
                    } else if self.peek() == Some('>') {
                        self.advance();
                        TokenKind::Arrow
                    } else {
                        TokenKind::Eq
                    }
                }
                '!' => {
                    if self.peek() == Some('=') {
                        self.advance();
                        TokenKind::NotEq
                    } else {
                        return Ok(Token::error(
                            "Expected '=' after '!'".to_string(),
                            Span::new(start_pos, self.pos, start_line, start_column),
                        ));
                    }
                }
                '<' => {
                    if self.peek() == Some('=') {
                        self.advance();
                        TokenKind::LtEq
                    } else {
                        TokenKind::Lt
                    }
                }
                '>' => {
                    if self.peek() == Some('=') {
                        self.advance();
                        TokenKind::GtEq
                    } else if self.peek() == Some('>') {
                        self.advance();
                        TokenKind::GtGt
                    } else {
                        TokenKind::Gt
                    }
                }

                // Bitwise operators (Phase 2)
                '&' => TokenKind::Ampersand,
                '|' => TokenKind::Pipe,
                '^' => TokenKind::Caret,
                '~' => TokenKind::Tilde,

                // String literals
                '"' | '\'' => {
                    // Check for triple-quoted string (block comment/docstring)
                    let quote_char = c;
                    if self.peek() == Some(quote_char) {
                        self.advance(); // consume second quote
                        if self.peek() == Some(quote_char) {
                            self.advance(); // consume third quote
                            // Skip until closing triple quote
                            let mut found_end = false;
                            while let Some(ch) = self.chars.peek() {
                                if *ch == quote_char {
                                    self.advance();
                                    // Check for two more quotes
                                    if self.peek() == Some(quote_char) {
                                        self.advance();
                                        if self.peek() == Some(quote_char) {
                                            self.advance();
                                            found_end = true;
                                            break;
                                        }
                                    }
                                } else {
                                    self.advance();
                                }
                            }
                            if !found_end {
                                return Err("Unterminated triple-quoted string".to_string());
                            }
                            // Triple-quoted strings are treated as comments (ignored)
                            continue;
                        }
                    }
                    let s = self.read_string(quote_char)?;
                    TokenKind::Str(s)
                }

                // Number literals
                c if c.is_ascii_digit() => self.read_number(c)?,

                // Identifiers and keywords
                c if c.is_alphabetic() || c == '_' => {
                    let mut ident = c.to_string();
                    while let Some(c) = self.peek() {
                        if c.is_alphanumeric() || c == '_' {
                            ident.push(self.advance());
                        } else {
                            break;
                        }
                    }
                    self.keyword_or_ident(ident)
                }

                // Newline - should not reach here due to whitespace handling
                '\n' => {
                    self.start_of_line = true;
                    TokenKind::Newline
                }

                c => {
                    return Ok(Token::error(
                        format!("Unexpected character: '{}'", c),
                        Span::new(start_pos, self.pos, start_line, start_column),
                    ));
                }
            };

            let span = Span::new(start_pos, self.pos, start_line, start_column);
            return Ok(Token::new(kind, span));
        }
    }

    fn advance(&mut self) -> char {
        let c = self.chars.next().unwrap();
        self.pos += c.len_utf8();
        if c == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        c
    }

    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    fn read_string(&mut self, quote: char) -> Result<String, String> {
        let mut s = String::new();
        let mut escaped = false;

        loop {
            match self.chars.peek() {
                None => return Err("Unterminated string".to_string()),
                Some(&c) => {
                    let ch = c;
                    if escaped {
                        match ch {
                            'n' => s.push('\n'),
                            't' => s.push('\t'),
                            'r' => s.push('\r'),
                            '\\' => s.push('\\'),
                            '\'' => s.push('\''),
                            '"' => s.push('"'),
                            'x' => {
                                // Hex escape: \x41
                                self.advance(); // consume 'x'
                                let mut hex = String::new();
                                for _ in 0..2 {
                                    if let Some(h) = self.peek() {
                                        if h.is_ascii_hexdigit() {
                                            hex.push(self.advance());
                                        } else {
                                            break;
                                        }
                                    }
                                }
                                if hex.len() == 2 {
                                    let code = u8::from_str_radix(&hex, 16)
                                        .map_err(|_| format!("Invalid hex escape: \\x{}", hex))?;
                                    s.push(code as char);
                                } else {
                                    return Err(format!("Invalid hex escape: \\x{}", hex));
                                }
                            }
                            _ => s.push(ch),
                        }
                        escaped = false;
                        self.advance();
                    } else if ch == '\\' {
                        escaped = true;
                        self.advance();
                    } else if ch == quote {
                        self.advance();
                        break;
                    } else {
                        s.push(self.advance());
                    }
                }
            }
        }

        Ok(s)
    }

    fn read_number(&mut self, first: char) -> Result<TokenKind, String> {
        let mut s = first.to_string();
        let mut is_float = false;

        // Check for hex literal (0x, 0X)
        if first == '0' && self.peek() == Some('x') || self.peek() == Some('X') {
            s.push(self.advance()); // consume 'x' or 'X'
            // Read hex digits
            while let Some(c) = self.peek() {
                if c.is_ascii_hexdigit() {
                    s.push(self.advance());
                } else {
                    break;
                }
            }
            // Parse as hex integer
            let value = i64::from_str_radix(&s[2..], 16)
                .map_err(|_| format!("Invalid hex literal: {}", s))?;
            return Ok(TokenKind::Int(value));
        }

        // Read decimal number
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                s.push(self.advance());
            } else if c == '.' && !is_float {
                // Check if next char is a digit (to distinguish from method call)
                self.advance();
                if self.peek().map_or(false, |c| c.is_ascii_digit()) {
                    s.push('.');
                    is_float = true;
                } else {
                    // It's an integer followed by a dot - probably a method call
                    // Put back the dot by not consuming it
                    self.pos -= 1;
                    self.column -= 1;
                    break;
                }
            } else if (c == 'e' || c == 'E') && !is_float {
                // Scientific notation: 1e10, 1E10, 1e-10, 1e+10
                self.advance();
                s.push('e');
                
                // Optional sign
                if let Some(sign) = self.peek() {
                    if sign == '+' || sign == '-' {
                        s.push(self.advance());
                    }
                }
                
                // Exponent digits (required)
                let mut has_exp_digits = false;
                while let Some(c) = self.peek() {
                    if c.is_ascii_digit() {
                        s.push(self.advance());
                        has_exp_digits = true;
                    } else {
                        break;
                    }
                }
                
                if !has_exp_digits {
                    return Err("Scientific notation requires exponent digits".to_string());
                }
                
                is_float = true;
            } else {
                break;
            }
        }

        if is_float {
            Ok(TokenKind::Float(s.parse().unwrap()))
        } else {
            Ok(TokenKind::Int(s.parse().unwrap()))
        }
    }

    fn keyword_or_ident(&self, ident: String) -> TokenKind {
        match ident.as_str() {
            "def" => TokenKind::Def,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "elif" => TokenKind::Elif,
            "while" => TokenKind::While,
            "for" => TokenKind::For,
            "in" => TokenKind::In,
            "return" => TokenKind::Return,
            "pass" => TokenKind::Pass,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "True" => TokenKind::True,
            "False" => TokenKind::False,
            "None" => TokenKind::None,
            "mut" => TokenKind::Mut,
            "sync" => TokenKind::Sync,
            "task" => TokenKind::Task,
            "try" => TokenKind::Try,
            "except" => TokenKind::Except,
            "finally" => TokenKind::Finally,
            "as" => TokenKind::As,
            "class" => TokenKind::Class,
            "import" => TokenKind::Import,
            "from" => TokenKind::From,
            "is" => TokenKind::Is,
            "and" => TokenKind::And,
            "or" => TokenKind::Or,
            "not" => TokenKind::Not,
            "global" => TokenKind::Ident("global".to_string()),  // Reserved for Phase 3
            "const" => TokenKind::Ident("const".to_string()),    // Reserved for Phase 2
            "lambda" => TokenKind::Ident("lambda".to_string()),  // Phase 2
            "yield" => TokenKind::Ident("yield".to_string()),    // Phase 3
            _ => TokenKind::Ident(ident),
        }
    }
}
