//! Unit tests for the Viper lexer module
//! Tests for: IndentStack, TokenKind::Display, Token constructors, Lexer::tokenize

use viper_lang::lexer::{IndentChange, IndentStack, Lexer, Token, TokenKind};
use viper_lang::utils::Span;

// ============================================================================
// IndentStack Tests
// ============================================================================

#[test]
fn test_indent_stack_new() {
    let stack = IndentStack::new();
    assert_eq!(stack.current(), 0);
    assert_eq!(stack.depth(), 0);
}

#[test]
fn test_indent_stack_process_indent_same() {
    let mut stack = IndentStack::new();
    let result = stack.process_indent(0);
    assert_eq!(result, IndentChange::None);
    assert_eq!(stack.current(), 0);
}

#[test]
fn test_indent_stack_process_indent_increase() {
    let mut stack = IndentStack::new();
    let result = stack.process_indent(4);
    assert_eq!(result, IndentChange::Indent);
    assert_eq!(stack.current(), 4);
    assert_eq!(stack.depth(), 1);
}

#[test]
fn test_indent_stack_process_indent_double_increase() {
    let mut stack = IndentStack::new();
    stack.process_indent(4);
    let result = stack.process_indent(8);
    assert_eq!(result, IndentChange::Indent);
    assert_eq!(stack.current(), 8);
    assert_eq!(stack.depth(), 2);
}

#[test]
fn test_indent_stack_process_dedent_single() {
    let mut stack = IndentStack::new();
    stack.process_indent(4);
    let result = stack.process_indent(0);
    // When popping a single level, returns DedentCount(1), not Dedent
    assert_eq!(result, IndentChange::DedentCount(1));
    assert_eq!(stack.current(), 0);
    assert_eq!(stack.depth(), 0);
}

#[test]
fn test_indent_stack_process_dedent_multiple() {
    let mut stack = IndentStack::new();
    stack.process_indent(4);
    stack.process_indent(8);
    let result = stack.process_indent(0);
    assert_eq!(result, IndentChange::DedentCount(2));
    assert_eq!(stack.current(), 0);
    assert_eq!(stack.depth(), 0);
}

#[test]
fn test_indent_stack_process_dedent_error() {
    let mut stack = IndentStack::new();
    stack.process_indent(4);
    let result = stack.process_indent(2);
    assert!(matches!(result, IndentChange::Error(_)));
}

#[test]
fn test_indent_stack_reset() {
    let mut stack = IndentStack::new();
    stack.process_indent(4);
    stack.process_indent(8);
    stack.reset();
    assert_eq!(stack.current(), 0);
    assert_eq!(stack.depth(), 0);
}

#[test]
fn test_indent_stack_depth() {
    let mut stack = IndentStack::new();
    assert_eq!(stack.depth(), 0);
    stack.process_indent(4);
    assert_eq!(stack.depth(), 1);
    stack.process_indent(8);
    assert_eq!(stack.depth(), 2);
    stack.process_indent(4);
    assert_eq!(stack.depth(), 1);
}

// ============================================================================
// TokenKind::Display Tests
// ============================================================================

#[test]
fn test_token_kind_display_literals() {
    assert_eq!(format!("{}", TokenKind::Int(42)), "Int(42)");
    assert_eq!(format!("{}", TokenKind::Float(3.14)), "Float(3.14)");
    assert_eq!(format!("{}", TokenKind::Str("hello".to_string())), "Str(hello)");
    assert_eq!(format!("{}", TokenKind::FString("world".to_string())), "FString(world)");
    assert_eq!(format!("{}", TokenKind::Bool(true)), "Bool(true)");
    assert_eq!(format!("{}", TokenKind::Bool(false)), "Bool(false)");
    assert_eq!(format!("{}", TokenKind::Ident("x".to_string())), "Ident(x)");
}

