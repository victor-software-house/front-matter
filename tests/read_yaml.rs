//! `read` gives the top-level values of a YAML block, each with its spans,
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
fn a_quoted_string() {
    let source = "---\ntitle: \"Install é\"\n---\n";
    let values = values(source);
    assert_eq!(values.len(), 1);
    let title = &values[0];
    assert_eq!(title.key, "title");
    assert_eq!(&source[title.key_span.clone()], "title");
    assert_eq!(title.style, Style::DoubleQuoted);
    assert_eq!(&source[title.span.clone()], "\"Install é\"");
    assert_eq!(title.text.as_deref(), Some("Install é"));
}

#[test]
fn a_double_quoted_escape() {
    let source = "---\ntitle: \"a\\tb\"\n---\n";
    let values = values(source);
    assert_eq!(&source[values[0].span.clone()], "\"a\\tb\"");
    assert_eq!(values[0].text.as_deref(), Some("a\tb"));
}

#[test]
fn a_single_quoted_string_and_a_quoted_key() {
    let source = "---\n'k é': 'it''s'\n---\n";
    let values = values(source);
    assert_eq!(values[0].key, "k é");
    assert_eq!(&source[values[0].key_span.clone()], "'k é'");
    assert_eq!(values[0].style, Style::SingleQuoted);
    assert_eq!(&source[values[0].span.clone()], "'it''s'");
    assert_eq!(values[0].text.as_deref(), Some("it's"));
}

#[test]
fn a_plain_string() {
    let source = "---\nplain: Install guide\n---\n";
    let values = values(source);
    assert_eq!(values[0].style, Style::Plain);
    assert_eq!(&source[values[0].span.clone()], "Install guide");
    assert_eq!(values[0].text.as_deref(), Some("Install guide"));
}

#[test]
fn a_block_scalar() {
    let source = "---\ndescription: |\n  One.\n  Two.\nnext: x\n---\n";
    let values = values(source);
    assert_eq!(values[0].style, Style::Literal);
    assert_eq!(&source[values[0].span.clone()], "|\n  One.\n  Two.");
    assert_eq!(values[0].text.as_deref(), Some("One.\nTwo.\n"));
    assert_eq!(values[1].key, "next");
    assert_eq!(&source[values[1].span.clone()], "x");
}

#[test]
fn a_folded_block_scalar_with_a_header_modifier_and_a_comment() {
    let source = "---\ndescription: !!str >- # note\n  One.\n  Two.\n\n\nnext: x\n---\n";
    let values = values(source);
    assert_eq!(values[0].style, Style::Folded);
    assert_eq!(&source[values[0].span.clone()], ">- # note\n  One.\n  Two.");
    assert_eq!(values[0].text.as_deref(), Some("One. Two."));
}

#[test]
fn a_block_scalar_with_windows_line_endings() {
    let source = "---\r\ndescription: |\r\n  a\r\n  b\r\n---\r\n";
    let values = values(source);
    assert_eq!(values[0].text.as_deref(), Some("a\nb\n"));
    assert_eq!(&source[values[0].span.clone()], "|\r\n  a\r\n  b");
}

#[test]
fn windows_line_endings_leave_no_carriage_return_in_a_span() {
    let source = "---\r\ntitle: Install\r\nname: 'x'\r\n---\r\nBody\r\n";
    let values = values(source);
    assert_eq!(values[0].text.as_deref(), Some("Install"));
    assert_eq!(&source[values[0].span.clone()], "Install");
    assert_eq!(&source[values[1].span.clone()], "'x'");
}

#[test]
fn values_that_are_no_string() {
    let source = "---\ndraft: true\ntags: [a, b]\n---\n";
    let values = values(source);
    for value in &values {
        assert_eq!(value.style, Style::Other);
        assert_eq!(value.text, None);
    }
    assert_eq!(&source[values[0].span.clone()], "true");
    assert_eq!(&source[values[1].span.clone()], "[a, b]");
}

#[test]
fn the_core_schema_reads_these_plain_scalars_as_no_string() {
    for plain in [
        "", "~", "null", "Null", "NULL", "true", "True", "TRUE", "false", "False", "FALSE", "0",
        "12", "-12", "+12", "0o17", "0x1F", "1.5", "-1.5e3", "1.", ".5", "1e3", "+.inf", "-.Inf",
        ".INF", ".nan", ".NaN",
    ] {
        let source = format!("---\nkey: {plain}\n---\n");
        let values = values(&source);
        assert_eq!(values[0].style, Style::Other, "{plain:?}");
        assert_eq!(values[0].text, None, "{plain:?}");
    }
}

#[test]
fn the_core_schema_reads_these_plain_scalars_as_a_string() {
    for plain in [
        "yes",
        "No",
        "on",
        "1_000",
        "0x",
        "0o8",
        "1.2.3",
        ".",
        "e3",
        "1e",
        "+",
        "0b11",
        "2026-11-14",
        "nan",
        ".infinity",
        "Null1",
    ] {
        let source = format!("---\nkey: {plain}\n---\n");
        let values = values(&source);
        assert_eq!(values[0].style, Style::Plain, "{plain:?}");
        assert_eq!(values[0].text.as_deref(), Some(plain), "{plain:?}");
    }
}

