//! The one parser for the committed Swiss Ephemeris corpus manifests
//! (`data/*-corpus/manifest.txt`), shared by every parity gate (issue #223).
//!
//! A manifest ties a corpus CSV to its row count and fnv1a64 checksum, so a
//! gate fails closed on drift. Three layouts are in use:
//!
//! - a `slice <name> … rows=N checksum=C` line ([`slice_entry`]);
//! - one `file: <name> rows=N checksum=C` line per CSV ([`file_entries`]);
//! - a `rows: N` line and a `checksum=C` token (the crossings corpus,
//!   [`labelled_entry`]).
//!
//! Each gate maps [`ManifestError`] into its own error type; the messages are
//! the ones the per-gate copies gave.

use std::collections::BTreeMap;
use std::fmt;

/// A row count and an fnv1a64 checksum.
pub(crate) type ManifestEntry = (usize, u64);

/// Why a manifest could not be read. Every case fails the gate closed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ManifestError {
    /// No `slice` line.
    NoSliceLine,
    /// No `file:` line.
    NoFileLines,
    /// A `file:` line without a name, `rows=` and `checksum=`.
    MalformedFileLine(String),
    /// A field is absent; `line` is the line it was looked for on, if any.
    Missing {
        field: &'static str,
        line: Option<String>,
    },
    /// A field's value does not parse.
    Invalid { field: &'static str, reason: String },
}

impl fmt::Display for ManifestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoSliceLine => f.write_str("no slice line"),
            Self::NoFileLines => f.write_str("no `file:` lines found in manifest"),
            Self::MalformedFileLine(line) => write!(f, "malformed file line: {line}"),
            Self::Missing { field, line: None } => write!(f, "{field} missing"),
            Self::Missing {
                field,
                line: Some(line),
            } => write!(f, "{field} missing: {line}"),
            Self::Invalid { field, reason } => write!(f, "{field}: {reason}"),
        }
    }
}

impl std::error::Error for ManifestError {}

fn parse_rows(value: &str) -> Result<usize, ManifestError> {
    value.parse().map_err(|e| ManifestError::Invalid {
        field: "rows",
        reason: format!("{e}"),
    })
}

fn parse_checksum(value: &str) -> Result<u64, ManifestError> {
    value.parse().map_err(|e| ManifestError::Invalid {
        field: "checksum",
        reason: format!("{e}"),
    })
}

/// The `rows=` and `checksum=` tokens among `tokens`; a later token of the
/// same field wins. `line` names the line in a missing-field error.
fn entry_from_tokens<'a>(
    tokens: impl IntoIterator<Item = &'a str>,
    line: Option<&str>,
) -> Result<ManifestEntry, ManifestError> {
    let mut rows = None;
    let mut checksum = None;
    for token in tokens {
        if let Some(value) = token.strip_prefix("rows=") {
            rows = Some(parse_rows(value)?);
        } else if let Some(value) = token.strip_prefix("checksum=") {
            checksum = Some(parse_checksum(value)?);
        }
    }
    let missing = |field| ManifestError::Missing {
        field,
        line: line.map(str::to_string),
    };
    Ok((
        rows.ok_or_else(|| missing("rows="))?,
        checksum.ok_or_else(|| missing("checksum="))?,
    ))
}

/// The entry on the first `slice` line.
pub(crate) fn slice_entry(manifest: &str) -> Result<ManifestEntry, ManifestError> {
    let line = manifest
        .lines()
        .find(|line| line.trim_start().starts_with("slice"))
        .ok_or(ManifestError::NoSliceLine)?;
    entry_from_tokens(line.split_whitespace(), None)
}

/// Every `file: <name> rows=N checksum=C` line, by file name. A manifest
/// without one fails.
pub(crate) fn file_entries(
    manifest: &str,
) -> Result<BTreeMap<String, ManifestEntry>, ManifestError> {
    let mut entries = BTreeMap::new();
    for line in manifest.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("file:") else {
            continue;
        };
        let tokens: Vec<&str> = rest.split_whitespace().collect();
        if tokens.len() < 3 {
            return Err(ManifestError::MalformedFileLine(line.to_string()));
        }
        let entry = entry_from_tokens(tokens[1..].iter().copied(), Some(line))?;
        entries.insert(tokens[0].to_string(), entry);
    }
    if entries.is_empty() {
        return Err(ManifestError::NoFileLines);
    }
    Ok(entries)
}

/// The row count on a `rows: N` line and the `checksum=` token on any line.
pub(crate) fn labelled_entry(manifest: &str) -> Result<ManifestEntry, ManifestError> {
    let mut rows = None;
    let mut checksum = None;
    for line in manifest.lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix("rows:") {
            rows = Some(parse_rows(value.trim())?);
        }
        for token in line.split_whitespace() {
            if let Some(value) = token.strip_prefix("checksum=") {
                checksum = Some(parse_checksum(value)?);
            }
        }
    }
    let missing = |field| ManifestError::Missing { field, line: None };
    Ok((
        rows.ok_or_else(|| missing("rows:"))?,
        checksum.ok_or_else(|| missing("checksum="))?,
    ))
}

#[cfg(test)]
mod tests;
