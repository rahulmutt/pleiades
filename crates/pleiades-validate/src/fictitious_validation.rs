//! Fail-closed two-tier `validate-fictitious` gate over the committed
//! Swiss-Ephemeris fictitious-body reference corpus (Task 8,
//! `data/fictitious-corpus/{fictitious.csv,manifest.txt}`).
//!
//! Tier 1 (self-consistency, no SE reference): every row is recomputed with
//! [`FictitiousBackend::position`] and the backend's own output is checked for
//! internal consistency — finite longitude/latitude/distance, longitude in
//! `[0, 360)`, latitude in `[-90, 90]`.
//!
//! Tier 2 (SE parity): each recomputed row is compared to the Swiss-Ephemeris
//! `lon_deg`/`lat_deg`/`dist_au` columns. The corpus is GEOMETRIC (no
//! light-time, aberration, or gravitational deflection — see
//! `tools/se-fictitious-reference`'s module doc), matching
//! `FictitiousBackend::position`'s geocentric-J2000 mean/geometric output
//! exactly, so this is a like-for-like comparison with no frame or
//! light-time correction needed.
//!
//! **Nibiru** (SE body 49) is a documented outlier: its `seorbel.txt` equinox
//! is ~370 AD (~1630 years before J2000), well beyond the accurate range of
//! the IAU-1976 precession helper `pleiades_fict::frame::rotate_ecliptic_to_j2000`
//! uses to carry its elements to the J2000 mean ecliptic (see
//! `docs/plans/fictitious-bodies` Reconciliation §2). Its residual is gated
//! under a separate, wider per-body ceiling
//! ([`crate::fictitious_thresholds::NIBIRU_LONGITUDE_ARCSEC`] etc.) rather than
//! inflating the global ceilings that gate the other 18 bodies.
//!
//! A sibling `manifest.txt` records the fnv1a64 digest of the CSV (drift
//! guard); a mismatch fails the gate closed.

use crate::fictitious_thresholds::*;
use pleiades_apparent::fnv1a64;
use pleiades_backend::{EphemerisBackend, EphemerisRequest};
use pleiades_data::packaged_backend;
use pleiades_fict::FictitiousBackend;
use pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale};
use std::collections::BTreeMap;

const CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/fictitious-corpus/fictitious.csv"
));
const MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/fictitious-corpus/manifest.txt"
));
const CSV_FILE: &str = "fictitious.csv";

/// Fixture row count pinned by the corpus (Task 8: 19 bodies x 30 samples).
/// Update when the corpus is regenerated.
pub const EXPECTED_ROWS: usize = 570;

#[derive(Debug)]
pub enum FictitiousError {
    /// A committed CSV's digest disagrees with the manifest.
    ChecksumMismatch {
        file: &'static str,
        got: u64,
        want: u64,
    },
    /// The parsed corpus row count disagrees with [`EXPECTED_ROWS`].
    RowCountMismatch { expected: usize, got: usize },
    /// A Tier-2 residual exceeded its ceiling.
    ToleranceExceeded {
        category: &'static str,
        label: String,
        residual: f64,
        ceiling: f64,
    },
    /// Malformed manifest or corpus row (also covers a Tier-1
    /// self-consistency invariant failing on the recomputed row).
    Parse { row: String },
    /// The backend errored while recomputing a row.
    Backend(String),
}

impl std::fmt::Display for FictitiousError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for FictitiousError {}

/// Summary of the measured maxima and checked-row count for the gate.
#[derive(Debug, Default)]
pub struct FictitiousReport {
    pub rows: usize,
    /// Max ecliptic-longitude residual vs SE (arcseconds), over ALL 19 bodies
    /// (including Nibiru).
    pub max_lon_arcsec: f64,
    /// Max ecliptic-latitude residual vs SE (arcseconds), over ALL 19 bodies.
    pub max_lat_arcsec: f64,
    /// Max radial-distance residual vs SE (AU), over ALL 19 bodies.
    pub max_dist_au: f64,
}

impl FictitiousReport {
    /// The gate passed iff every committed row was checked (a silently
    /// truncated corpus is a failure, not a pass). Every ceiling is enforced
    /// fail-closed by [`validate_fictitious_corpus`], so reaching a report
    /// already implies every checked row was within ceiling.
    pub fn passed(&self) -> bool {
        self.rows == EXPECTED_ROWS
    }

