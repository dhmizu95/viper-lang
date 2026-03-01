//! Tests for utils::span module

use viper_lang::utils::Span;

#[test]
fn test_span_new() {
    let span = Span::new(10, 20, 5, 15);
    assert_eq!(span.start, 10);
    assert_eq!(span.end, 20);
    assert_eq!(span.line, 5);
    assert_eq!(span.column, 15);
}

#[test]
fn test_span_empty() {
    let span = Span::empty(3, 10);
    assert_eq!(span.start, 0);
    assert_eq!(span.end, 0);
    assert_eq!(span.line, 3);
    assert_eq!(span.column, 10);
}

#[test]
fn test_span_merge() {
    let span1 = Span::new(10, 30, 5, 1);
    let span2 = Span::new(20, 40, 6, 5);
    let merged = span1.merge(span2);
    assert_eq!(merged.start, 10);
    assert_eq!(merged.end, 40);
    assert_eq!(merged.line, 5);
    assert_eq!(merged.column, 1);
}

#[test]
fn test_span_merge_reverse() {
    let span1 = Span::new(50, 60, 10, 5);
    let span2 = Span::new(20, 40, 5, 1);
    let merged = span1.merge(span2);
    assert_eq!(merged.start, 20);
    assert_eq!(merged.end, 60);
    assert_eq!(merged.line, 5);
    assert_eq!(merged.column, 1);
}

#[test]
fn test_span_display() {
    let span = Span::new(10, 25, 3, 5);
    assert_eq!(format!("{}", span), "3:5:15");
}

#[test]
fn test_span_default() {
    let span: Span = Default::default();
    assert_eq!(span.start, 0);
    assert_eq!(span.end, 0);
    assert_eq!(span.line, 0);
    assert_eq!(span.column, 0);
}

#[test]
fn test_span_clone() {
    let span1 = Span::new(1, 2, 3, 4);
    let span2 = span1.clone();
    assert_eq!(span1, span2);
}

#[test]
fn test_span_copy() {
    let span1 = Span::new(1, 2, 3, 4);
    let span2 = span1;
    // After copy, span1 should still be valid
    assert_eq!(span1.start, 1);
    assert_eq!(span2.start, 1);
}

#[test]
fn test_span_equality() {
    let span1 = Span::new(1, 2, 3, 4);
    let span2 = Span::new(1, 2, 3, 4);
    let span3 = Span::new(1, 2, 3, 5);
    assert_eq!(span1, span2);
    assert_ne!(span1, span3);
}
