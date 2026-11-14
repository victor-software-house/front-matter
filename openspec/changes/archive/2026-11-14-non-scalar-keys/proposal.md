# Proposal

## Why

YAML lets a key be a list, a mapping or an alias, as in `? [a, b]`.
`read` gives a fault for such a key, so a valid block fails, and a caller loses every value of it.
hakkiri, the first caller, reads the other entries of such a block today and must keep doing so when it reads front matter through this crate.

## What Changes

- A top-level key that is no scalar holds no name, so its entry gives no value.
- `read` goes on with the entries after it, and gives no fault for it.

## Capabilities

### Modified Capabilities

- `front-matter`: an entry whose key is no scalar is skipped.

## Impact

- A block that gave a fault for such a key now reads.
- No public type changes.
