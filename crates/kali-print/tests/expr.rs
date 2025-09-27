use std::io::Write;

use kali_ast::{Eraser, Rewriter};
use kali_parse::{kali_expr, parse_expr_str};
use kali_print::{Context, Print};

struct TestWriter {
    pub output: Vec<u8>,
}

impl Write for TestWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.output.extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn print_to_string<T: Print>(value: &T) -> String {
    let mut writer = TestWriter { output: Vec::new() };
    let mut ctx = Context::new(&mut writer);
    value.print(&mut ctx).unwrap();
    String::from_utf8(writer.output).unwrap()
}

#[test]
fn test_e2e_binary_expr() {
    let expr = Eraser::rewrite(&mut (), kali_expr! { x + 1 }).unwrap();
    let formatted = print_to_string(&expr);
    let parsed = Eraser::rewrite(&mut (), parse_expr_str(&formatted).unwrap()).unwrap();
    assert_eq!(expr, parsed);
}

#[test]
fn test_e2e_unary_expr() {
    let expr = Eraser::rewrite(&mut (), kali_expr! { -x }).unwrap();
    let formatted = print_to_string(&expr);
    let parsed = Eraser::rewrite(&mut (), parse_expr_str(&formatted).unwrap()).unwrap();
    assert_eq!(expr, parsed);
}

#[test]
fn test_e2e_literal_expr() {
    let expr = Eraser::rewrite(&mut (), kali_expr! { 42 }).unwrap();
    let formatted = print_to_string(&expr);
    let parsed = Eraser::rewrite(&mut (), parse_expr_str(&formatted).unwrap()).unwrap();
    assert_eq!(expr, parsed);
}

#[test]
fn test_e2e_paren_expr() {
    let expr = Eraser::rewrite(&mut (), kali_expr! { (x + 1) }).unwrap();
    let formatted = print_to_string(&expr);
    let parsed = Eraser::rewrite(&mut (), parse_expr_str(&formatted).unwrap()).unwrap();
    assert_eq!(expr, parsed);
}

#[test]
fn test_e2e_call_expr() {
    let expr = Eraser::rewrite(&mut (), kali_expr! { foo(x, 2) }).unwrap();
    let formatted = print_to_string(&expr);
    let parsed = Eraser::rewrite(&mut (), parse_expr_str(&formatted).unwrap()).unwrap();
    assert_eq!(expr, parsed);
}

#[test]
fn test_e2e_if_expr() {
    let expr = Eraser::rewrite(&mut (), kali_expr! { if x 1 else 2 }).unwrap();
    let formatted = print_to_string(&expr);
    let parsed = Eraser::rewrite(&mut (), parse_expr_str(&formatted).unwrap()).unwrap();
    assert_eq!(expr, parsed);
}
