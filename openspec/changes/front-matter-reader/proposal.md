# Proposal

## Why

hakkiri lints the string values of a Markdown file's front matter, and reads them through a general config reader that reads JSON as YAML.
A JSON block that YAML also reads, such as `{title: Install guide}`, then passes as front matter when it is no JSON.
The operator ruled on 2026-11-13 that front matter is document content, read by one parser per format, and on 2026-11-14 that the reader lives in this crate so other tools can use it.

## What Changes

- `find` gives the block at the top of a file: its format, its span with the fences, and the span between them.
  It reads the forms of the README: `---`, `---yaml`, `+++`, `---toml`, `;;;`, `---json` and an object that opens with `{`; YAML may close with `...`.
- `read` gives each top-level value of the block: its key, the span of the key, the span of the value as written, how it is written, and its text when it is a string.
- YAML is read from `granit-parser` events, TOML from the spans of `toml`, and JSON with `struson`.
- A block its parser refuses gives a `Fault` with the line of the file and the parser's message.

## Capabilities

### New Capabilities

- `front-matter`: finding and reading the front matter of a Markdown or MDX file.

## Impact

- New dependencies: `granit-parser` 2, `toml` 1 and `struson` 0.7, each MIT OR Apache-2.0.
- The JSON tests pin a subset of JSONTestSuite, which is MIT, with its notice.