    pub fn summary_line(&self) -> String {
        format!(
            "validate-fictitious: {} rows — max residuals: lon {:.3}\", lat {:.3}\", dist {:.2e} AU",
            self.rows, self.max_lon_arcsec, self.max_lat_arcsec, self.max_dist_au,
        )
    }
}

/// All measured residual maxima over the committed corpus, split into
/// Nibiru / non-Nibiru so each can be gated under its own ceiling (see the
/// module doc). Tier-1 self-consistency is enforced during measurement (it
/// never depends on the numeric ceilings); ceiling gating is applied
/// afterwards by [`validate_fictitious_corpus`].
#[derive(Debug, Default)]
struct Measured {
    rows: usize,
    max_lon_arcsec: f64,
    max_lat_arcsec: f64,
    max_dist_au: f64,
    non_nibiru_max_lon_arcsec: f64,
    non_nibiru_max_lat_arcsec: f64,
    non_nibiru_max_dist_au: f64,
    nibiru_max_lon_arcsec: f64,
    nibiru_max_lat_arcsec: f64,
    nibiru_max_dist_au: f64,
}

impl Measured {
    fn into_report(self) -> FictitiousReport {
        FictitiousReport {
            rows: self.rows,
            max_lon_arcsec: self.max_lon_arcsec,
            max_lat_arcsec: self.max_lat_arcsec,
            max_dist_au: self.max_dist_au,
        }
    }
}

/// SE fictitious-body number (40..=58) -> `CelestialBody`. Matches
/// `pleiades_fict::elements`'s internal label table (SE_FICT_OFFSET_1 (39) +
/// 1..=19); the corpus's `se_body` column is keyed the same way.
fn body_from_se(se_body: i64) -> Option<CelestialBody> {
    match se_body {
        40 => Some(CelestialBody::Cupido),
        41 => Some(CelestialBody::Hades),
        42 => Some(CelestialBody::Zeus),
        43 => Some(CelestialBody::Kronos),
        44 => Some(CelestialBody::Apollon),
        45 => Some(CelestialBody::Admetos),
        46 => Some(CelestialBody::Vulkanus),
        47 => Some(CelestialBody::Poseidon),
        48 => Some(CelestialBody::Transpluto),
        49 => Some(CelestialBody::Nibiru),
        50 => Some(CelestialBody::Harrington),
        51 => Some(CelestialBody::NeptuneLeverrier),
        52 => Some(CelestialBody::NeptuneAdams),
        53 => Some(CelestialBody::PlutoLowell),
        54 => Some(CelestialBody::PlutoPickering),
        55 => Some(CelestialBody::Vulcan),
        56 => Some(CelestialBody::WhiteMoon),
        57 => Some(CelestialBody::Proserpina),
        58 => Some(CelestialBody::Waldemath),
        _ => None,
    }
}

fn wrap180(d: f64) -> f64 {
    ((d + 180.0).rem_euclid(360.0)) - 180.0
}

fn parse_manifest() -> Result<BTreeMap<String, (usize, u64)>, FictitiousError> {
    let mut map = BTreeMap::new();
    for line in MANIFEST.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("file:") else {
            continue;
        };
        let toks: Vec<&str> = rest.split_whitespace().collect();
        if toks.len() < 3 {
            return Err(FictitiousError::Parse {
                row: format!("malformed file line: {line}"),
            });
        }
        let name = toks[0].to_string();
        let mut rows = None;
        let mut checksum = None;
        for tok in &toks[1..] {
            if let Some(v) = tok.strip_prefix("rows=") {
                rows = Some(v.parse::<usize>().map_err(|e| FictitiousError::Parse {
                    row: format!("rows: {e}"),
                })?);
            } else if let Some(v) = tok.strip_prefix("checksum=") {
                checksum = Some(v.parse::<u64>().map_err(|e| FictitiousError::Parse {
                    row: format!("checksum: {e}"),
                })?);
            }
        }
        let rows = rows.ok_or_else(|| FictitiousError::Parse {
            row: format!("rows= missing: {line}"),
        })?;
        let checksum = checksum.ok_or_else(|| FictitiousError::Parse {
            row: format!("checksum= missing: {line}"),
        })?;
        map.insert(name, (rows, checksum));
    }
    if map.is_empty() {
        return Err(FictitiousError::Parse {
            row: "no `file:` lines found in manifest".to_string(),
        });
    }
    Ok(map)
}

