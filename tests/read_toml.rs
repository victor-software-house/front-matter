//! `read` gives the top-level values of a TOML block, each with its spans,
//! style and text.

use front_matter::{Fault, Style, Value, find, read};

/// The values of a source that must read.
#[track_caller]
fn values(source: &str) -> Vec<Value> {
    let block = find(source).unwrap_or_else(|| panic!("no block in {source:?}"));
    read(source, &block).unwrap_or_else(|fault| panic!("{source:?} does not read: {fault}"))
}

/// The fault of a source that must not read.
#[track_caller]
fn fault(source: &str) -> Fault {
    let block = find(source).unwrap_or_else(|| panic!("no block in {source:?}"));
    read(source, &block).expect_err("the block should not read")
}

#[test]
fn each_string_form_has_its_style() {
    let source = "+++\nbasic = \"a\\tb é\"\nliteral = 'a\\tb'\nmultiline_basic = \"\"\"\nOne.\n\"\"\"\nmultiline_literal = '''\nOne.\\n\n'''\n+++\n";
    let values = values(source);
    let found: Vec<(Style, &str, &str)> = values
        .iter()
        .map(|value| {
            (
                value.style.clone(),
                &source[value.span.clone()],
                value.text.as_deref().unwrap_or(""),
            )
        })
        .collect();
    assert_eq!(
        found,
        [
            (Style::Basic, "\"a\\tb é\"", "a\tb é"),
            (Style::LiteralString, "'a\\tb'", "a\\tb"),
            (Style::MultilineBasic, "\"\"\"\nOne.\n\"\"\"", "One.\n"),
            (Style::MultilineLiteral, "'''\nOne.\\n\n'''", "One.\\n\n"),
        ]
    );
}

#[test]
fn a_multiline_string() {
    let source = "+++\nsummary = \"\"\"\nOne.\n\"\"\"\n+++\n";
    let values = values(source);
    assert_eq!(values[0].style, Style::MultilineBasic);
    assert_eq!(values[0].text.as_deref(), Some("One.\n"));
}

#[test]
fn multiline_strings_with_windows_line_endings() {
    let source = "+++\r\na = \"\"\"\r\nOne.\r\nTwo.\"\"\"\r\nb = '''\r\nThree.\r\nFour.'''\r\nc = \"\"\"x\\r\\ny\"\"\"\r\n+++\r\n";
    let values = values(source);
    assert_eq!(values[0].text.as_deref(), Some("One.\nTwo."));
    assert_eq!(
        &source[values[0].span.clone()],
        "\"\"\"\r\nOne.\r\nTwo.\"\"\""
    );
    assert_eq!(values[1].text.as_deref(), Some("Three.\nFour."));
    // An escaped carriage return and line feed stay as the file escapes them.
    assert_eq!(values[2].text.as_deref(), Some("x\r\ny"));
}

#[test]
fn a_string_with_a_line_ending_backslash() {
    let source = "+++\r\na = \"\"\"One \\\r\n    Two\"\"\"\r\n+++\r\n";
    assert_eq!(values(source)[0].text.as_deref(), Some("One Two"));
}

#[test]
fn values_that_are_no_string() {
    let source = "+++\nn = 1\nb = true\nd = 1979-05-27\nl = [1, 2]\nt = { a = 1 }\n+++\n";
    let values = values(source);
    assert!(
        values
            .iter()
            .all(|value| value.style == Style::Other && value.text.is_none())
    );
    assert_eq!(&source[values[2].span.clone()], "1979-05-27");
    assert_eq!(&source[values[3].span.clone()], "[1, 2]");
    assert_eq!(&source[values[4].span.clone()], "{ a = 1 }");
}

#[test]
fn keys_come_in_source_order_with_their_spans() {
    let source = "+++\nb = 'x'\n\"é\" = 'y'\na = 'z'\n+++\n";
    let values = values(source);
    let keys: Vec<(&str, &str)> = values
        .iter()
        .map(|value| (value.key.as_str(), &source[value.key_span.clone()]))
        .collect();
    assert_eq!(keys, [("b", "b"), ("é", "\"é\""), ("a", "a")]);
}

#[test]
fn spans_follow_a_byte_order_mark() {
    let source = "\u{feff}+++\ntitle = \"é\"\n+++\n";
    let values = values(source);
    assert_eq!(&source[values[0].key_span.clone()], "title");
    assert_eq!(&source[values[0].span.clone()], "\"é\"");
}

#[test]
fn a_block_named_toml() {
    let source = "---toml\ntitle = 'Install'\n---\n";
    assert_eq!(values(source)[0].text.as_deref(), Some("Install"));
}

#[test]
fn an_empty_block_has_no_value() {
    assert_eq!(values("+++\n+++\n"), []);
}

#[test]
fn a_syntax_error() {
    let source = "+++\ntitle = 'Install'\nbad = \n+++\nBody\n";
    let block = find(source).expect("a block");
    assert_eq!(&source[block.span.end..], "Body\n");
    assert_eq!(read(source, &block).expect_err("a fault").line, 3);
}

#[test]
fn a_byte_order_mark_changes_no_line() {
    assert_eq!(fault("\u{feff}+++\ntitle = 'é'\nbad = \n+++\n").line, 3);
}
