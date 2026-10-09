use kali_print::print_module;
pub use pretty_assertions::assert_eq;

pub fn print(src: &str) -> String {
    let (module, errors) = kali_parse::parse_str(src);
    assert!(errors.is_empty(), "{src:?} failed to parse: {errors:?}");
    let mut out = Vec::new();
    print_module(&module.unwrap(), src, &mut out).unwrap();
    String::from_utf8(out).unwrap()
}

/// Printing `src` gives `expected`, and printing `expected` gives it back unchanged.
pub fn assert_round_trip(src: &str, expected: &str) {
    assert_eq!(print(src), expected, "first print of {src:?}");
    assert_eq!(print(expected), expected, "reprint of {expected:?}");
}

/// `src` is already in canonical form.
pub fn assert_fixed(src: &str) {
    assert_round_trip(src, src);
}