#[test]
fn test_token_kind_display_keywords() {
    assert_eq!(format!("{}", TokenKind::Def), "def");
    assert_eq!(format!("{}", TokenKind::If), "if");
    assert_eq!(format!("{}", TokenKind::Else), "else");
    assert_eq!(format!("{}", TokenKind::Elif), "elif");
    assert_eq!(format!("{}", TokenKind::While), "while");
    assert_eq!(format!("{}", TokenKind::For), "for");
    assert_eq!(format!("{}", TokenKind::In), "in");
    assert_eq!(format!("{}", TokenKind::Return), "return");
    assert_eq!(format!("{}", TokenKind::Pass), "pass");
    assert_eq!(format!("{}", TokenKind::Break), "break");
    assert_eq!(format!("{}", TokenKind::Continue), "continue");
    assert_eq!(format!("{}", TokenKind::True), "True");
    assert_eq!(format!("{}", TokenKind::False), "False");
    assert_eq!(format!("{}", TokenKind::None), "None");
    assert_eq!(format!("{}", TokenKind::Void), "void");
    assert_eq!(format!("{}", TokenKind::Mut), "mut");
    assert_eq!(format!("{}", TokenKind::Sync), "sync");
    assert_eq!(format!("{}", TokenKind::Task), "task");
    assert_eq!(format!("{}", TokenKind::Try), "try");
    assert_eq!(format!("{}", TokenKind::Except), "except");
    assert_eq!(format!("{}", TokenKind::Finally), "finally");
    assert_eq!(format!("{}", TokenKind::As), "as");
    assert_eq!(format!("{}", TokenKind::Class), "class");
    assert_eq!(format!("{}", TokenKind::Import), "import");
    assert_eq!(format!("{}", TokenKind::From), "from");
    assert_eq!(format!("{}", TokenKind::Async), "async");
    assert_eq!(format!("{}", TokenKind::Await), "await");
    assert_eq!(format!("{}", TokenKind::Struct), "struct");
    assert_eq!(format!("{}", TokenKind::Extern), "extern");
    assert_eq!(format!("{}", TokenKind::Match), "match");
    assert_eq!(format!("{}", TokenKind::Case), "case");
    assert_eq!(format!("{}", TokenKind::Underscore), "_");
    assert_eq!(format!("{}", TokenKind::Select), "select");
    assert_eq!(format!("{}", TokenKind::Recv), "recv");
    assert_eq!(format!("{}", TokenKind::Send), "send");
    assert_eq!(format!("{}", TokenKind::Unless), "unless");
    assert_eq!(format!("{}", TokenKind::Pipeline), "|>");
    assert_eq!(format!("{}", TokenKind::DotDot), "..");
    assert_eq!(format!("{}", TokenKind::Global), "global");
    assert_eq!(format!("{}", TokenKind::Const), "const");
    assert_eq!(format!("{}", TokenKind::Type), "type");
    assert_eq!(format!("{}", TokenKind::Tuple), "tuple");
    assert_eq!(format!("{}", TokenKind::Optional), "Optional");
}

#[test]
fn test_token_kind_display_operators() {
    assert_eq!(format!("{}", TokenKind::Plus), "+");
    assert_eq!(format!("{}", TokenKind::Minus), "-");
    assert_eq!(format!("{}", TokenKind::Star), "*");
    assert_eq!(format!("{}", TokenKind::Slash), "/");
    assert_eq!(format!("{}", TokenKind::Percent), "%");
    assert_eq!(format!("{}", TokenKind::DoubleStar), "**");
    assert_eq!(format!("{}", TokenKind::DoubleSlash), "//");
    assert_eq!(format!("{}", TokenKind::PlusPlus), "++");
    assert_eq!(format!("{}", TokenKind::MinusMinus), "--");
    assert_eq!(format!("{}", TokenKind::PlusEq), "+=");
    assert_eq!(format!("{}", TokenKind::MinusEq), "-=");
    assert_eq!(format!("{}", TokenKind::StarEq), "*=");
    assert_eq!(format!("{}", TokenKind::SlashEq), "/=");
    assert_eq!(format!("{}", TokenKind::PercentEq), "%=");
    assert_eq!(format!("{}", TokenKind::DoubleStarEq), "**=");
    assert_eq!(format!("{}", TokenKind::DoubleSlashEq), "//=");
    assert_eq!(format!("{}", TokenKind::Eq), "=");
    assert_eq!(format!("{}", TokenKind::EqEq), "==");
    assert_eq!(format!("{}", TokenKind::NotEq), "!=");
    assert_eq!(format!("{}", TokenKind::Lt), "<");
    assert_eq!(format!("{}", TokenKind::LtEq), "<=");
    assert_eq!(format!("{}", TokenKind::Gt), ">");
    assert_eq!(format!("{}", TokenKind::GtEq), ">=");
    assert_eq!(format!("{}", TokenKind::And), "and");
    assert_eq!(format!("{}", TokenKind::Or), "or");
    assert_eq!(format!("{}", TokenKind::Not), "not");
    assert_eq!(format!("{}", TokenKind::Ampersand), "&");
    assert_eq!(format!("{}", TokenKind::Pipe), "|");
    assert_eq!(format!("{}", TokenKind::Caret), "^");
    assert_eq!(format!("{}", TokenKind::Tilde), "~");
    assert_eq!(format!("{}", TokenKind::LtLt), "<<");
    assert_eq!(format!("{}", TokenKind::GtGt), ">>");
    assert_eq!(format!("{}", TokenKind::Is), "is");
    assert_eq!(format!("{}", TokenKind::IsNot), "is not");
    assert_eq!(format!("{}", TokenKind::NotIn), "not in");
    assert_eq!(format!("{}", TokenKind::Question), "?");
    assert_eq!(format!("{}", TokenKind::Lambda), "lambda");
    assert_eq!(format!("{}", TokenKind::Fn), "fn");
}

