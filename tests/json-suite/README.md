# JSONTestSuite cases

These cases come from [JSONTestSuite](https://github.com/nst/JSONTestSuite) at commit `1ef36fa01286573e846ac449e8683f8833c5b26a`, folder `test_parsing`.
The suite is MIT licensed, and its notice is the `LICENSE` file beside the cases.

The cases are:

- every `y_object_*` file,
- every `n_object_*` and `n_structure_*` file whose first byte after whitespace is `{`.

`tests/json_suite.rs` reads each case inside a `;;;` block, as `;;;\n<case>\n;;;\n`.
A `y_` case must read, and an `n_` case must give a fault.

## Skipped

`find` takes `&str`, so a case that is not UTF-8 is left out:

- `n_object_lone_continuation_byte_in_key_and_trailing_comma.json`
