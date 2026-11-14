//! `read` gives the top-level members of a JSON block, each with its spans,
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
fn a_string_with_an_escape() {
    let source = ";;;\n{\"title\": \"Tab\\there\"}\n;;;\n";
    let values = values(source);
    assert_eq!(values[0].key, "title");
    assert_eq!(&source[values[0].key_span.clone()], "\"title\"");
    assert_eq!(values[0].style, Style::String);
    assert_eq!(&source[values[0].span.clone()], "\"Tab\\there\"");
    assert_eq!(values[0].text.as_deref(), Some("Tab\there"));
}

#[test]
fn every_kind_of_value_has_its_span() {
    let source = ";;;\n{ \"s\" : \"é\" , \"n\": -1.5e3, \"t\": true, \"f\": false, \"z\": null, \"a\": [1, {\"x\": [2]}], \"o\": {} , \"é\":\n \"é\"}\n;;;\n";
    let values = values(source);
    let found: Vec<(&str, &str, Style)> = values
        .iter()
        .map(|value| {
            (
                &source[value.key_span.clone()],
                &source[value.span.clone()],
                value.style.clone(),
            )
        })
        .collect();
    assert_eq!(
        found,
        [
            ("\"s\"", "\"é\"", Style::String),
            ("\"n\"", "-1.5e3", Style::Other),
            ("\"t\"", "true", Style::Other),
            ("\"f\"", "false", Style::Other),
            ("\"z\"", "null", Style::Other),
            ("\"a\"", "[1, {\"x\": [2]}]", Style::Other),
            ("\"o\"", "{}", Style::Other),
            ("\"é\"", "\"é\"", Style::String),
        ]
    );
    assert!(
        values
            .iter()
            .filter(|value| value.style == Style::Other)
            .all(|value| value.text.is_none())
    );
}

#[test]
fn a_bare_object() {
    let source = "{\n  \"title\": \"Install\"\n}\nBody\n";
    let values = values(source);
    assert_eq!(&source[values[0].span.clone()], "\"Install\"");
    assert_eq!(values[0].text.as_deref(), Some("Install"));
}

#[test]
fn a_bare_object_after_a_byte_order_mark() {
    let source = "\u{feff}{\"title\": \"Install\"}\nBody\n";
    let values = values(source);
    assert_eq!(&source[values[0].key_span.clone()], "\"title\"");
    assert_eq!(&source[values[0].span.clone()], "\"Install\"");
}

#[test]
fn a_block_named_json() {
    let source = "---json\n{\"title\": \"Install\"}\n---\n";
    assert_eq!(values(source)[0].text.as_deref(), Some("Install"));
}

#[test]
fn windows_line_endings() {
    let source = ";;;\r\n{\r\n  \"title\": \"Install\"\r\n}\r\n;;;\r\n";
    let values = values(source);
    assert_eq!(&source[values[0].span.clone()], "\"Install\"");
}

#[test]
fn an_empty_object_has_no_value() {
    assert_eq!(values(";;;\n{}\n;;;\n"), []);
}

#[test]
fn data_after_a_fenced_object() {
    assert_eq!(fault(";;;\n{\"a\": \"b\"} 1\n;;;\n").line, 2);
    assert_eq!(fault(";;;\n{\"a\": \"b\"}\n{}\n;;;\n").line, 3);
}

#[test]
fn whitespace_after_a_fenced_object() {
    assert_eq!(values(";;;\n{\"a\": \"b\"} \t\n\n;;;\n").len(), 1);
}

#[test]
fn yaml_flow_syntax_is_a_fault() {
    assert_eq!(fault(";;;\n{title: Install}\n;;;\n").line, 2);
}

#[test]
fn a_top_level_that_is_no_object() {
    assert_eq!(fault(";;;\n[1]\n;;;\n").line, 2);
    assert_eq!(fault(";;;\n\"x\"\n;;;\n").line, 2);
    assert_eq!(fault(";;;\n;;;\n").line, 2);
}

#[test]
fn a_syntax_error_names_its_line() {
    let source = ";;;\n{\n  \"a\": 1,\n  \"b\": ,\n}\n;;;\nBody\n";
    let block = find(source).expect("a block");
    assert_eq!(&source[block.span.end..], "Body\n");
    assert_eq!(read(source, &block).expect_err("a fault").line, 4);
}

#[test]
fn a_byte_order_mark_changes_no_line() {
    assert_eq!(
        fault("\u{feff};;;\n{\n\"é\": 1,\n\"b\": ,\n}\n;;;\n").line,
        4
    );
}