#[test]
fn test_token_kind_display_delimiters() {
    assert_eq!(format!("{}", TokenKind::LParen), "(");
    assert_eq!(format!("{}", TokenKind::RParen), ")");
    assert_eq!(format!("{}", TokenKind::LBracket), "[");
    assert_eq!(format!("{}", TokenKind::RBracket), "]");
    assert_eq!(format!("{}", TokenKind::LBrace), "{");
    assert_eq!(format!("{}", TokenKind::RBrace), "}");
    assert_eq!(format!("{}", TokenKind::Comma), ",");
    assert_eq!(format!("{}", TokenKind::Colon), ":");
    assert_eq!(format!("{}", TokenKind::Semi), ";");
    assert_eq!(format!("{}", TokenKind::Dot), ".");
    assert_eq!(format!("{}", TokenKind::Arrow), "->");
    assert_eq!(format!("{}", TokenKind::At), "@");
}

#[test]
fn test_token_kind_display_special() {
    assert_eq!(format!("{}", TokenKind::Indent), "<INDENT>");
    assert_eq!(format!("{}", TokenKind::Dedent), "<DEDENT>");
    assert_eq!(format!("{}", TokenKind::Newline), "<NEWLINE>");
    assert_eq!(format!("{}", TokenKind::Eof), "<EOF>");
    assert_eq!(format!("{}", TokenKind::Error("test error".to_string())), "<ERROR: test error>");
}

// ============================================================================
// Token Tests
// ============================================================================

#[test]
fn test_token_new() {
    let span = Span::new(0, 5, 1, 1);
    let token = Token::new(TokenKind::Int(42), span);
    assert!(matches!(token.kind, TokenKind::Int(42)));
    assert_eq!(token.span, span);
}

#[test]
fn test_token_eof() {
    let span = Span::new(0, 0, 1, 1);
    let token = Token::eof(span);
    assert!(matches!(token.kind, TokenKind::Eof));
    assert_eq!(token.span, span);
}

#[test]
fn test_token_error() {
    let span = Span::new(0, 5, 1, 1);
    let token = Token::error("test error".to_string(), span);
    assert!(matches!(token.kind, TokenKind::Error(ref msg) if msg == "test error"));
    assert_eq!(token.span, span);
}

// ============================================================================
// Lexer::tokenize Tests
// ============================================================================

fn tokenize(src: &str) -> Result<Vec<Token>, String> {
    let mut lexer = Lexer::new(src);
    lexer.tokenize()
}

fn token_kinds(src: &str) -> Result<Vec<TokenKind>, String> {
    tokenize(src).map(|tokens| tokens.into_iter().map(|t| t.kind).collect())
}

// --- Literal Tests ---

#[test]
fn test_lexer_integer() {
    let kinds = token_kinds("42").unwrap();
    assert_eq!(kinds.len(), 2); // Int + Eof
    assert!(matches!(kinds[0], TokenKind::Int(42)));
    assert!(matches!(kinds[1], TokenKind::Eof));
}

