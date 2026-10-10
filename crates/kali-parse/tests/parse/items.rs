use crate::common::{assert_eq, parses, rejects};

fn item_count(src: &str) -> usize {
    parses(src).items.len()
}

#[test]
fn definitions() {
    parses("let x = 1");
    parses("let (a, b) = (1, 2)");
    parses("let { x, y: z } = p");
    parses("let ((a, b), c) = t");
    rejects("let = 1");
    rejects("let x");
    rejects("let x = ");
    rejects("let 1 = x");
}

#[test]
fn type_aliases_and_declarations() {
    parses("type T = int");
    parses("type S = Circle(int) | Rectangle(int, int) | Point");
    parses("type S = | Circle(int) | Point");
    parses("type S = Foo");
    rejects("type T");
    rejects("type T = ");
    rejects("type t = Circle(int) |");
    // Structural unions and intersections are gone.
    rejects("type T = int | bool");
    rejects("type T = { x: int } & { y: int }");
}

#[test]
fn imports() {
    parses("import a");
    parses("import a as b");
    parses("import a :: *");
    parses("import a :: b :: c");
    parses("import a :: { b, c as d, e :: *, }");
    parses("import a :: { }");
    rejects("import");
    rejects("import a ::");
    rejects("import a as");
}

#[test]
fn export_is_not_parsed() {
    rejects("export let x = 1");
    rejects("export type T = int");
}

#[test]
fn line_ends_terminate_items() {
    assert_eq!(item_count("let a = 1\nlet b = 2"), 2);
    assert_eq!(item_count("let a = 1\n\n\nlet b = 2\n"), 2);
    assert_eq!(item_count("let a = 1; let b = 2;"), 2);
    assert_eq!(item_count("let a = 1 // note\nlet b = 2"), 2);
    assert_eq!(item_count("import std :: *\nlet a = 1"), 2);
    assert_eq!(item_count("type S = A | B\nlet a = 1"), 2);
    rejects("let a = 1 let b = 2");
}

#[test]
fn line_ends_continue_open_constructs() {
    assert_eq!(item_count("let x = fib (n - 1) +\n\tfib (n - 2)"), 1);
    assert_eq!(item_count("let x = [1,\n\t2]"), 1);
    assert_eq!(item_count("let x = [1, 2\n]"), 1);
    assert_eq!(item_count("let x = (\n\t1, 2\n)"), 1);
    assert_eq!(item_count("let x = f (\n\ta)"), 1);
    assert_eq!(item_count("let x = match a {\n\t| 0 -> 1\n\t| _ -> 2\n}"), 1);
    assert_eq!(item_count("let x = if a {\n\tb\n} else {\n\tc\n}"), 1);
    assert_eq!(item_count("type S =\n\t| A\n\t| B"), 1);
    assert_eq!(item_count("let f = a ->\n\ta + 1"), 1);
    assert_eq!(item_count("let x = a &&\n\tb"), 1);
}

#[test]
fn operator_must_end_the_line_to_continue() {
    rejects("let x = fib (n - 1)\n\t+ fib (n - 2)");
    rejects("let x = a\n\t&& b");
}

#[test]
fn empty_and_comment_only_modules() {
    assert_eq!(item_count(""), 0);
    assert_eq!(item_count("\n\n"), 0);
    let module = parses("// a\n/// b\n");
    assert_eq!(module.items.len(), 0);
    assert_eq!(module.comments.len(), 2);
}
