# Front matter

## ADDED Requirements

### Requirement: Find the block

`find` SHALL give the front matter block at the start of a source, with its format, its span with the fences, and the span of its content, for each form the design lists.
A source whose first line opens no form, or whose block never closes, SHALL have no block.

#### Scenario: A YAML block

- **WHEN** the source is `---\ntitle: Install\n---\nBody\n`
- **THEN** `find` gives a YAML block whose span ends after the second `---` line, and whose content is `title: Install\n`

#### Scenario: A YAML block closed by dots

- **WHEN** the source is `---\ntitle: Install\n...\nBody\n`
- **THEN** `find` gives a YAML block that ends after the `...` line

#### Scenario: A TOML block

- **WHEN** the source is `+++\ntitle = "Install"\n+++\n`
- **THEN** `find` gives a TOML block

#### Scenario: A JSON block with fences

- **WHEN** the source is `;;;\n{"title": "Install"}\n;;;\n`
- **THEN** `find` gives a JSON block whose content is the object line

#### Scenario: A bare JSON object

- **WHEN** the source is `{\n  "title": "Install"\n}\nBody\n`
- **THEN** `find` gives a JSON block that ends after the line holding the closing brace

#### Scenario: A block that never closes

- **WHEN** the source is `---\ntitle: Install\nBody\n`
- **THEN** `find` gives no block

#### Scenario: A rule that is no fence

- **WHEN** the source starts with a blank line, then `---`
- **THEN** `find` gives no block

### Requirement: Read each top-level value

`read` SHALL give, for each top-level key of the block in source order, its key, the byte span of the key, the byte span of the value as written, its `Style`, and its text when the value is a string.
Every span SHALL be a range of the source, so `&source[span]` is the value as the file writes it.

#### Scenario: A quoted YAML string

- **WHEN** the block is `---\ntitle: "Install é"\n---\n`
- **THEN** the value of `title` has style `DoubleQuoted`, a span that holds both quotes, and the text `Install é`

#### Scenario: A YAML block scalar

- **WHEN** the block holds `description: |\n  One.\n  Two.\n`
- **THEN** the value has style `Literal`, a span that starts at `|`, and the text `One.\nTwo.\n`

#### Scenario: A value that is no string

- **WHEN** the block holds `draft: true` and `tags: [a, b]`
- **THEN** both values have style `Other` and no text

#### Scenario: A TOML multiline string

- **WHEN** the block holds `summary = """\nOne.\n"""`
- **THEN** the value has style `MultilineBasic` and the text `One.\n`

#### Scenario: A JSON string with an escape

- **WHEN** the block is `;;;\n{"title": "Tab\\there"}\n;;;\n`
- **THEN** the value has style `String`, a span with its quotes, and the text with a tab

### Requirement: One parser per format

Each format SHALL be read only by its own parser: YAML by `granit-parser`, TOML by `toml`, and JSON by `struson`.

#### Scenario: YAML flow syntax in a JSON block

- **WHEN** the block is `;;;\n{title: Install}\n;;;\n`
- **THEN** `read` gives a fault, because the key is not a JSON string

### Requirement: A fault names its line

A block that its parser refuses, or whose top level is no mapping, SHALL give a `Fault` with the line of the source where reading stopped, counted from 1, and the parser's message.
`find` SHALL still give the block's span for such a block, so a caller reads the rest of the file.

#### Scenario: A YAML syntax error

- **WHEN** the source is `---\ntitle: Install\n  bad: [\n---\nBody\n`
- **THEN** `find` gives a block that ends after the second `---`, and `read` gives a fault on line 3

#### Scenario: A top-level list

- **WHEN** the block is `---\n- a\n- b\n---\n`
- **THEN** `read` gives a fault on line 2