#[test]
fn test_lexer_integer_large() {
    let kinds = token_kinds("123456789").unwrap();
    assert!(matches!(kinds[0], TokenKind::Int(123456789)));
}

#[test]
fn test_lexer_float() {
    let kinds = token_kinds("3.14").unwrap();
    assert!(matches!(kinds[0], TokenKind::Float(f) if (f - 3.14).abs() < f64::EPSILON));
}

#[test]
fn test_lexer_float_negative_exponent() {
    let kinds = token_kinds("1e-10").unwrap();
    assert!(matches!(kinds[0], TokenKind::Float(_)));
}

#[test]
fn test_lexer_float_positive_exponent() {
    let kinds = token_kinds("1e10").unwrap();
    assert!(matches!(kinds[0], TokenKind::Float(_)));
}

#[test]
fn test_lexer_string_simple() {
    let kinds = token_kinds(r#""hello""#).unwrap();
    assert!(matches!(&kinds[0], TokenKind::Str(s) if s == "hello"));
}

#[test]
fn test_lexer_string_escape_sequence() {
    let kinds = token_kinds(r#""hello\nworld""#).unwrap();
    assert!(matches!(&kinds[0], TokenKind::Str(s) if s == "hello\nworld"));
}

#[test]
fn test_lexer_string_escape_tab() {
    let kinds = token_kinds(r#""hello\tworld""#).unwrap();
    assert!(matches!(&kinds[0], TokenKind::Str(s) if s == "hello\tworld"));
}

#[test]
fn test_lexer_string_escape_quote() {
    let kinds = token_kinds(r#""hello\"world""#).unwrap();
    assert!(matches!(&kinds[0], TokenKind::Str(s) if s == "hello\"world"));
}

#[test]
fn test_lexer_string_hex_escape() {
    let kinds = token_kinds(r#""\x41\x42\x43""#).unwrap();
    assert!(matches!(&kinds[0], TokenKind::Str(s) if s == "ABC"));
}

#[test]
fn test_lexer_fstring_simple() {
    let kinds = token_kinds(r#"f"hello {name}""#).unwrap();
    assert!(matches!(&kinds[0], TokenKind::FString(s) if s == "hello {name}"));
}

#[test]
fn test_lexer_fstring_expression() {
    let kinds = token_kinds(r#"f"x = {x}""#).unwrap();
    assert!(matches!(&kinds[0], TokenKind::FString(s) if s == "x = {x}"));
}

#[test]
fn test_lexer_boolean_true() {
    let kinds = token_kinds("True").unwrap();
    assert_eq!(kinds[0], TokenKind::True);
}

#[test]
fn test_lexer_boolean_false() {
    let kinds = token_kinds("False").unwrap();
    assert_eq!(kinds[0], TokenKind::False);
}

#[test]
fn test_lexer_none() {
    let kinds = token_kinds("None").unwrap();
    assert!(matches!(kinds[0], TokenKind::None));
}

#[test]
fn test_lexer_keyword_def() {
    let kinds = token_kinds("def").unwrap();
    assert_eq!(kinds[0], TokenKind::Def);
}

#[test]
fn test_lexer_keyword_if() {
    let kinds = token_kinds("if").unwrap();
    assert_eq!(kinds[0], TokenKind::If);
}

#[test]
fn test_lexer_keyword_for() {
    let kinds = token_kinds("for").unwrap();
    assert_eq!(kinds[0], TokenKind::For);
}

#[test]
fn test_lexer_keyword_while() {
    let kinds = token_kinds("while").unwrap();
    assert_eq!(kinds[0], TokenKind::While);
}

#[test]
fn test_lexer_keyword_return() {
    let kinds = token_kinds("return").unwrap();
    assert_eq!(kinds[0], TokenKind::Return);
}

#[test]
fn test_lexer_keyword_in() {
    let kinds = token_kinds("in").unwrap();
    assert_eq!(kinds[0], TokenKind::In);
}

#[test]
fn test_lexer_keyword_and() {
    let kinds = token_kinds("and").unwrap();
    assert_eq!(kinds[0], TokenKind::And);
}

#[test]
fn test_lexer_keyword_or() {
    let kinds = token_kinds("or").unwrap();
    assert_eq!(kinds[0], TokenKind::Or);
}

#[test]
fn test_lexer_keyword_not() {
    let kinds = token_kinds("not").unwrap();
    assert_eq!(kinds[0], TokenKind::Not);
}

#[test]
fn test_lexer_keyword_match() {
    let kinds = token_kinds("match").unwrap();
    assert_eq!(kinds[0], TokenKind::Match);
}

#[test]
fn test_lexer_keyword_case() {
    let kinds = token_kinds("case").unwrap();
    assert_eq!(kinds[0], TokenKind::Case);
}

#[test]
fn test_lexer_keyword_lambda() {
    let kinds = token_kinds("lambda").unwrap();
    assert_eq!(kinds[0], TokenKind::Lambda);
}

#[test]
fn test_lexer_keyword_extern() {
    let kinds = token_kinds("extern").unwrap();
    assert_eq!(kinds[0], TokenKind::Extern);
}

#[test]
fn test_lexer_keyword_struct() {
    let kinds = token_kinds("struct").unwrap();
    assert_eq!(kinds[0], TokenKind::Struct);
}

#[test]
fn test_lexer_keyword_async() {
    let kinds = token_kinds("async").unwrap();
    assert_eq!(kinds[0], TokenKind::Async);
}

#[test]
fn test_lexer_keyword_await() {
    let kinds = token_kinds("await").unwrap();
    assert_eq!(kinds[0], TokenKind::Await);
}

// --- Operator Tests ---

#[test]
fn test_lexer_operator_plus() {
    let kinds = token_kinds("+").unwrap();
    assert_eq!(kinds[0], TokenKind::Plus);
}

#[test]
fn test_lexer_operator_minus() {
    let kinds = token_kinds("-").unwrap();
    assert_eq!(kinds[0], TokenKind::Minus);
}

#[test]
fn test_lexer_operator_star() {
    let kinds = token_kinds("*").unwrap();
    assert_eq!(kinds[0], TokenKind::Star);
}

#[test]
fn test_lexer_operator_slash() {
    let kinds = token_kinds("/").unwrap();
    assert_eq!(kinds[0], TokenKind::Slash);
}

#[test]
fn test_lexer_operator_percent() {
    let kinds = token_kinds("%").unwrap();
    assert_eq!(kinds[0], TokenKind::Percent);
}

#[test]
fn test_lexer_operator_double_star() {
    let kinds = token_kinds("**").unwrap();
    assert_eq!(kinds[0], TokenKind::DoubleStar);
}

#[test]
fn test_lexer_operator_double_slash() {
    let kinds = token_kinds("//").unwrap();
    assert_eq!(kinds[0], TokenKind::DoubleSlash);
}

#[test]
fn test_lexer_operator_plus_plus() {
    let kinds = token_kinds("++").unwrap();
    assert_eq!(kinds[0], TokenKind::PlusPlus);
}

#[test]
fn test_lexer_operator_minus_minus() {
    let kinds = token_kinds("--").unwrap();
    assert_eq!(kinds[0], TokenKind::MinusMinus);
}

#[test]
fn test_lexer_operator_plus_eq() {
    let kinds = token_kinds("+=").unwrap();
    assert_eq!(kinds[0], TokenKind::PlusEq);
}

#[test]
fn test_lexer_operator_minus_eq() {
    let kinds = token_kinds("-=").unwrap();
    assert_eq!(kinds[0], TokenKind::MinusEq);
}

#[test]
fn test_lexer_operator_star_eq() {
    let kinds = token_kinds("*=").unwrap();
    assert_eq!(kinds[0], TokenKind::StarEq);
}

#[test]
fn test_lexer_operator_slash_eq() {
    let kinds = token_kinds("/=").unwrap();
    assert_eq!(kinds[0], TokenKind::SlashEq);
}

#[test]
fn test_lexer_operator_percent_eq() {
    let kinds = token_kinds("%=").unwrap();
    assert_eq!(kinds[0], TokenKind::PercentEq);
}

#[test]
fn test_lexer_operator_double_star_eq() {
    let kinds = token_kinds("**=").unwrap();
    assert_eq!(kinds[0], TokenKind::DoubleStarEq);
}

#[test]
fn test_lexer_operator_double_slash_eq() {
    let kinds = token_kinds("//=").unwrap();
    assert_eq!(kinds[0], TokenKind::DoubleSlashEq);
}

#[test]
fn test_lexer_operator_eq() {
    let kinds = token_kinds("=").unwrap();
    assert_eq!(kinds[0], TokenKind::Eq);
}

#[test]
fn test_lexer_operator_eq_eq() {
    let kinds = token_kinds("==").unwrap();
    assert_eq!(kinds[0], TokenKind::EqEq);
}

#[test]
fn test_lexer_operator_not_eq() {
    let kinds = token_kinds("!=").unwrap();
    assert_eq!(kinds[0], TokenKind::NotEq);
}

#[test]
fn test_lexer_operator_lt() {
    let kinds = token_kinds("<").unwrap();
    assert_eq!(kinds[0], TokenKind::Lt);
}

#[test]
fn test_lexer_operator_lt_eq() {
    let kinds = token_kinds("<=").unwrap();
    assert_eq!(kinds[0], TokenKind::LtEq);
}

#[test]
fn test_lexer_operator_gt() {
    let kinds = token_kinds(">").unwrap();
    assert_eq!(kinds[0], TokenKind::Gt);
}

#[test]
fn test_lexer_operator_gt_eq() {
    let kinds = token_kinds(">=").unwrap();
    assert_eq!(kinds[0], TokenKind::GtEq);
}

#[test]
fn test_lexer_operator_ampersand() {
    let kinds = token_kinds("&").unwrap();
    assert_eq!(kinds[0], TokenKind::Ampersand);
}

#[test]
fn test_lexer_operator_pipe() {
    let kinds = token_kinds("|").unwrap();
    assert_eq!(kinds[0], TokenKind::Pipe);
}

#[test]
fn test_lexer_operator_caret() {
    let kinds = token_kinds("^").unwrap();
    assert_eq!(kinds[0], TokenKind::Caret);
}

#[test]
fn test_lexer_operator_tilde() {
    let kinds = token_kinds("~").unwrap();
    assert_eq!(kinds[0], TokenKind::Tilde);
}

#[test]
fn test_lexer_operator_lt_lt() {
    let kinds = token_kinds("<<").unwrap();
    assert_eq!(kinds[0], TokenKind::LtLt);
}

#[test]
fn test_lexer_operator_gt_gt() {
    let kinds = token_kinds(">>").unwrap();
    assert_eq!(kinds[0], TokenKind::GtGt);
}

#[test]
fn test_lexer_operator_pipeline() {
    let kinds = token_kinds("|>").unwrap();
    assert_eq!(kinds[0], TokenKind::Pipeline);
}

#[test]
fn test_lexer_operator_is() {
    let kinds = token_kinds("is").unwrap();
    assert_eq!(kinds[0], TokenKind::Is);
}

#[test]
fn test_lexer_operator_is_not() {
    let kinds = token_kinds("is not").unwrap();
    assert_eq!(kinds[0], TokenKind::IsNot);
}

#[test]
fn test_lexer_operator_in() {
    let kinds = token_kinds("in").unwrap();
    assert_eq!(kinds[0], TokenKind::In);
}

#[test]
fn test_lexer_operator_not_in() {
    let kinds = token_kinds("not in").unwrap();
    assert_eq!(kinds[0], TokenKind::NotIn);
}

#[test]
fn test_lexer_operator_dot_dot() {
    let kinds = token_kinds("..").unwrap();
    // Lexer returns two separate Dot tokens
    assert_eq!(kinds[0], TokenKind::Dot);
    assert_eq!(kinds[1], TokenKind::Dot);
}

// --- Delimiter Tests ---

#[test]
fn test_lexer_delim_lparen() {
    let kinds = token_kinds("(").unwrap();
    assert_eq!(kinds[0], TokenKind::LParen);
}

#[test]
fn test_lexer_delim_rparen() {
    let kinds = token_kinds(")").unwrap();
    assert_eq!(kinds[0], TokenKind::RParen);
}

#[test]
fn test_lexer_delim_lbracket() {
    let kinds = token_kinds("[").unwrap();
    assert_eq!(kinds[0], TokenKind::LBracket);
}

#[test]
fn test_lexer_delim_rbracket() {
    let kinds = token_kinds("]").unwrap();
    assert_eq!(kinds[0], TokenKind::RBracket);
}

#[test]
fn test_lexer_delim_lbrace() {
    let kinds = token_kinds("{").unwrap();
    assert_eq!(kinds[0], TokenKind::LBrace);
}

#[test]
fn test_lexer_delim_rbrace() {
    let kinds = token_kinds("}").unwrap();
    assert_eq!(kinds[0], TokenKind::RBrace);
}

#[test]
fn test_lexer_delim_comma() {
    let kinds = token_kinds(",").unwrap();
    assert_eq!(kinds[0], TokenKind::Comma);
}

#[test]
fn test_lexer_delim_colon() {
    let kinds = token_kinds(":").unwrap();
    assert_eq!(kinds[0], TokenKind::Colon);
}

#[test]
fn test_lexer_delim_semi() {
    let kinds = token_kinds(";").unwrap();
    assert_eq!(kinds[0], TokenKind::Semi);
}

#[test]
fn test_lexer_delim_dot() {
    let kinds = token_kinds(".").unwrap();
    assert_eq!(kinds[0], TokenKind::Dot);
}

#[test]
fn test_lexer_delim_arrow() {
    let kinds = token_kinds("->").unwrap();
    assert_eq!(kinds[0], TokenKind::Arrow);
}

#[test]
fn test_lexer_delim_at() {
    let kinds = token_kinds("@").unwrap();
    assert_eq!(kinds[0], TokenKind::At);
}

// --- Identifier Tests ---

#[test]
fn test_lexer_identifier_simple() {
    let kinds = token_kinds("x").unwrap();
    assert!(matches!(&kinds[0], TokenKind::Ident(s) if s == "x"));
}

#[test]
fn test_lexer_identifier_underscore() {
    let kinds = token_kinds("_").unwrap();
    assert!(matches!(&kinds[0], TokenKind::Ident(s) if s == "_"));
}

#[test]
fn test_lexer_identifier_long() {
    let kinds = token_kinds("my_variable_name").unwrap();
    assert!(matches!(&kinds[0], TokenKind::Ident(s) if s == "my_variable_name"));
}

#[test]
fn test_lexer_identifier_with_numbers() {
    let kinds = token_kinds("var123").unwrap();
    assert!(matches!(&kinds[0], TokenKind::Ident(s) if s == "var123"));
}

// --- Indentation Tests ---

#[test]
fn test_lexer_indent_single() {
    let src = "def foo():\n    pass\n";
    let kinds = token_kinds(src).unwrap();
    // Def, Ident("foo"), LParen, RParen, Colon, Indent, Pass, Dedent, Eof
    assert!(kinds.iter().any(|k| matches!(k, TokenKind::Indent)));
    assert!(kinds.iter().any(|k| matches!(k, TokenKind::Dedent)));
}

#[test]
fn test_lexer_indent_multiple() {
    let src = "def foo():\n    if True:\n        pass\n";
    let kinds = token_kinds(src).unwrap();
    let indent_count = kinds.iter().filter(|k| matches!(k, TokenKind::Indent)).count();
    let dedent_count = kinds.iter().filter(|k| matches!(k, TokenKind::Dedent)).count();
    assert_eq!(indent_count, 2);
    assert_eq!(dedent_count, 2);
}

// --- Comment Tests ---

#[test]
fn test_lexer_comment_single_line() {
    let src = "x = 1 # this is a comment\n";
    let kinds = token_kinds(src).unwrap();
    // Should tokenize without the comment
    assert!(matches!(kinds[0], TokenKind::Ident(_)));
    assert!(matches!(kinds[1], TokenKind::Eq));
    assert!(matches!(kinds[2], TokenKind::Int(1)));
}

#[test]
fn test_lexer_comment_only() {
    let src = "# just a comment\n";
    let kinds = token_kinds(src).unwrap();
    // Should only have EOF
    assert!(matches!(kinds[0], TokenKind::Eof));
}

// --- Hex/Binary/Octal Tests ---

#[test]
fn test_lexer_hex_literal() {
    let kinds = token_kinds("0x1A").unwrap();
    assert!(matches!(kinds[0], TokenKind::Int(26)));
}

#[test]
fn test_lexer_binary_literal() {
    let kinds = token_kinds("0b1010").unwrap();
    assert!(matches!(kinds[0], TokenKind::Int(10)));
}

#[test]
fn test_lexer_octal_literal() {
    let kinds = token_kinds("0o755").unwrap();
    assert!(matches!(kinds[0], TokenKind::Int(493)));
}

// --- Raw String Tests ---

#[test]
fn test_lexer_raw_string() {
    let kinds = token_kinds(r#"r"hello\nworld""#).unwrap();
    // Raw string should contain literal \n, not newline
    assert!(matches!(&kinds[0], TokenKind::Str(s) if s == "hello\\nworld"));
}

// --- Compound Expression Tests ---

#[test]
fn test_lexer_multiple_tokens() {
    let src = "x = 42 + 3.14";
    let kinds = token_kinds(src).unwrap();
    // Ident, Eq, Int, Plus, Float, Eof (6 tokens)
    assert_eq!(kinds.len(), 6);
    assert!(matches!(&kinds[0], TokenKind::Ident(_)));
    assert_eq!(kinds[1], TokenKind::Eq);
    assert!(matches!(&kinds[2], TokenKind::Int(42)));
    assert_eq!(kinds[3], TokenKind::Plus);
    assert!(matches!(&kinds[4], TokenKind::Float(_)));
}

#[test]
fn test_lexer_function_call() {
    let src = "print(x)";
    let kinds = token_kinds(src).unwrap();
    assert!(matches!(&kinds[0], TokenKind::Ident(s) if s == "print"));
    assert_eq!(kinds[1], TokenKind::LParen);
    assert!(matches!(&kinds[2], TokenKind::Ident(s) if s == "x"));
    assert_eq!(kinds[3], TokenKind::RParen);
}

#[test]
fn test_lexer_list_literal() {
    let src = "[1, 2, 3]";
    let kinds = token_kinds(src).unwrap();
    assert_eq!(kinds[0], TokenKind::LBracket);
    assert!(matches!(kinds[1], TokenKind::Int(1)));
    assert_eq!(kinds[2], TokenKind::Comma);
    assert!(matches!(kinds[3], TokenKind::Int(2)));
    assert_eq!(kinds[4], TokenKind::Comma);
    assert!(matches!(kinds[5], TokenKind::Int(3)));
    assert_eq!(kinds[6], TokenKind::RBracket);
}

#[test]
fn test_lexer_dict_literal() {
    let src = r#"{"a": 1, "b": 2}"#;
    let kinds = token_kinds(src).unwrap();
    assert_eq!(kinds[0], TokenKind::LBrace);
    assert!(matches!(kinds[1], TokenKind::Str(_)));
    assert_eq!(kinds[2], TokenKind::Colon);
    assert!(matches!(kinds[3], TokenKind::Int(1)));
    assert_eq!(kinds[4], TokenKind::Comma);
}

#[test]
fn test_lexer_lambda() {
    let src = "lambda x: x + 1";
    let kinds = token_kinds(src).unwrap();
    assert_eq!(kinds[0], TokenKind::Lambda);
    assert!(matches!(kinds[1], TokenKind::Ident(_)));
    assert_eq!(kinds[2], TokenKind::Colon);
}

#[test]
fn test_lexer_ternary() {
    let src = "x if cond else y";
    let kinds = token_kinds(src).unwrap();
    assert!(matches!(kinds[0], TokenKind::Ident(_)));
    assert_eq!(kinds[1], TokenKind::If);
    assert!(matches!(kinds[2], TokenKind::Ident(_)));
    assert_eq!(kinds[3], TokenKind::Else);
    assert!(matches!(kinds[4], TokenKind::Ident(_)));
}

// --- Error Tests ---

#[test]
fn test_lexer_unterminated_string() {
    let src = r#""unterminated"#;
    let result = tokenize(src);
    assert!(result.is_err());
}

#[test]
fn test_lexer_invalid_hex() {
    let src = r#""\xGG""#;
    let result = tokenize(src);
    assert!(result.is_err());
}

#[test]
fn test_lexer_unrecognized_char() {
    let src = "$";
    let kinds = token_kinds(src).unwrap();
    assert!(matches!(kinds[0], TokenKind::Error(_)));
}
