use kali_print::{Settings, format_module};
pub use pretty_assertions::assert_eq;

pub fn print(src: &str) -> String {
    print_with(src, Settings::default())
}

pub fn print_with(src: &str, settings: Settings) -> String {
    let (module, errors) = kali_parse::debug_parse_str(src);
    assert!(errors.is_empty(), "{src:?} failed to parse: {errors:?}");
    let mut out = Vec::new();
    format_module(&module.unwrap(), src, settings, &mut out).unwrap();
    String::from_utf8(out).unwrap()
}

/// Printing `src` gives `expected`, and printing `expected` gives it back
/// unchanged.
pub fn assert_round_trip(src: &str, expected: &str) {
    assert_eq!(expected, print(src), "first print of {src:?}");
    assert_eq!(expected, print(expected), "reprint of {expected:?}");
}

/// `src` is already in canonical form.
pub fn assert_fixed(src: &str) {
    assert_round_trip(src, src);
}
