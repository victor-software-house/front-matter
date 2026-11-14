//! Finding the block at the top of a source.

use std::ops::Range;

use struson::reader::{JsonReader, JsonStreamReader};

/// The format of a front matter block, which picks its parser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Format {
    /// YAML, read by `granit-parser`.
    Yaml,
    /// TOML, read by `toml`.
    Toml,
    /// JSON, read by `struson`.
    Json,
}

/// A front matter block: its format and where it sits in the source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    /// The format of the block.
    pub format: Format,
    /// The bytes of the block with its fences and their line endings.
    /// It starts after a byte order mark, and a caller reads the body of the
    /// file from its end.
    pub span: Range<usize>,
    /// The bytes between the fences.
    /// For a bare JSON object, the object is its own fence, so this equals
    /// `span`.
    pub content: Range<usize>,
}

/// A byte order mark, which a file saved by some editors starts with.
const BYTE_ORDER_MARK: char = '\u{feff}';

/// One line of a source, without its line ending.
struct Line<'source> {
    /// The text of the line, with no `\n` or `\r\n`.
    text: &'source str,
    /// The byte where the line starts.
    start: usize,
    /// The byte where the next line starts.
    next: usize,
}

/// Reads the line that starts at `start`.
/// Only `\n` and `\r\n` end a line, so a lone `\r` stays in its text.
fn line_at(source: &str, start: usize) -> Line<'_> {
    let rest = &source[start..];
    match rest.find('\n') {
        Some(index) => {
            let text = &rest[..index];
            Line {
                text: text.strip_suffix('\r').unwrap_or(text),
                start,
                next: start + index + 1,
            }
        }
        None => Line {
            text: rest,
            start,
            next: source.len(),
        },
    }
}

/// The format that an opening line names, and the lines that close it.
fn opener(text: &str) -> Option<(Format, &'static [&'static str])> {
    match text.trim_end_matches([' ', '\t']) {
        "---" | "---yaml" => Some((Format::Yaml, &["---", "..."])),
        "+++" => Some((Format::Toml, &["+++"])),
        "---toml" => Some((Format::Toml, &["---"])),
        ";;;" => Some((Format::Json, &[";;;"])),
        "---json" => Some((Format::Json, &["---"])),
        _ => None,
    }
}

/// Finds the front matter block at the start of `source`.
///
/// The first line names the block, after an optional byte order mark:
/// `---` or `---yaml` for YAML, `+++` or `---toml` for TOML, `;;;` or `---json`
/// for JSON. A line that starts with `{` opens a JSON object that is its own
/// block. A block whose closing line never comes is no block, and neither is a
/// bare object that `struson` cannot read to its end.
///
/// ```
/// let source = "---\ntitle: Install\n---\nBody\n";
/// let block = front_matter::find(source).expect("a block");
/// assert_eq!(&source[block.content], "title: Install\n");
/// assert_eq!(&source[block.span.end..], "Body\n");
/// ```
#[must_use]
pub fn find(source: &str) -> Option<Block> {
    let start = if source.starts_with(BYTE_ORDER_MARK) {
        BYTE_ORDER_MARK.len_utf8()
    } else {
        0
    };
    let first = line_at(source, start);
    if first.text.starts_with('{') {
        return bare_object(source, start);
    }
    let (format, closers) = opener(first.text)?;
    let mut cursor = first.next;
    while cursor < source.len() {
        let line = line_at(source, cursor);
        let text = line.text.trim_end_matches([' ', '\t']);
        if closers.contains(&text) {
            return Some(Block {
                format,
                span: start..line.next,
                content: first.next..line.start,
            });
        }
        cursor = line.next;
    }
    None
}

/// Finds the JSON object that opens at `start`, which ends on the line of its
/// last byte.
fn bare_object(source: &str, start: usize) -> Option<Block> {
    let mut reader = JsonStreamReader::new(&source.as_bytes()[start..]);
    reader.skip_value().ok()?;
    let length = usize::try_from(reader.current_position(false).data_pos?).ok()?;
    let rest = line_at(source, start + length);
    if !rest
        .text
        .chars()
        .all(|character| matches!(character, ' ' | '\t'))
    {
        return None;
    }
    Some(Block {
        format: Format::Json,
        span: start..rest.next,
        content: start..rest.next,
    })
}
