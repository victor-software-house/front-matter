//! Reading a JSON block with `struson`.
//!
//! After `peek` or `has_next`, the reader's data position is the first byte of
//! the next token, and after the token is consumed it is the byte after it.

use struson::reader::{JsonReader, JsonStreamReader, ReaderError, ValueType};

use crate::value::{Fault, Style, Value, shift};

/// The reader over the content of a block.
type Reader<'text> = JsonStreamReader<&'text [u8]>;

/// Reads the members of a JSON object, and refuses data after it.
pub(crate) fn read(source: &str, text: &str, base: usize) -> Result<Vec<Value>, Fault> {
    let mut reader = JsonStreamReader::new(text.as_bytes());
    let values = members(&mut reader, base)
        .map_err(|error| Fault::at(source, base + offset(&reader), error.to_string()))?;
    reader.consume_trailing_whitespace().map_err(|error| {
        let at = if let ReaderError::SyntaxError(syntax) = &error {
            syntax
                .location
                .data_pos
                .and_then(|position| usize::try_from(position).ok())
        } else {
            None
        };
        Fault::at(source, base + at.unwrap_or(text.len()), error.to_string())
    })?;
    Ok(values)
}

/// The byte of the content where the reader is.
fn offset(reader: &Reader<'_>) -> usize {
    reader
        .current_position(false)
        .data_pos
        .map_or(0, |position| {
            usize::try_from(position).unwrap_or(usize::MAX)
        })
}

/// Reads the top-level object, member by member.
fn members(reader: &mut Reader<'_>, base: usize) -> Result<Vec<Value>, ReaderError> {
    let mut values = Vec::new();
    reader.begin_object()?;
    while reader.has_next()? {
        let key_start = offset(reader);
        let key = reader.next_name_owned()?;
        let key_end = offset(reader);
        let kind = reader.peek()?;
        let start = offset(reader);
        let (style, text) = if kind == ValueType::String {
            (Style::String, Some(reader.next_string()?))
        } else {
            reader.skip_value()?;
            (Style::Other, None)
        };
        values.push(Value {
            key,
            key_span: shift(key_start..key_end, base),
            span: shift(start..offset(reader), base),
            style,
            text,
        });
    }
    reader.end_object()?;
    Ok(values)
}
