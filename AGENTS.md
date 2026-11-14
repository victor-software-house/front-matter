# front-matter

The front matter of a Markdown or MDX file.  
**A library with no command.** Import it as `front_matter`.  
This file is the development contract.

## Scope

- Finding the block at the top of a file, in each form of the README.
- Reading it with the parser of its format, and giving each top-level value its key, byte span and text.
- Reporting a block its parser refuses, with the line of the fault.

A caller keeps its own rules, such as which values it lints and what it does with a fault.

## Rules

1. **One parser per format.** YAML through `granit-parser` events, TOML through `toml` spans, JSON through `struson`. A format is never read by another format's parser.
2. **Bytes of the file.** Every span is a byte range of the input as given, so a caller can point at the file as written.
3. **A fault never hides the body.** A refused block gives an error and the length of the block, so the caller can read the rest of the file.
4. **MIT OR Apache-2.0**, so permissive crates can depend on front-matter. No dependency may carry a copyleft license.
5. **Nothing local leaks.** The repository is public: no local path, machine name, private repository or credential in any tracked file.

## Workflow

- Every change goes through a pull request, and the pull request's CI verifies it.
- Shared repository files are rendered from the shared Copier template; change them through a template update.
