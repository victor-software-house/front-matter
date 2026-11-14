//! Reading a YAML block from the events of `granit-parser`.

use std::ops::Range;

use granit_parser::{Event, Parser, ScalarStyle, ScanError, Span, StructureStyle};

use crate::value::{Fault, Style, Value, shift};

/// Reads the top-level keys of a YAML document in source order.
pub(crate) fn read(source: &str, text: &str, base: usize) -> Result<Vec<Value>, Fault> {
    let mut reader = Reader::default();
    for item in Parser::new_from_str(text) {
        let (event, span) = item.map_err(|error| scan_fault(source, base, &error))?;
        let range = byte_range(source, base, &span)?;
        reader.accept(source, &event, range)?;
    }
    Ok(reader.values)
}

/// A fault at the place where the parser stopped.
fn scan_fault(source: &str, base: usize, error: &ScanError) -> Fault {
    let at = error.marker().byte_offset().unwrap_or(0);
    Fault::at(source, base + at, error.info())
}

/// The bytes of a span in the source.
fn byte_range(source: &str, base: usize, span: &Span) -> Result<Range<usize>, Fault> {
    span.byte_range()
        .map(|range| shift(range, base))
        .ok_or_else(|| Fault::at(source, base, "the parser gave no byte offset"))
}

/// Where the reader is in the document.
#[derive(Debug, Default)]
enum Phase {
    /// Before the mapping at the top.
    #[default]
    Before,
    /// Inside the mapping at the top.
    Inside,
    /// After the mapping at the top.
    After,
}

/// A list or a mapping that is the value of a top-level key.
#[derive(Debug)]
struct Nested {
    /// The first byte of the node.
    start: usize,
    /// The byte after the last byte read of the node.
    end: usize,
    /// For each open collection, whether it is written in flow style.
    flow: Vec<bool>,
}

impl Nested {
    /// Takes an event inside the node, and tells whether the node is complete.
    /// A flow collection ends at its closing bracket, and a block collection
    /// ends where its last scalar does.
    fn accept(&mut self, event: &Event<'_>, range: &Range<usize>) -> bool {
        if let Event::SequenceStart(style, ..) | Event::MappingStart(style, ..) = event {
            self.flow.push(*style == StructureStyle::Flow);
            self.end = self.end.max(range.end);
        } else if matches!(event, Event::SequenceEnd | Event::MappingEnd) {
            if self.flow.pop() == Some(true) {
                self.end = self.end.max(range.end);
            }
            return self.flow.is_empty();
        } else if matches!(event, Event::Scalar(..) | Event::Alias(_)) {
            self.end = self.end.max(range.end);
        }
        false
    }
}

/// The events of the document, folded into the top-level values.
#[derive(Debug, Default)]
struct Reader {
    /// The values read so far.
    values: Vec<Value>,
    /// Where the reader is.
    phase: Phase,
    /// The key that waits for its value.
    key: Option<(String, Range<usize>)>,
    /// The list or mapping that is the value of that key.
    nested: Option<Nested>,
}

impl Reader {
    /// Takes the next event of the document.
    fn accept(
        &mut self,
        source: &str,
        event: &Event<'_>,
        range: Range<usize>,
    ) -> Result<(), Fault> {
        match self.phase {
            Phase::Before => {
                if is_node(event) {
                    if matches!(event, Event::MappingStart(..)) {
                        self.phase = Phase::Inside;
                    } else {
                        return Err(Fault::at(
                            source,
                            range.start,
                            "front matter must map keys to values",
                        ));
                    }
                }
                Ok(())
            }
            Phase::Inside => self.inside(source, event, range),
            Phase::After => Ok(()),
        }
    }

    /// Takes an event of the top-level mapping.
    fn inside(
        &mut self,
        source: &str,
        event: &Event<'_>,
        range: Range<usize>,
    ) -> Result<(), Fault> {
        if let Some(nested) = self.nested.as_mut() {
            if nested.accept(event, &range)
                && let Some(done) = self.nested.take()
            {
                self.finish(Style::Other, None, done.start..done.end);
            }
            return Ok(());
        }
        if matches!(event, Event::MappingEnd) {
            self.phase = Phase::After;
        } else if self.key.is_none() {
            if let Event::Scalar(text, ..) = event {
                self.key = Some((text.to_string(), range));
            } else if is_node(event) {
                return Err(Fault::at(source, range.start, "a key must be a scalar"));
            }
        } else if let Event::Scalar(text, style, ..) = event {
            let (style, text) = scalar(text, *style);
            let range = match (&style, &self.key) {
                (Style::Literal | Style::Folded, Some((_, key_span))) => {
                    block_scalar_span(source, key_span.end, range)
                }
                _ => range,
            };
            self.finish(style, text, range);
        } else if matches!(event, Event::Alias(_)) {
            self.finish(Style::Other, None, range);
        } else if let Event::SequenceStart(style, ..) | Event::MappingStart(style, ..) = event {
            self.nested = Some(Nested {
                start: range.start,
                end: range.end,
                flow: vec![*style == StructureStyle::Flow],
            });
        }
        Ok(())
    }

