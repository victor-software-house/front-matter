//! The `JSONTestSuite` object cases, each read inside a `;;;` block.
//! A `y_` case reads, and an `n_` case gives a fault on a line of the block.

use std::fs;
use std::path::Path;

use front_matter::{Fault, find, read};

/// The cases whose name starts with `prefix`, as file name and text, in name
/// order.
fn cases(prefix: &str) -> Vec<(String, String)> {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/json-suite");
    let mut cases: Vec<(String, String)> = fs::read_dir(folder)
        .unwrap_or_else(|error| panic!("the suite folder does not open: {error}"))
        .map(|entry| entry.unwrap_or_else(|error| panic!("an entry does not read: {error}")))
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            name.starts_with(prefix).then(|| {
                let text = fs::read_to_string(entry.path())
                    .unwrap_or_else(|error| panic!("{name} is not UTF-8 text: {error}"));
                (name, text)
            })
        })
        .collect();
    cases.sort();
    assert!(!cases.is_empty(), "no case starts with {prefix}");
    cases
}

/// Reads a case inside a `;;;` block.
fn read_case(case: &str) -> Result<usize, Fault> {
    let source = format!(";;;\n{case}\n;;;\n");
    let block = find(&source).ok_or_else(|| Fault {
        line: 0,
        message: "the case is inside a block that find does not give".to_owned(),
    })?;
    read(&source, &block).map(|values| values.len())
}

#[test]
fn each_y_case_reads() {
    for (name, case) in cases("y_") {
        if let Err(fault) = read_case(&case) {
            panic!("{name} does not read: {fault}");
        }
    }
}

#[test]
fn each_n_case_gives_a_fault() {
    for (name, case) in cases("n_") {
        match read_case(&case) {
            Ok(count) => panic!("{name} reads with {count} values"),
            Err(fault) => assert!(
                fault.line >= 2,
                "{name} has its fault on line {}, before the case",
                fault.line
            ),
        }
    }
}
