//! Reading a TOML block through the spans of `toml`.

use toml::Spanned;
use toml::de::{DeString, DeTable, DeValue};

use crate::value::{Fault, Style, Value, shift};

/// Reads the top-level keys of a TOML document in source order.
pub(crate) fn read(source: &str, text: &str, base: usize) -> Result<Vec<Value>, Fault> {
    let table = DeTable::parse(text).map_err(|error| {
        let at = error.span().map_or(0, |span| span.start);
        Fault::at(source, base + at, error.message())
    })?;
    // The table is ordered by key, so the source order comes from the key
    // spans.
    let mut values = table
        .get_ref()
        .iter()
        .map(|(key, value)| value_of(source, key, value, text, base))
        .collect::<Result<Vec<Value>, Fault>>()?;
    values.sort_by_key(|value| value.key_span.start);
    Ok(values)
}

/// A key and its value, with the spans moved to the source.
fn value_of(
    source: &str,
    key: &Spanned<DeString<'_>>,
    value: &Spanned<DeValue<'_>>,
    text: &str,
    base: usize,
) -> Result<Value, Fault> {
    let span = value.span();
    let (style, string) = if let DeValue::String(string) = value.get_ref() {
        let written = &text[span.clone()];
        let style = string_style(written);
        let string = string_text(source, written, string, &style, base + span.start)?;
        (style, Some(string))
    } else {
        (Style::Other, None)
    };
    Ok(Value {
        key: key.get_ref().to_string(),
        key_span: shift(key.span(), base),
        span: shift(span, base),
        style,
        text: string,
    })
}

/// The text of a string, with `\n` for each line break.
/// `toml` keeps a raw `\r\n` of a multiline string.
/// Such a string is read again with `\n` in its place.
/// An escaped `\r\n` stays as it is.
fn string_text(
    source: &str,
    written: &str,
    parsed: &str,
    style: &Style,
    offset: usize,
) -> Result<String, Fault> {
    if !matches!(style, Style::MultilineBasic | Style::MultilineLiteral)
        || !written.contains("\r\n")
    {
        return Ok(parsed.to_owned());
    }
    let unix = written.replace("\r\n", "\n");
    let normalized =
        DeValue::parse(&unix).map_err(|error| Fault::at(source, offset, error.message()))?;
    normalized
        .get_ref()
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| Fault::at(source, offset, "the string no longer reads as a string"))
}

/// The style of a string, from the first bytes of its span.
fn string_style(written: &str) -> Style {
    if written.starts_with("\"\"\"") {
        Style::MultilineBasic
    } else if written.starts_with("'''") {
        Style::MultilineLiteral
    } else if written.starts_with('\'') {
        Style::LiteralString
    } else {
        Style::Basic
    }
}