#[test]
fn a_quoted_scalar_is_a_string_whatever_it_holds() {
    let source = "---\na: \"true\"\nb: '12'\nc: \"\"\n---\n";
    let values = values(source);
    assert_eq!(values[0].text.as_deref(), Some("true"));
    assert_eq!(values[1].text.as_deref(), Some("12"));
    assert_eq!(values[2].text.as_deref(), Some(""));
}

#[test]
fn an_explicit_string_tag_makes_a_plain_scalar_a_string() {
    let source = "---\na: !!str true\nb: !<tag:yaml.org,2002:str> 12\nc: !!str\n---\n";
    let values = values(source);
    for (value, text) in values.iter().zip(["true", "12", ""]) {
        assert_eq!(value.style, Style::Plain);
        assert_eq!(value.text.as_deref(), Some(text));
    }
    assert_eq!(&source[values[0].span.clone()], "true");
}

#[test]
fn every_other_tag_is_read_as_if_absent() {
    let source = "---\na: !!int 12\nb: !custom foo\nc: !custom 12\nd: !!null x\n---\n";
    let values = values(source);
    assert_eq!(values[0].style, Style::Other);
    assert_eq!(values[1].style, Style::Plain);
    assert_eq!(values[1].text.as_deref(), Some("foo"));
    assert_eq!(values[2].style, Style::Other);
    assert_eq!(values[3].style, Style::Plain);
    assert_eq!(values[3].text.as_deref(), Some("x"));
}

#[test]
fn a_value_that_is_missing() {
    let source = "---\nempty:\nnext: x\n---\n";
    let values = values(source);
    assert_eq!(values[0].style, Style::Other);
    assert_eq!(values[0].text, None);
    assert_eq!(values[1].text.as_deref(), Some("x"));
}

#[test]
fn a_list_and_a_mapping_span_their_whole_node() {
    let source = "---\ntags: [a, b]\nm: {a: 1}\nl:\n  - x\n  - y\nb:\n  k: v\n  j: w\nz: 1\n---\n";
    let values = values(source);
    let spans: Vec<&str> = values
        .iter()
        .map(|value| &source[value.span.clone()])
        .collect();
    assert_eq!(
        spans,
        ["[a, b]", "{a: 1}", "- x\n  - y", "k: v\n  j: w", "1"]
    );
    assert!(values.iter().all(|value| value.style == Style::Other));
}

#[test]
fn nested_collections_are_one_value() {
    let source = "---\na:\n  - [1, {b: [2, 3]}]\n  - c: d\nz: end\n---\n";
    let values = values(source);
    assert_eq!(values.len(), 2);
    assert_eq!(
        &source[values[0].span.clone()],
        "- [1, {b: [2, 3]}]\n  - c: d"
    );
    assert_eq!(values[1].text.as_deref(), Some("end"));
}

#[test]
fn an_alias_is_no_string() {
    let source = "---\nbase: &anchor Install\ncopy: *anchor\n---\n";
    let values = values(source);
    assert_eq!(values[0].text.as_deref(), Some("Install"));
    assert_eq!(values[1].style, Style::Other);
    assert_eq!(&source[values[1].span.clone()], "*anchor");
}

#[test]
fn keys_come_in_source_order() {
    let source = "---\nb: 1\na: 2\nc: 3\n---\n";
    let keys: Vec<String> = values(source).into_iter().map(|value| value.key).collect();
    assert_eq!(keys, ["b", "a", "c"]);
}

#[test]
fn an_empty_block_has_no_value() {
    assert_eq!(values("---\n---\n"), []);
}

#[test]
fn a_block_closed_by_dots() {
    let source = "---\ntitle: Install\n...\nBody\n";
    assert_eq!(values(source)[0].text.as_deref(), Some("Install"));
}

#[test]
fn a_syntax_error() {
    let source = "---\ntitle: Install\n  bad: [\n---\nBody\n";
    let block = find(source).expect("a block");
    assert_eq!(&source[block.span.end..], "Body\n");
    assert_eq!(read(source, &block).expect_err("a fault").line, 3);
}

#[test]
fn a_top_level_list() {
    assert_eq!(fault("---\n- a\n- b\n---\n").line, 2);
}

#[test]
fn a_top_level_scalar() {
    assert_eq!(fault("---\nInstall\n---\n").line, 2);
}

#[test]
fn a_key_that_is_a_list() {
    assert_eq!(fault("---\n? [a, b]\n: c\n---\n").line, 2);
}

#[test]
fn a_byte_order_mark_changes_no_line() {
    let source = "\u{feff}---\ntitle: Install\n  bad: [\n---\n";
    assert_eq!(fault(source).line, 3);
    let source = "\u{feff}---\ntitle: é\nbad: [\n---\n";
    assert_eq!(fault(source).line, 3);
}

#[test]
fn spans_follow_a_byte_order_mark() {
    let source = "\u{feff}---\ntitle: é\n---\n";
    let values = values(source);
    assert_eq!(&source[values[0].key_span.clone()], "title");
    assert_eq!(&source[values[0].span.clone()], "é");
}

#[test]
fn a_fault_displays_its_line() {
    let fault = fault("---\n- a\n---\n");
    assert!(fault.to_string().starts_with("line 2: "));
}
