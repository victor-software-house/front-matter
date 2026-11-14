# Design

## Context

hakkiri reads front matter through ctl-core's config input, which reads JSON as YAML, and keeps the rules that turn a value into prose: how lines fold, which plain scalars are dates.
This crate takes the reading: where the block is, which parser reads it, and where each value sits in the file.
The caller keeps what it does with a value.

## Decisions

### 1. The interface

```rust
pub enum Format { Yaml, Toml, Json }

pub struct Block { pub format: Format, pub span: Range<usize>, pub content: Range<usize> }

pub fn find(source: &str) -> Option<Block>;
pub fn read(source: &str, block: &Block) -> Result<Vec<Value>, Fault>;

pub struct Value {
    pub key: String,
    pub key_span: Range<usize>,
    pub span: Range<usize>,
    pub style: Style,
    pub text: Option<String>,
}

pub struct Fault { pub line: usize, pub message: String }
```

`span` of a `Block` covers the fences, and `content` the bytes between them.
Every span is a byte range of `source`.
`Value::span` is the value as written: a quoted string with its quotes, a block scalar with its header line.
`text` is the string with its escapes resolved, or `None` when the value is no string.
`Fault::line` counts from 1 in `source`.

Alternative: give the caller a tree of every nested value. hakkiri reads top-level strings only, as Vale does, and a nested reader can come when a caller needs it.

### 2. How a value is written

`Style` names the form of a value: `Plain`, `SingleQuoted`, `DoubleQuoted`, `Literal` and `Folded` for YAML, `Basic`, `Literal`, `MultilineBasic` and `MultilineLiteral` for TOML strings, `String` for a JSON string, and `Other` for a number, a boolean, a null, a date of TOML, a list or a table.
A YAML plain scalar that the core schema reads as a boolean, a null or a number is `Other`.
A YAML date stays `Plain` with its text, since the core schema reads it as a string; a caller that follows another schema decides.

Alternative: one `is_string` flag. The caller folds the lines of each form differently, so it needs the form.

### 3. Finding the block

The first line of `source` names the block, with blanks allowed after the fence:

| First line | Format | Closing line |
|:--|:--|:--|
| `---` | YAML | `---` or `...` |
| `---yaml` | YAML | `---` or `...` |
| `+++` | TOML | `+++` |
| `---toml` | TOML | `---` |
| `;;;` | JSON | `;;;` |
| `---json` | JSON | `---` |
| a line that starts with `{` | JSON | the line where the object closes |

A block whose closing line never comes is no block.
For a bare object, `struson` reads one JSON value from the first byte, and the block ends after the line that holds its last byte; the object is its own fence.
A bare object that `struson` cannot read to its end is no block, since nothing else marks where it ends, so decision 5 holds only for the fenced forms.
This is the set of [adrg/frontmatter][adrg], which Hugo and many static site tools share.

### 4. One parser per format

YAML is read from the events of [`granit-parser`][granit], whose marks hold byte offsets: the mapping at the top, each key scalar, and each value's first and last mark.
TOML is read with `toml`'s de-tables, whose `Spanned` items hold the span of each key and value.
JSON is read with [`struson`][struson], whose reader positions hold the byte offset of each value.
A format is never read by another format's parser, so `;;; {title: Install guide} ;;;` is a fault, and YAML reads it.

`struson` 0.8.0 passes the 3-day publish-age cooldown of the workspace on 2026-11-15; the crate starts on 0.7.2, which holds the same positions, and moves to 0.8 then.

Alternative: serde into a map. It loses the spans and the form of each value.

### 5. A fault keeps the body readable

A block its parser refuses gives a `Fault` whose line is the line of `source` where the parser stopped, and `find` still gives the block's span, so a caller lints the body after it.
A top-level value that is no mapping, such as a YAML list, is a fault too: front matter maps keys to values.

### 6. Tests

Each form of decision 3 has a case, with its spans checked against the input.
The JSON reader passes a subset of [JSONTestSuite][suite]: each `y_` case that is an object reads, and each `n_` case gives a fault; the cases live under `tests/json-suite/` with the suite's MIT notice.

[adrg]: https://github.com/adrg/frontmatter
[granit]: https://crates.io/crates/granit-parser
[struson]: https://crates.io/crates/struson
[suite]: https://github.com/nst/JSONTestSuite
