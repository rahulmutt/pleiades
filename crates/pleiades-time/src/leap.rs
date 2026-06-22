//! Leap-second table (`TAI − UTC`) and lookup, checksum-pinned and fail-closed.

use std::sync::OnceLock;

use crate::error::CivilTimeError;
use crate::fnv1a64;

const LEAP_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/leap-seconds.csv"
));

/// FNV-1a checksum of `data/leap-seconds.csv`. Regenerate with the `pinned_checksum`
/// test below if the table is updated, and bump `VALID_THROUGH_JD` accordingly.
const LEAP_CSV_CHECKSUM: u64 = 16253160508809344072; // pinned

/// JD of the first UTC leap-second epoch (1972-01-01 00:00).
pub const LEAP_EPOCH_JD: f64 = 2441317.5;

/// Last UTC date the table is authoritative for (2025-12-31 00:00; no leap second
/// announced through 2025 per IERS Bulletin C, as-of 2026-06).
pub const VALID_THROUGH_JD: f64 = 2461040.5;

static LEAP_ROWS: OnceLock<Result<Vec<(f64, i32)>, CivilTimeError>> = OnceLock::new();

fn parse_table() -> Result<Vec<(f64, i32)>, CivilTimeError> {
    if fnv1a64(LEAP_CSV) != LEAP_CSV_CHECKSUM {
        return Err(CivilTimeError::StaleTimeData {
            kind: "leap-second",
        });
    }
    let mut rows = Vec::new();
    for line in LEAP_CSV.lines().skip(1) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split(',');
        let jd: f64 = parts.next().and_then(|s| s.trim().parse().ok()).ok_or(
            CivilTimeError::StaleTimeData {
                kind: "leap-second",
            },
        )?;
        let secs: i32 = parts.next().and_then(|s| s.trim().parse().ok()).ok_or(
            CivilTimeError::StaleTimeData {
                kind: "leap-second",
            },
        )?;
        rows.push((jd, secs));
    }
    Ok(rows)
}

fn table() -> Result<&'static [(f64, i32)], CivilTimeError> {
    LEAP_ROWS
        .get_or_init(parse_table)
        .as_deref()
        .map_err(|e| *e)
}

/// Returns `TAI − UTC` in whole seconds for a UTC instant, or `Ok(None)` if the
/// instant is before 1972 or after the table's validated horizon.
pub fn tai_minus_utc(jd_utc: f64) -> Result<Option<i32>, CivilTimeError> {
    if !(LEAP_EPOCH_JD..=VALID_THROUGH_JD).contains(&jd_utc) {
        return Ok(None);
    }
    let rows = table()?;
    let mut current = None;
    for &(effective, secs) in rows {
        if jd_utc >= effective {
            current = Some(secs);
        }
    }
    Ok(current)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pinned_checksum() {
        // If this fails after a deliberate table edit, copy the printed value into
        // LEAP_CSV_CHECKSUM and bump VALID_THROUGH_JD.
        assert_eq!(
            fnv1a64(LEAP_CSV),
            LEAP_CSV_CHECKSUM,
            "checksum = {}",
            fnv1a64(LEAP_CSV)
        );
    }

    #[test]
    fn lookup_at_known_boundaries() {
        // 2017-01-01 (JD 2457754.5) -> 37
        assert_eq!(tai_minus_utc(2457754.5).unwrap(), Some(37));
        // 2000-01-01 12:00 (JD 2451545.0) -> 32 (1999-01-01 epoch)
        assert_eq!(tai_minus_utc(2451545.0).unwrap(), Some(32));
        // 1972-01-01 -> 10
        assert_eq!(tai_minus_utc(2441317.5).unwrap(), Some(10));
    }

    #[test]
    fn returns_none_outside_window() {
        assert_eq!(tai_minus_utc(2441317.4).unwrap(), None); // before 1972
        assert_eq!(tai_minus_utc(VALID_THROUGH_JD + 1.0).unwrap(), None); // past horizon
    }
}
