# front-matter

The front matter of a Markdown or MDX file: the YAML, TOML or JSON block at its top.  
front-matter finds the block, reads it with a parser of its format, and gives each top-level value its key, its byte span in the file, and its text.  
It is a Rust library, not a command.

## What it reads

The first release is planned to read these blocks:

- YAML between `---` lines, or between `---yaml` and `---`, closed by `---` or `...`.
- TOML between `+++` lines, or between `---toml` and `---`.
- JSON between `;;;` lines, between `---json` and `---`, or as an object that opens on the first line with `{`.

A block that its parser refuses gives an error with the line of the fault, and the body of the file stays readable.

## Parsers

YAML is read from the events of `granit-parser`, TOML from the spans of `toml`, and JSON with `struson`, so each value keeps the bytes it came from.

## License

MIT OR Apache-2.0.
