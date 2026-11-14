# Tasks

## 1. Find

- [ ] 1.1 Add `Format`, `Block` and `find` for every form of design decision 3, the bare object read by `struson`.
  Verify: a test per form, the line-ending and byte-order-mark cases, and the no-block cases of the spec check the spans against the input.

## 2. Read

- [ ] 2.1 Add `Value`, `Style`, `Fault` and `read` for YAML from `granit-parser` events.
  Verify: the YAML scenarios of the spec pass, block scalars and the core schema's non-strings included.
- [ ] 2.2 Read TOML through `toml`'s spanned tables.
  Verify: the TOML scenarios pass, each of the four string forms with its style.
- [ ] 2.3 Read JSON through `struson`.
  Verify: the JSON scenarios pass, and the YAML flow object in a JSON block is a fault.
- [ ] 2.4 Give a fault the line of the source, for a parser error and for a top level that is no mapping.
  Verify: the fault scenarios pass.

## 3. Suite and docs

- [ ] 3.1 Add the JSONTestSuite cases under `tests/json-suite/` with the suite's MIT notice.
  Verify: each `y_` object case reads, and each `n_` case gives a fault.
- [ ] 3.2 Document the interface in the crate doc and the README, with one example per format.
  Verify: `test:doc` runs the examples in CI.
- [ ] 3.3 Add a `.changeset/` fragment for the first release.
  Verify: the Changes check passes.
