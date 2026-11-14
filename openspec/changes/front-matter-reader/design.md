# Design

## Context

hakkiri reads front matter through ctl-core's config input.
That input reads JSON as YAML.
hakkiri also holds the rules that turn a value into prose, such as how lines fold and which plain scalars are dates.

This crate finds the block and picks its parser.
It gives the byte span of each value in the file.
The caller decides what to do with a value.

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

`Style` names the form of a value:

| Format | Styles |
|:--|:--|
| YAML | `Plain`, `SingleQuoted`, `DoubleQuoted`, `Literal`, `Folded` |
| TOML | `Basic`, `Literal`, `MultilineBasic`, `MultilineLiteral` |
| JSON | `String` |
| any | `Other`, for a number, a boolean, a null, a TOML date, a list or a table |

A YAML plain scalar that the core schema reads as a boolean, a null or a number is `Other`.
The core schema reads a YAML date as a string, so it stays `Plain` with its text.
A caller that follows another schema decides for itself.

Alternative: one `is_string` flag. The caller folds the lines of each form differently, so it needs the form.

### 3. Finding the block

The first line of `source` names the block, with blanks allowed after the fence.
A leading U+FEFF is skipped, and the block's span starts after it.
A line ends with `\n` or `\r\n`, so a file saved on Windows reads the same.
A span never splits a `\r\n` and never ends on a `\r`: `content` starts after the opening line's ending, `span` holds the closing line's ending, and a value's span stops before it.
A closing line may end the file with no line ending, in either style.
The text of a string holds `\n` for each line break, whatever the file uses.
`Fault::line` counts the lines of `source`, so a byte order mark changes no line number.

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
For a bare object, `struson` reads one JSON value from the first byte.
The block ends after the line that holds the value's last byte, so the object is its own fence.
A bare object that `struson` cannot read to its end is no block, since nothing else marks where it ends.
Decision 5 therefore holds only for the fenced forms.
Only blanks may follow a bare object on its last line, or there is no block.
In a fenced JSON block, `read` allows only whitespace after the object (`consume_trailing_whitespace`), and any other data is a fault.

This is the set of [adrg/frontmatter][adrg], which Hugo and many static site tools share.

### 4. One parser per format

YAML is read from the events of [`granit-parser`][granit].
Each event's mark holds a byte offset (`Marker::byte_offset`), for the mapping at the top, each key scalar, and each value's first and last mark.
TOML is read with `toml`'s de-tables, whose `Spanned` items hold the span of each key and value.
JSON is read with [`struson`][struson], whose reader positions hold the byte offset of each value.

A format is never read by another format's parser.
A JSON block that holds `{title: Install guide}` is a fault, even though YAML would read it.

`struson` 0.8.0 passes the repository's 3-day publish-age cooldown on 2026-11-15.
The crate starts on 0.7.2, which holds the same positions, and moves to 0.8 then.

Alternative: serde into a map. It loses the spans and the form of each value.

### 5. A fault keeps the body readable

A block its parser refuses gives a `Fault` whose line is the line of `source` where the parser stopped, and `find` still gives the block's span, so a caller lints the body after it.
A top-level value that is no mapping, such as a YAML list, is a fault too: front matter maps keys to values.

### 6. Tests

Each form of decision 3 has a case, with its spans checked against the input.
The JSON reader passes a subset of [JSONTestSuite][suite].
Each `y_` case that is an object reads, and each `n_` case gives a fault.
The cases live under `tests/json-suite/` with the suite's MIT notice.

[adrg]: https://github.com/adrg/frontmatter
[granit]: https://crates.io/crates/granit-parser
[struson]: https://crates.io/crates/struson
[suite]: https://github.com/nst/JSONTestSuite
