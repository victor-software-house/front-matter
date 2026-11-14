---
front-matter: patch
"verctl:group": added
---

The first release finds the front matter block at the top of a Markdown or MDX file, in YAML, TOML or JSON.
It reads each top-level value with the parser of its format and gives its key, its byte span in the file, how it is written and its text.

A block that its parser refuses gives a fault with the line, and the body of the file stays readable.
