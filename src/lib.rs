//! The front matter of a Markdown or MDX file: the YAML, TOML or JSON block at
//! its top.
//!
//! [`find`] gives the block at the start of a source: its format, its span with
//! the fences, and the span between them.
//! [`read`] reads the block with the parser of its format and gives each
//! top-level value its key, its byte span and its text.
//!
//! Every span is a byte range of the source as given, so `&source[span]` is
//! what the file holds.
//!
//! A block its parser refuses gives a [`Fault`] with the line of the fault, and
//! the span from [`find`] still marks where the body of the file starts.
//!
//! # YAML
//!
//! ```
//! let source = "---\ntitle: \"Install é\"\n---\nBody\n";
//! let block = front_matter::find(source).expect("a block");
//! assert_eq!(block.format, front_matter::Format::Yaml);
//!
//! let values = front_matter::read(source, &block)?;
//! assert_eq!(values[0].key, "title");
//! assert_eq!(values[0].style, front_matter::Style::DoubleQuoted);
//! assert_eq!(&source[values[0].span.clone()], "\"Install é\"");
//! assert_eq!(values[0].text.as_deref(), Some("Install é"));
//! # Ok::<(), front_matter::Fault>(())
//! ```
//!
//! # TOML
//!
//! ```
//! let source = "+++\ntitle = 'Install'\n+++\nBody\n";
//! let block = front_matter::find(source).expect("a block");
//! assert_eq!(block.format, front_matter::Format::Toml);
//!
//! let values = front_matter::read(source, &block)?;
//! assert_eq!(values[0].style, front_matter::Style::LiteralString);
//! assert_eq!(&source[values[0].span.clone()], "'Install'");
//! assert_eq!(values[0].text.as_deref(), Some("Install"));
//! # Ok::<(), front_matter::Fault>(())
//! ```
//!
//! # JSON
//!
//! ```
//! let source = ";;;\n{\"title\": \"Install\", \"draft\": true}\n;;;\nBody\n";
//! let block = front_matter::find(source).expect("a block");
//! assert_eq!(block.format, front_matter::Format::Json);
//!
//! let values = front_matter::read(source, &block)?;
//! assert_eq!(values[0].text.as_deref(), Some("Install"));
//! assert_eq!(values[1].style, front_matter::Style::Other);
//! assert_eq!(&source[values[1].span.clone()], "true");
//! # Ok::<(), front_matter::Fault>(())
//! ```

mod block;
mod read_json;
mod read_toml;
mod read_yaml;
mod value;

pub use block::{Block, Format, find};
pub use value::{Fault, Style, Value, read};