    /// Completes the value of the waiting key.
    fn finish(&mut self, style: Style, text: Option<String>, span: Range<usize>) {
        if let Some((key, key_span)) = self.key.take() {
            self.values.push(Value {
                key,
                key_span,
                span,
                style,
                text,
            });
        }
    }
}

/// The span of a block scalar, from its `|` or `>` to its last line.
/// The parser gives the content only, so the indicator is found after the key.
/// It stops before the line ending that ends the content.
fn block_scalar_span(source: &str, key_end: usize, content: Range<usize>) -> Range<usize> {
    let start = indicator(source, key_end, content.start).unwrap_or(content.start);
    let end = source[..content.end].trim_end_matches(['\n', '\r']).len();
    start..end.max(start + 1)
}

/// The byte of the `|` or `>` that opens a block scalar, searched from the end
/// of its key. Between the two there can be the colon, an anchor, a tag, a
/// comment and blanks.
fn indicator(source: &str, from: usize, until: usize) -> Option<usize> {
    let mut at = from;
    loop {
        let rest = source.get(at..until)?;
        let trimmed = rest.trim_start();
        at += rest.len() - trimmed.len();
        match trimmed.chars().next()? {
            ':' => at += 1,
            '#' => at += trimmed.find('\n').unwrap_or(trimmed.len()),
            '&' | '!' => {
                let length = if trimmed.starts_with("!<") {
                    trimmed.find('>')? + 1
                } else {
                    trimmed.find(char::is_whitespace).unwrap_or(trimmed.len())
                };
                at += length;
            }
            '|' | '>' => return Some(at),
            _ => return None,
        }
    }
}

/// Whether the event is a node: a scalar, an alias, a list or a mapping.
fn is_node(event: &Event<'_>) -> bool {
    matches!(
        event,
        Event::Scalar(..) | Event::Alias(_) | Event::SequenceStart(..) | Event::MappingStart(..)
    )
}

/// The style and the text of a scalar value.
fn scalar(text: &str, style: ScalarStyle) -> (Style, Option<String>) {
    let style = match style {
        ScalarStyle::Plain => Style::Plain,
        ScalarStyle::SingleQuoted => Style::SingleQuoted,
        ScalarStyle::DoubleQuoted => Style::DoubleQuoted,
        ScalarStyle::Literal => Style::Literal,
        ScalarStyle::Folded => Style::Folded,
    };
    if style == Style::Plain && is_core_non_string(text) {
        (Style::Other, None)
    } else {
        (style, Some(text.to_owned()))
    }
}

/// Whether the YAML 1.2 core schema reads a plain scalar as a null, a boolean
/// or a number. A date stays a string in that schema.
fn is_core_non_string(text: &str) -> bool {
    matches!(
        text,
        "" | "~"
            | "null"
            | "Null"
            | "NULL"
            | "true"
            | "True"
            | "TRUE"
            | "false"
            | "False"
            | "FALSE"
            | ".nan"
            | ".NaN"
            | ".NAN"
    ) || is_infinity(text)
        || is_number(text)
}

/// Whether the text is `.inf` with an optional sign.
fn is_infinity(text: &str) -> bool {
    let unsigned = text.strip_prefix(['-', '+']).unwrap_or(text);
    matches!(unsigned, ".inf" | ".Inf" | ".INF")
}

/// Whether the text is a core schema integer or float.
fn is_number(text: &str) -> bool {
    if let Some(digits) = text.strip_prefix("0x") {
        return all_of(digits, |character| character.is_ascii_hexdigit());
    }
    if let Some(digits) = text.strip_prefix("0o") {
        return all_of(digits, |character| ('0'..='7').contains(&character));
    }
    let unsigned = text.strip_prefix(['-', '+']).unwrap_or(text);
    let (mantissa, exponent) = match unsigned.split_once(['e', 'E']) {
        Some((mantissa, exponent)) => (mantissa, Some(exponent)),
        None => (unsigned, None),
    };
    let mantissa_ok = match mantissa.split_once('.') {
        Some((whole, fraction)) => {
            (whole.is_empty() || is_decimal(whole))
                && (fraction.is_empty() || is_decimal(fraction))
                && !(whole.is_empty() && fraction.is_empty())
        }
        None => is_decimal(mantissa),
    };
    mantissa_ok
        && exponent.is_none_or(|exponent| {
            is_decimal(exponent.strip_prefix(['-', '+']).unwrap_or(exponent))
        })
}

/// Whether the text holds one or more decimal digits and nothing else.
fn is_decimal(text: &str) -> bool {
    all_of(text, |character| character.is_ascii_digit())
}

/// Whether the text is not empty and every character passes.
fn all_of(text: &str, check: impl Fn(char) -> bool) -> bool {
    !text.is_empty() && text.chars().all(check)
}
