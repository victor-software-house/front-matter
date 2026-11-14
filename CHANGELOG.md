# Changelog

## front-matter 0.0.2

### Fixed

- A YAML key that is a list, a mapping or an alias, such as `? [a, b]`, no longer makes `read` fail.
  Its entry gives no value, and the entries after it are read ([`e7f570d`](https://github.com/victor-software-house/front-matter/commit/e7f570d4758c50298034c5fff3cade601e2acfad)).

## front-matter 0.0.1

### Added

- The first release finds the front matter block at the top of a Markdown or MDX file, in YAML, TOML or JSON.
  It reads each top-level value with the parser of its format and gives its key, its byte span in the file, how it is written and its text ([`0f7ad4d`](https://github.com/victor-software-house/front-matter/commit/0f7ad4d337c952d7a723b284025ab278321a95c8)).

  A block that its parser refuses gives a fault with the line, and the body of the file stays readable.

