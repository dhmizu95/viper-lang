//! Tests for lexer::indent_stack module

use viper_lang::lexer::{IndentChange, IndentStack};

#[test]
fn test_new_stack() {
    let stack = IndentStack::new();
    assert_eq!(stack.current(), 0);
    assert_eq!(stack.depth(), 0);
}

#[test]
fn test_same_indent() {
    let mut stack = IndentStack::new();
    assert_eq!(stack.process_indent(0), IndentChange::None);
    assert_eq!(stack.process_indent(0), IndentChange::None);
}

#[test]
fn test_single_indent() {
    let mut stack = IndentStack::new();
    assert_eq!(stack.process_indent(4), IndentChange::Indent);
    assert_eq!(stack.current(), 4);
    assert_eq!(stack.depth(), 1);
}

#[test]
fn test_multiple_indent() {
    let mut stack = IndentStack::new();
    assert_eq!(stack.process_indent(4), IndentChange::Indent);
    assert_eq!(stack.process_indent(8), IndentChange::Indent);
    assert_eq!(stack.process_indent(12), IndentChange::Indent);
    assert_eq!(stack.current(), 12);
    assert_eq!(stack.depth(), 3);
}

#[test]
fn test_single_dedent() {
    let mut stack = IndentStack::new();
    stack.process_indent(4);
    stack.process_indent(8);
    // Single dedent returns DedentCount(1)
    assert_eq!(stack.process_indent(4), IndentChange::DedentCount(1));
    assert_eq!(stack.current(), 4);
    assert_eq!(stack.depth(), 1);
}

#[test]
fn test_multiple_dedent() {
    let mut stack = IndentStack::new();
    stack.process_indent(4);
    stack.process_indent(8);
    stack.process_indent(12);
    assert_eq!(stack.process_indent(4), IndentChange::DedentCount(2));
    assert_eq!(stack.current(), 4);
    assert_eq!(stack.depth(), 1);
}

#[test]
fn test_dedent_to_zero() {
    let mut stack = IndentStack::new();
    stack.process_indent(4);
    // Dedent to zero returns DedentCount(1)
    assert_eq!(stack.process_indent(0), IndentChange::DedentCount(1));
    assert_eq!(stack.current(), 0);
    assert_eq!(stack.depth(), 0);
}

#[test]
fn test_dedent_to_zero_multiple() {
    let mut stack = IndentStack::new();
    stack.process_indent(4);
    stack.process_indent(8);
    assert_eq!(stack.process_indent(0), IndentChange::DedentCount(2));
    assert_eq!(stack.current(), 0);
    assert_eq!(stack.depth(), 0);
}

#[test]
fn test_inconsistent_indent() {
    let mut stack = IndentStack::new();
    stack.process_indent(4);
    stack.process_indent(8);
    // Try to dedent to a level that doesn't exist (6)
    let result = stack.process_indent(6);
    assert!(matches!(result, IndentChange::Error(_)));
    if let IndentChange::Error(msg) = result {
        assert!(msg.contains("Inconsistent indentation"));
    }
}

#[test]
fn test_reset() {
    let mut stack = IndentStack::new();
    stack.process_indent(4);
    stack.process_indent(8);
    stack.reset();
    assert_eq!(stack.current(), 0);
    assert_eq!(stack.depth(), 0);
}

#[test]
fn test_indent_after_reset() {
    let mut stack = IndentStack::new();
    stack.process_indent(4);
    stack.reset();
    assert_eq!(stack.process_indent(2), IndentChange::Indent);
    assert_eq!(stack.current(), 2);
}

#[test]
fn test_default() {
    // Default creates empty levels, so we need to reset to initialize properly
    let mut stack: IndentStack = Default::default();
    stack.reset();
    assert_eq!(stack.current(), 0);
    assert_eq!(stack.depth(), 0);
}

#[test]
fn test_indent_dedent_indent() {
    let mut stack = IndentStack::new();
    // Indent
    assert_eq!(stack.process_indent(4), IndentChange::Indent);
    // Dedent returns DedentCount(1)
    assert_eq!(stack.process_indent(0), IndentChange::DedentCount(1));
    // Indent again
    assert_eq!(stack.process_indent(4), IndentChange::Indent);
    assert_eq!(stack.current(), 4);
}

#[test]
fn test_complex_sequence() {
    let mut stack = IndentStack::new();
    // Level 0
    assert_eq!(stack.process_indent(0), IndentChange::None);
    // Indent to level 4
    assert_eq!(stack.process_indent(4), IndentChange::Indent);
    // Stay at level 4
    assert_eq!(stack.process_indent(4), IndentChange::None);
    // Indent to level 8
    assert_eq!(stack.process_indent(8), IndentChange::Indent);
    // Stay at level 8
    assert_eq!(stack.process_indent(8), IndentChange::None);
    // Dedent to level 4 returns DedentCount(1)
    assert_eq!(stack.process_indent(4), IndentChange::DedentCount(1));
    // Dedent to level 0 returns DedentCount(1)
    assert_eq!(stack.process_indent(0), IndentChange::DedentCount(1));
    // Stay at level 0
    assert_eq!(stack.process_indent(0), IndentChange::None);
}

#[test]
fn test_dedent_count_variants() {
    let mut stack = IndentStack::new();
    // Add multiple levels
    stack.process_indent(2);
    stack.process_indent(4);
    stack.process_indent(6);
    stack.process_indent(8);
    assert_eq!(stack.depth(), 4);
    
    // Dedent by 3 levels
    assert_eq!(stack.process_indent(2), IndentChange::DedentCount(3));
    assert_eq!(stack.depth(), 1);
}

#[test]
fn test_current_returns_last() {
    let mut stack = IndentStack::new();
    assert_eq!(stack.current(), 0);
    stack.process_indent(5);
    assert_eq!(stack.current(), 5);
    stack.process_indent(10);
    assert_eq!(stack.current(), 10);
}
