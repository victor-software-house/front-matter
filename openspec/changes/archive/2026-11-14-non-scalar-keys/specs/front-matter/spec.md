## MODIFIED Requirements

### Requirement: Read each top-level value

`read` SHALL give each top-level key of the block in source order.
For each key it SHALL give the key, the byte span of the key, the byte span of the value as written, its `Style`, and its text when the value is a string.
Every span SHALL be a range of the source, so `&source[span]` is the value as the file writes it.
A YAML key that is a list, a mapping or an alias holds no name, so its entry SHALL give no value, and the entries after it SHALL be read.

#### Scenario: A quoted YAML string

- **WHEN** the block is `---\ntitle: "Install é"\n---\n`
- **THEN** the value of `title` has style `DoubleQuoted`, a span that holds both quotes, and the text `Install é`

#### Scenario: A YAML block scalar

- **WHEN** the block holds `description: |\n  One.\n  Two.\n`
- **THEN** the value has style `Literal`, a span that starts at `|`, and the text `One.\nTwo.\n`

#### Scenario: A value that is no string

- **WHEN** the block holds `draft: true` and `tags: [a, b]`
- **THEN** both values have style `Other` and no text

#### Scenario: An explicit string tag

- **WHEN** the block is `---\ncount: !!str 12\n---\n`
- **THEN** the value of `count` has style `Plain` and the text `12`

#### Scenario: A TOML multiline string

- **WHEN** the block holds `summary = """\nOne.\n"""`
- **THEN** the value has style `MultilineBasicString` and the text `One.\n`

#### Scenario: A JSON string with an escape

- **WHEN** the block is `;;;\n{"title": "Tab\\there"}\n;;;\n`
- **THEN** the value has style `String`, a span with its quotes, and the text with a tab

#### Scenario: A YAML key that is no scalar

- **WHEN** the block holds `? [a, b]\n: c\ntitle: Install\n`
- **THEN** `read` gives one value, `title`, and no fault
