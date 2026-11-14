//! The front matter of a Markdown or MDX file: the YAML, TOML or JSON block at
//! its top.
//!
//! The crate finds the block, reads it with a parser of its format, and gives
//! each top-level value its key, its byte span and its text.
//! The reader lands in the first release.
