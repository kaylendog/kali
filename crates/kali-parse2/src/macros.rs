//! Provides utilities for defining and testing parsers using macros.

/// A macro to define a parser function with a specified name, return type, and subparsers.
///
/// # Parameters
/// - `$name`: The name of the parser function to be defined.
/// - `$T`: The return type of the parser function.
/// - `{ $( $subparser_name : $subparser_ty ),* }`: A list of subparser names and their respective types.
/// - `$body`: The body of the parser function, which defines its behavior.
///
/// # Example
/// ```no_run
/// define_parser! {
///     my_parser: MyType,
///     {
///         subparser1: SubType1,
///         subparser2: SubType2
///     },
///     {
///         // Parser logic here
///     }
/// }
/// ```
#[macro_export]
macro_rules! define_parser {
    ($name:ident: $T:ty, { $( $subparser_name:ident : $subparser_ty:ty ),* }, $body:block) => {
        pub fn $name<'src, F>(
            $(
                $subparser_name: impl chumsky::Parser<
                    'src,
                    chumsky::input::MappedSpan<kali_ast2::Span, &'src str, F>,
                    $subparser_ty,
                    chumsky::extra::Full<
                        chumsky::error::Rich<'src, char, kali_ast2::Span>,
                        chumsky::extra::SimpleState<$crate::State>,
                        ()
                    >,
                > + Clone,
            )*
        ) -> impl chumsky::Parser<
            'src,
            chumsky::input::MappedSpan<kali_ast2::Span, &'src str, F>,
            $T,
            chumsky::extra::Full<
                chumsky::error::Rich<'src, char, kali_ast2::Span>,
                chumsky::extra::SimpleState<$crate::State>,
                ()
            >,
        > + Clone
        where
            F: Fn(chumsky::span::SimpleSpan) -> kali_ast2::Span + 'src,
        {
            $body
        }
    };
}

/// A macro to assert that a parser produces the expected result when parsing an input.
///
/// # Parameters
/// - `$parser`: The parser to be tested.
/// - `$input`: The input to be parsed.
/// - `$expected`: The expected result of the parsing.
///
/// # Example
/// ```no_run
/// assert_parse!(my_parser, "input_string", expected_result);
/// ```
#[macro_export]
macro_rules! assert_parse_ok {
    ($parser:expr, $input:expr, $expected:expr) => {{
        let mut state = chumsky::extra::SimpleState($crate::State {
            current_id: 0,
            identifiers: lasso::Rodeo::new(),
        });
        let result = $parser.parse_with_state(
            chumsky::input::Input::map_span($input, chumsky::span::SimpleSpan::into),
            &mut state,
        );
        assert_eq!(result.unwrap(), $expected);
    }};
}

/// A macro to assert that a parser produces an error when parsing an input.
///
/// # Parameters
/// - `$parser`: The parser to be tested.
/// - `$input`: The input to be parsed.
///
/// # Example
/// ```no_run
/// assert_parse_err!(my_parser, "invalid_input");
/// ```
#[macro_export]
macro_rules! assert_parse_err {
    ($parser:expr, $input:expr) => {{
        let mut state = chumsky::extra::SimpleState($crate::State {
            current_id: 0,
            identifiers: lasso::Rodeo::new(),
        });
        let result = $parser.parse_with_state(
            chumsky::input::Input::map_span($input, chumsky::span::SimpleSpan::into),
            &mut state,
        );
        assert!(result.is_err(), "Expected an error, but got: {:?}", result);
    }};
}

/// A macro to create a `Span` for a given literal or a specific range.
///
/// # Parameters
/// - `$lit`: The literal string to calculate the span for.
/// - `$start`: (Optional) The start position of the span.
/// - `$end`: (Optional) The end position of the span.
/// - `$offset`: (Optional) The offset to apply to the span.
///
/// # Examples
/// ```no_run
/// let span = span_of!("example");
/// let span_with_range = span_of!("example", 1, 4);
/// let span_with_offset = span_of!("example", offset = 2);
/// ```
#[macro_export]
macro_rules! span_of {
    ($input:tt) => {
        kali_ast2::Span {
            start: 0,
            end: $input.len(),
            file_id: 0,
        }
    };
    ($input:tt, start = $start:expr, end = $end:expr) => {
        kali_ast2::Span {
            start: $start,
            end: $end,
            file_id: 0,
        }
    };
    ($input:tt, offset = $offset:expr) => {
        kali_ast2::Span {
            start: $offset,
            end: $offset + $input.len(),
            file_id: 0,
        }
    };
}
