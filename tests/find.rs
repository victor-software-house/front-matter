//! `find` gives the block of each form, and none for a source that opens no
//! form.

use front_matter::{Block, Format, find};

/// The block of a source that must have one.
#[track_caller]
fn found(source: &str) -> Block {
    find(source).unwrap_or_else(|| panic!("no block in {source:?}"))
}

#[test]
fn a_yaml_block() {
    let source = "---\ntitle: Install\n---\nBody\n";
    let block = found(source);
    assert_eq!(block.format, Format::Yaml);
    assert_eq!(&source[block.span.clone()], "---\ntitle: Install\n---\n");
    assert_eq!(&source[block.content], "title: Install\n");
    assert_eq!(&source[block.span.end..], "Body\n");
}

#[test]
fn a_yaml_block_closed_by_dots() {
    let source = "---\ntitle: Install\n...\nBody\n";
    let block = found(source);
    assert_eq!(block.format, Format::Yaml);
    assert_eq!(&source[block.span], "---\ntitle: Install\n...\n");
}

#[test]
fn a_yaml_block_named_yaml() {
    let source = "---yaml\ntitle: Install\n---\nBody\n";
    let block = found(source);
    assert_eq!(block.format, Format::Yaml);
    assert_eq!(&source[block.content], "title: Install\n");
}

#[test]
fn a_toml_block() {
    let source = "+++\ntitle = \"Install\"\n+++\nBody\n";
    let block = found(source);
    assert_eq!(block.format, Format::Toml);
    assert_eq!(&source[block.span], "+++\ntitle = \"Install\"\n+++\n");
}

#[test]
fn a_toml_block_named_toml() {
    let source = "---toml\ntitle = \"Install\"\n---\nBody\n";
    let block = found(source);
    assert_eq!(block.format, Format::Toml);
    assert_eq!(&source[block.content], "title = \"Install\"\n");
}

#[test]
fn a_json_block_with_fences() {
    let source = ";;;\n{\"title\": \"Install\"}\n;;;\nBody\n";
    let block = found(source);
    assert_eq!(block.format, Format::Json);
    assert_eq!(&source[block.content], "{\"title\": \"Install\"}\n");
    assert_eq!(&source[block.span.end..], "Body\n");
}

#[test]
fn a_json_block_named_json() {
    let source = "---json\n{\"title\": \"Install\"}\n---\nBody\n";
    let block = found(source);
    assert_eq!(block.format, Format::Json);
    assert_eq!(&source[block.content], "{\"title\": \"Install\"}\n");
}

#[test]
fn a_bare_json_object() {
    let source = "{\n  \"title\": \"Install\"\n}\nBody\n";
    let block = found(source);
    assert_eq!(block.format, Format::Json);
    assert_eq!(
        &source[block.span.clone()],
        "{\n  \"title\": \"Install\"\n}\n"
    );
    assert_eq!(block.content, block.span);
    assert_eq!(&source[block.span.end..], "Body\n");
}

#[test]
fn a_bare_json_object_with_blanks_after_it() {
    let source = "{\"title\": \"Install\"} \t\nBody\n";
    let block = found(source);
    assert_eq!(&source[block.span.end..], "Body\n");
}

#[test]
fn a_bare_json_object_that_other_data_follows_has_no_block() {
    assert_eq!(find("{\"title\": \"Install\"} 1\nBody\n"), None);
}

#[test]
fn a_bare_json_object_that_does_not_close_has_no_block() {
    assert_eq!(find("{\"title\": \"Install\"\nBody\n"), None);
}

#[test]
fn a_bare_json_object_that_json_refuses_has_no_block() {
    assert_eq!(find("{title: Install}\nBody\n"), None);
}

#[test]
fn blanks_after_a_fence() {
    let source = "--- \t\ntitle: Install\n---  \nBody\n";
    let block = found(source);
    assert_eq!(&source[block.content], "title: Install\n");
    assert_eq!(&source[block.span.end..], "Body\n");
}

#[test]
fn an_empty_block() {
    let source = "---\n---\nBody\n";
    let block = found(source);
    assert_eq!(block.content, 4..4);
    assert_eq!(&source[block.span.end..], "Body\n");
}

#[test]
fn windows_line_endings() {
    let source = "---\r\ntitle: Install\r\n---\r\nBody\r\n";
    let block = found(source);
    assert_eq!(block.content.start, "---\r\n".len());
    assert_eq!(&source[block.content], "title: Install\r\n");
    assert_eq!(block.span.end, "---\r\ntitle: Install\r\n---\r\n".len());
    assert_eq!(&source[block.span.end..], "Body\r\n");
}

#[test]
fn windows_line_endings_in_a_bare_object() {
    let source = "{\r\n  \"title\": \"Install\"\r\n}\r\nBody\r\n";
    let block = found(source);
    assert_eq!(&source[block.span.end..], "Body\r\n");
}

#[test]
fn a_closing_fence_at_the_end_of_the_file() {
    let source = "---\r\ntitle: Install\r\n---";
    let block = found(source);
    assert_eq!(block.span.end, source.len());
    assert_eq!(&source[block.content], "title: Install\r\n");

    let source = "---\ntitle: Install\n---";
    assert_eq!(found(source).span.end, source.len());
}

#[test]
fn a_bare_object_at_the_end_of_the_file() {
    let source = "{\"title\": \"Install\"}";
    assert_eq!(found(source).span.end, source.len());
}

#[test]
fn a_byte_order_mark() {
    let source = "\u{feff}---\ntitle: Install\n---\n";
    let block = found(source);
    assert_eq!(block.span.start, 3);
    assert_eq!(&source[block.content], "title: Install\n");
}

#[test]
fn a_byte_order_mark_before_a_bare_object() {
    let source = "\u{feff}{\"title\": \"Install\"}\nBody\n";
    let block = found(source);
    assert_eq!(block.format, Format::Json);
    assert_eq!(block.span.start, 3);
    assert_eq!(&source[block.span.end..], "Body\n");
}

#[test]
fn a_block_that_never_closes() {
    assert_eq!(find("---\ntitle: Install\nBody\n"), None);
    assert_eq!(find("---\n"), None);
    assert_eq!(find("---"), None);
}

#[test]
fn a_rule_that_is_no_fence() {
    assert_eq!(find("\n---\ntitle: Install\n---\n"), None);
}

#[test]
fn a_fence_that_opens_no_form() {
    assert_eq!(find("--\ntitle: Install\n--\n"), None);
    assert_eq!(find("---x\ntitle: Install\n---\n"), None);
    assert_eq!(find("Body\n"), None);
    assert_eq!(find(""), None);
}

#[test]
fn a_lone_carriage_return_ends_no_line() {
    assert_eq!(find("---\rtitle: Install\r---\r"), None);
    assert_eq!(find("---\ntitle: Install\r---\n"), None);
    let source = "---\ntitle: a\rb\n---\n";
    let block = found(source);
    assert_eq!(&source[block.content], "title: a\rb\n");
}

#[test]
fn a_closing_line_does_not_open_a_form() {
    // The first closing line ends the block, so a later fence belongs to the
    // body.
    let source = "---\ntitle: Install\n---\nBody\n---\nmore\n---\n";
    let block = found(source);
    assert_eq!(&source[block.span.end..], "Body\n---\nmore\n---\n");
}
