//! What the text readers share: decoding, tokens with their line numbers, numbers, and errors.

use std::borrow::Cow;

use crate::{MAX_TABLE_VALUES, PhotometryError};

/// Larger files are refused before reading. Real photometric files are a few hundred kilobytes.
pub const MAX_FILE_BYTES: usize = 16 << 20;

/// Why a file could not be read, and the line (1-based) it concerns.
#[derive(Debug, Clone, PartialEq)]
pub struct ReadError {
    pub line: usize,
    pub kind: ReadErrorKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReadErrorKind {
    /// More than [`MAX_FILE_BYTES`].
    TooLarge { bytes: usize },
    /// The file ends before this item.
    Missing(&'static str),
    /// This item is not a (finite) number.
    NotANumber { what: &'static str, found: String },
    /// This item has a value the format does not allow.
    Invalid { what: &'static str, reason: &'static str },
    /// Allowed by the format, not supported by luxpath yet.
    Unsupported(&'static str),
    /// The distribution breaks an invariant of [`crate::IntensityTable`].
    Table(PhotometryError),
}

impl std::fmt::Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}: ", self.line)?;
        match &self.kind {
            ReadErrorKind::TooLarge { bytes } => {
                write!(f, "{bytes} bytes is more than the {MAX_FILE_BYTES}-byte limit")
            }
            ReadErrorKind::Missing(what) => write!(f, "the file ends before the {what}"),
            ReadErrorKind::NotANumber { what, found } => write!(f, "the {what} should be a number, found `{found}`"),
            ReadErrorKind::Invalid { what, reason } => write!(f, "the {what} {reason}"),
            ReadErrorKind::Unsupported(what) => write!(f, "not supported yet: {what}"),
            ReadErrorKind::Table(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for ReadError {}

pub(crate) fn error(line: usize, kind: ReadErrorKind) -> ReadError {
    ReadError { line, kind }
}

/// A table error at the line it concerns: the first line of the γ angles, of the C angles, or of
/// the values.
pub(crate) fn table_error(e: PhotometryError, c_line: usize, gamma_line: usize, values_line: usize) -> ReadError {
    let line = match &e {
        PhotometryError::BadAngles { which: "gamma", .. } => gamma_line,
        PhotometryError::BadAngles { .. } => c_line,
        _ => values_line,
    };
    error(line, ReadErrorKind::Table(e))
}

/// Whether `line` is the `TILT=` line of an IES file.
pub(crate) fn is_tilt_line(line: &[u8]) -> bool {
    let line = line.trim_ascii_start();
    line.get(..4).is_some_and(|w| w.eq_ignore_ascii_case(b"TILT"))
        && line.get(4..).is_some_and(|rest| rest.trim_ascii_start().starts_with(b"="))
}

pub(crate) fn invalid(line: usize, what: &'static str, reason: &'static str) -> ReadError {
    error(line, ReadErrorKind::Invalid { what, reason })
}

/// The file as text: UTF-8 if it is, otherwise Latin-1 (older EULUMDAT and IES files), without a
/// byte-order mark.
pub(crate) fn decode(bytes: &[u8]) -> Result<Cow<'_, str>, ReadError> {
    if bytes.len() > MAX_FILE_BYTES {
        return Err(error(1, ReadErrorKind::TooLarge { bytes: bytes.len() }));
    }
    let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes);
    let text = match std::str::from_utf8(bytes) {
        Ok(s) => Cow::Borrowed(s),
        Err(_) => Cow::Owned(bytes.iter().map(|b| char::from(*b)).collect()),
    };
    // Classic Mac OS files end lines with a bare carriage return.
    if text.contains('\r') && !text.contains('\n') {
        return Ok(Cow::Owned(text.replace('\r', "\n")));
    }
    Ok(text)
}

/// A short, char-safe excerpt of what was found, for messages.
pub(crate) fn excerpt(found: &str) -> String {
    let mut out: String = found.chars().take(40).collect();
    if found.chars().nth(40).is_some() {
        out.push('…');
    }
    out
}

/// A finite number.
pub(crate) fn number(line: usize, what: &'static str, found: &str) -> Result<f64, ReadError> {
    found
        .parse::<f64>()
        .ok()
        .filter(|v| v.is_finite())
        .ok_or_else(|| error(line, ReadErrorKind::NotANumber { what, found: excerpt(found) }))
}

/// A count: a whole number from 0 to [`MAX_TABLE_VALUES`] (written as `12` or `12.0`).
pub(crate) fn count(line: usize, what: &'static str, found: &str) -> Result<usize, ReadError> {
    let v = number(line, what, found)?;
    if v < 0.0 || v.fract() != 0.0 || v > MAX_TABLE_VALUES as f64 {
        return Err(invalid(line, what, "is not a whole number in range"));
    }
    // Whole, non-negative and at most MAX_TABLE_VALUES: the cast is exact.
    Ok(v as usize)
}

fn is_separator(c: char) -> bool {
    // Commas separate values in some IES files; 0x1A ends files written by old DOS tools.
    c.is_whitespace() || c == ',' || c == '\u{1A}'
}

/// The whitespace- or comma-separated words of some lines, each with its line number.
#[derive(Clone)]
pub(crate) struct Tokens<'a> {
    lines: std::iter::Enumerate<std::str::Lines<'a>>,
    first_line: usize,
    line: usize,
    rest: &'a str,
}

impl<'a> Tokens<'a> {
    /// The words of `text`, whose first line is line `first_line` of the file.
    pub(crate) fn new(text: &'a str, first_line: usize) -> Self {
        Self { lines: text.lines().enumerate(), first_line, line: first_line, rest: "" }
    }

    /// The line of the next word (or of the last, at the end).
    pub(crate) fn next_line(&self) -> usize {
        self.clone().next().map_or(self.line, |(line, _)| line)
    }

    /// The next word as a number, or an error naming `what`.
    pub(crate) fn next_number(&mut self, what: &'static str) -> Result<(usize, f64), ReadError> {
        let (line, word) = self.next().ok_or_else(|| error(self.line, ReadErrorKind::Missing(what)))?;
        Ok((line, number(line, what, word)?))
    }

    /// The next word as a count.
    pub(crate) fn next_count(&mut self, what: &'static str) -> Result<(usize, usize), ReadError> {
        let (line, word) = self.next().ok_or_else(|| error(self.line, ReadErrorKind::Missing(what)))?;
        Ok((line, count(line, what, word)?))
    }
}

impl<'a> Iterator for Tokens<'a> {
    type Item = (usize, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let rest = self.rest.trim_start_matches(is_separator);
            if !rest.is_empty() {
                // `find` returns a char boundary, so the split cannot fail.
                let end = rest.find(is_separator).unwrap_or(rest.len());
                let (word, tail) = rest.split_at_checked(end).unwrap_or((rest, ""));
                self.rest = tail;
                return Some((self.line, word));
            }
            let (i, line) = self.lines.next()?;
            self.line = self.first_line.saturating_add(i);
            self.rest = line;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_carry_their_line_numbers() {
        let words: Vec<_> = Tokens::new("1 2,3\n\n  4\u{1A}\n", 10).collect();
        assert_eq!(words, vec![(10, "1"), (10, "2"), (10, "3"), (12, "4")]);
        let mut t = Tokens::new("1\n\n2", 1);
        t.next();
        assert_eq!((t.line, t.next_line()), (1, 3));
    }

    #[test]
    fn numbers_must_be_finite_and_counts_whole() {
        assert_eq!(number(1, "x", "1e3"), Ok(1000.0));
        for bad in ["inf", "NaN", "1e999", "", "1,5", "x"] {
            assert!(number(1, "x", bad).is_err(), "{bad}");
        }
        assert_eq!(count(1, "n", "12.0"), Ok(12));
        for bad in ["-1", "1.5", "1e9"] {
            assert!(count(1, "n", bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn latin1_files_decode_and_large_ones_are_refused() {
        assert_eq!(decode(b"Lumi\xE8re").unwrap(), "Lumière");
        assert_eq!(decode(b"\xEF\xBB\xBFok").unwrap(), "ok");
        assert_eq!(decode(b"a\rb").unwrap(), "a\nb");
        assert!(is_tilt_line(b" tilt = NONE") && !is_tilt_line(b"Tilt Lighting Ltd"));
        assert!(matches!(
            decode(&vec![b' '; MAX_FILE_BYTES + 1]),
            Err(ReadError { kind: ReadErrorKind::TooLarge { .. }, .. })
        ));
    }

    #[test]
    fn excerpts_stop_at_a_char_boundary() {
        assert_eq!(excerpt(&"é".repeat(50)).chars().count(), 41);
    }
}
