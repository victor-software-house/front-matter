//! The values of a block, and the fault of a block that cannot be read.

use std::error::Error;
use std::fmt;
use std::ops::Range;

use crate::{Block, Format, read_json, read_toml, read_yaml};

/// How a value is written in the file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Style {
    /// A YAML plain scalar that the core schema reads as a string.
    Plain,
    /// A YAML scalar in single quotes.
    SingleQuoted,
    /// A YAML scalar in double quotes.
    DoubleQuoted,
    /// A YAML block scalar that starts with `|`.
    Literal,
    /// A YAML block scalar that starts with `>`.
    Folded,
    /// A TOML string in double quotes.
    Basic,
    /// A TOML string in single quotes.
    LiteralString,
    /// A TOML string in triple double quotes.
    MultilineBasic,
    /// A TOML string in triple single quotes.
    MultilineLiteral,
    /// A JSON string.
    String,
    /// A value that is no string: a number, a boolean, a null, a TOML date, a
    /// list or a table. A YAML plain scalar that the core schema reads as a
    /// boolean, a null or a number is also `Other`.
    Other,
}

/// A top-level value of a block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Value {
    /// The key.
    pub key: String,
    /// The bytes of the key in the source.
    pub key_span: Range<usize>,
    /// The bytes of the value as written: a quoted string with its quotes, a
    /// block scalar with its header line.
    /// A span never holds the line ending after the value.
    pub span: Range<usize>,
    /// How the value is written.
    pub style: Style,
    /// The string with its escapes resolved and `\n` for each line break, or
    /// `None` when the value is no string.
    pub text: Option<String>,
}

/// A block that its parser refuses, or whose top level is no mapping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fault {
    /// The line of the source where reading stopped, counted from 1.
    pub line: usize,
    /// What the parser reported.
    pub message: String,
}

impl Fault {
    /// A fault at `offset` of `source`, on the line that holds that byte.
    /// The line is one more than the number of `\n` bytes before the offset.
    pub(crate) fn at(source: &str, offset: usize, message: impl Into<String>) -> Self {
        let bytes = source.as_bytes();
        let before = bytes.get(..offset).unwrap_or(bytes);
        Self {
            line: before.split(|byte| *byte == b'\n').count(),
            message: message.into(),
        }
    }
}

impl fmt::Display for Fault {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "line {}: {}", self.line, self.message)
    }
}

impl Error for Fault {}

/// Moves a range of a block's content to a range of the source.
pub(crate) fn shift(range: Range<usize>, base: usize) -> Range<usize> {
    base + range.start..base + range.end
}

/// Reads each top-level value of `block`, in source order, with the parser of
/// its format.
///
/// # Errors
///
/// Returns a [`Fault`] when the parser refuses the block, or when its top level
/// is no mapping.
/// The block's span from [`find`](crate::find) stays valid, so a caller reads
/// the rest of the file.
///
/// ```
/// let source = "---\ntitle: Install\n  bad: [\n---\nBody\n";
/// let block = front_matter::find(source).expect("a block");
/// let fault = front_matter::read(source, &block).expect_err("a fault");
/// assert_eq!(fault.line, 3);
/// assert_eq!(&source[block.span.end..], "Body\n");
/// ```
pub fn read(source: &str, block: &Block) -> Result<Vec<Value>, Fault> {
    let text = source.get(block.content.clone()).ok_or_else(|| {
        Fault::at(
            source,
            0,
            "the content of the block is not a range of the source",
        )
    })?;
    let base = block.content.start;
    match block.format {
        Format::Yaml => read_yaml::read(source, text, base),
        Format::Toml => read_toml::read(source, text, base),
        Format::Json => read_json::read(source, text, base),
    }
}
