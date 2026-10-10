use crate::common::{assert_fixed, assert_round_trip};

#[test]
fn comments() {
    assert_fixed("// leading\n/// doc\nlet a = 1 // after a\nlet b = 2\n// end\n");
}

#[test]
fn empty_comment() {
    assert_fixed("//\nlet a = 1 //\n");
}

#[test]
fn comments_on_every_item_kind() {
    assert_fixed("/// A shape.\ntype Shape = Circle(int)\n// io\nimport std::io\n");
}

#[test]
fn comment_only_module() {
    assert_fixed("// nothing here\n");
    assert_round_trip("\n// a\n\n\n// b\n", "// a\n\n// b\n");
}

#[test]
fn dangling_comments_follow_the_item_for_now() {
    // Comments inside an item have no slot yet, so the printer moves them after it.
    assert_round_trip(
        "let x = match a {\n\t// zero\n\t| 0 -> 1\n\t| _ -> 2\n}\n",
        "let x = match a {\n\t| 0 -> 1\n\t| _ -> 2\n}\n// zero\n",
    );
}