/// Looks up `file` in the manifest and compares `fnv1a64(csv)` against the
/// recorded checksum, fail-closed. Returns the manifest's declared row count
/// on success.
fn check_checksum(file: &'static str, csv: &str) -> Result<usize, FictitiousError> {
    let manifest = parse_manifest()?;
    let (rows, want) = *manifest.get(file).ok_or_else(|| FictitiousError::Parse {
        row: format!("manifest missing entry for {file}"),
    })?;
    let got = fnv1a64(csv);
    if got != want {
        return Err(FictitiousError::ChecksumMismatch { file, got, want });
    }
    Ok(rows)
}

fn parse_f64(s: &str, row: &str) -> Result<f64, FictitiousError> {
    s.trim().parse::<f64>().map_err(|_| FictitiousError::Parse {
        row: row.to_string(),
    })
}

fn parse_i64(s: &str, row: &str) -> Result<i64, FictitiousError> {
    s.trim().parse::<i64>().map_err(|_| FictitiousError::Parse {
        row: row.to_string(),
    })
}

/// Runs the checksum guard, parses the corpus, recomputes every row via a
/// freshly built [`FictitiousBackend`], enforces Tier-1 self-consistency, and
/// accumulates every Tier-2 residual maximum (overall, and split Nibiru /
/// non-Nibiru for the per-body carve-out). Numeric ceiling gating is NOT
/// applied here (that is [`validate_fictitious_corpus`]'s job) — so this
/// succeeds regardless of the ceiling constants.
fn measure() -> Result<Measured, FictitiousError> {
    check_checksum(CSV_FILE, CSV)?;

    let backend = FictitiousBackend::new(packaged_backend());
    let mut m = Measured::default();

    for line in CSV.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("label,") {
            continue;
        }
        let f: Vec<&str> = line.split(',').collect();
        if f.len() != 6 {
            return Err(FictitiousError::Parse {
                row: line.to_string(),
            });
        }
        let label = f[0];
        let se_body = parse_i64(f[1], line)?;
        let jd_tt = parse_f64(f[2], line)?;
        let se_lon = parse_f64(f[3], line)?;
        let se_lat = parse_f64(f[4], line)?;
        let se_dist = parse_f64(f[5], line)?;

        let body = body_from_se(se_body).ok_or_else(|| FictitiousError::Parse {
            row: line.to_string(),
        })?;

        let instant = Instant::new(JulianDay::from_days(jd_tt), TimeScale::Tt);
        let req = EphemerisRequest::new(body, instant);
        let result = backend
            .position(&req)
            .map_err(|e| FictitiousError::Backend(format!("{label}: {e}")))?;
        let ecl = result.ecliptic.ok_or_else(|| {
            FictitiousError::Backend(format!("{label}: backend returned no ecliptic position"))
        })?;
        let lon = ecl.longitude.degrees();
        let lat = ecl.latitude.degrees();
        let dist = ecl.distance_au.ok_or_else(|| {
            FictitiousError::Backend(format!("{label}: backend returned no distance"))
        })?;

        // ---- Tier 1: self-consistency (no SE reference) ----
        if !(lon.is_finite() && lat.is_finite() && dist.is_finite()) {
            return Err(FictitiousError::Parse {
                row: format!("{label}: non-finite output lon={lon} lat={lat} dist={dist}"),
            });
        }
        if !(0.0..360.0).contains(&lon) {
            return Err(FictitiousError::Parse {
                row: format!("{label}: longitude out of [0,360): {lon}"),
            });
        }
        if !(-90.0..=90.0).contains(&lat) {
            return Err(FictitiousError::Parse {
                row: format!("{label}: latitude out of [-90,90]: {lat}"),
            });
        }

        // ---- Tier 2: SE parity residuals ----
        let lon_res = wrap180(lon - se_lon).abs() * 3600.0;
        let lat_res = (lat - se_lat).abs() * 3600.0;
        let dist_res = (dist - se_dist).abs();

        m.max_lon_arcsec = m.max_lon_arcsec.max(lon_res);
        m.max_lat_arcsec = m.max_lat_arcsec.max(lat_res);
        m.max_dist_au = m.max_dist_au.max(dist_res);

        if matches!(body_from_se(se_body), Some(CelestialBody::Nibiru)) {
            m.nibiru_max_lon_arcsec = m.nibiru_max_lon_arcsec.max(lon_res);
            m.nibiru_max_lat_arcsec = m.nibiru_max_lat_arcsec.max(lat_res);
            m.nibiru_max_dist_au = m.nibiru_max_dist_au.max(dist_res);
        } else {
            m.non_nibiru_max_lon_arcsec = m.non_nibiru_max_lon_arcsec.max(lon_res);
            m.non_nibiru_max_lat_arcsec = m.non_nibiru_max_lat_arcsec.max(lat_res);
            m.non_nibiru_max_dist_au = m.non_nibiru_max_dist_au.max(dist_res);
        }

        m.rows += 1;
    }

    if m.rows != EXPECTED_ROWS {
        return Err(FictitiousError::RowCountMismatch {
            expected: EXPECTED_ROWS,
            got: m.rows,
        });
    }
    Ok(m)
}

