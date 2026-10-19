//! The specification is a fixture: every ```` ```kali ```` block in it must
//! parse. A block showing syntax the parser does not accept yet is tagged ````
//! ```kali-planned ```` instead, so the spec cannot claim syntax the parser
//! rejects without this test failing.

const SPEC: &str = include_str!("../../../../spec/SPECIFICATION.typ");

fn blocks(tag: &str) -> Vec<(usize, String)> {
    let mut blocks = Vec::new();
    let mut lines = SPEC.lines().enumerate().peekable();
    while let Some((number, line)) = lines.next() {
        if line.trim() != format!("```{tag}") {
            continue;
        }
        let body = lines
            .by_ref()
            .map(|(_, line)| line)
            .take_while(|line| line.trim() != "```")
            .collect::<Vec<_>>()
            .join("\n");
        blocks.push((number + 1, body));
    }
    blocks
}

#[test]
fn examples_parse() {
    let blocks = blocks("kali");
    assert!(blocks.len() > 10, "found only {} blocks", blocks.len());
    let failures = blocks
        .iter()
        .filter_map(|(line, src)| {
            let (_, errors) =
                kali_parse::parse_str(src, kali_span::Context { src: kali_span::SourceId(0) });
            (!errors.is_empty()).then(|| format!("block at line {line}: {errors:?}\n{src}"))
        })
        .collect::<Vec<_>>();
    assert!(failures.is_empty(), "\n{}", failures.join("\n\n"));
}

#[test]
fn planned_examples_do_not_parse_yet() {
    // When one of these starts parsing, retag it `kali` in the spec.
    let stale = blocks("kali-planned")
        .into_iter()
        .filter(|(_, src)| {
            kali_parse::parse_str(src, kali_span::Context { src: kali_span::SourceId(0) })
                .1
                .is_empty()
        })
        .map(|(line, src)| format!("block at line {line} now parses:\n{src}"))
        .collect::<Vec<_>>();
    assert!(stale.is_empty(), "\n{}", stale.join("\n\n"));
}