/// Tier-1 (self-consistency) only: checksum guard + per-row recompute +
/// finite/range invariants, with NO Tier-2 ceiling gating. Passes on the
/// committed corpus independently of the threshold constants.
pub fn run_fictitious_tier1_only() -> Result<FictitiousReport, FictitiousError> {
    Ok(measure()?.into_report())
}

/// Full two-tier gate: Tier-1 self-consistency (via [`measure`]) plus Tier-2
/// SE parity gated under the global ceilings in
/// [`crate::fictitious_thresholds`] for the 18 well-behaved bodies, and the
/// separate, wider Nibiru ceilings for the one documented outlier (see the
/// module doc). Fails closed on any exceeded ceiling.
pub fn validate_fictitious_corpus() -> Result<FictitiousReport, FictitiousError> {
    let m = measure()?;

    let checks: [(&'static str, f64, f64); 3] = [
        ("longitude_arcsec", m.non_nibiru_max_lon_arcsec, LONGITUDE_ARCSEC),
        ("latitude_arcsec", m.non_nibiru_max_lat_arcsec, LATITUDE_ARCSEC),
        ("distance_au", m.non_nibiru_max_dist_au, DISTANCE_AU),
    ];
    for (category, residual, ceiling) in checks {
        if !residual.is_finite() || residual > ceiling {
            return Err(FictitiousError::ToleranceExceeded {
                category,
                label: "corpus-max (excl. Nibiru)".to_string(),
                residual,
                ceiling,
            });
        }
    }

    let nibiru_checks: [(&'static str, f64, f64); 3] = [
        (
            "nibiru_longitude_arcsec",
            m.nibiru_max_lon_arcsec,
            NIBIRU_LONGITUDE_ARCSEC,
        ),
        (
            "nibiru_latitude_arcsec",
            m.nibiru_max_lat_arcsec,
            NIBIRU_LATITUDE_ARCSEC,
        ),
        ("nibiru_distance_au", m.nibiru_max_dist_au, NIBIRU_DISTANCE_AU),
    ];
    for (category, residual, ceiling) in nibiru_checks {
        if !residual.is_finite() || residual > ceiling {
            return Err(FictitiousError::ToleranceExceeded {
                category,
                label: "Nibiru".to_string(),
                residual,
                ceiling,
            });
        }
    }

    Ok(m.into_report())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_row_count_is_pinned() {
        let report = run_fictitious_tier1_only().expect("tier1 passes");
        assert_eq!(report.rows, EXPECTED_ROWS);
    }

    #[test]
    fn checksum_drift_fails_closed() {
        assert!(check_checksum("fictitious.csv", "mutated,body\n").is_err());
    }

    #[test]
    fn gate_passes_on_committed_corpus() {
        validate_fictitious_corpus().expect("fictitious gate passes");
    }
}
