//! Keystone fail-closed gate: the regenerated packaged backend's RAW J2000
//! ecliptic latitude (no chart/apparent pipeline) must match an INDEPENDENT
//! J2000 source at far epochs (1900/2100, where the of-date vs J2000 obliquity
//! gap is ~46″ and so discriminates the frame): Sun + 7 planets via VSOP87 at
//! 1900/2100, plus the Moon via the JPL snapshot fixture at 1900 = 17 rows.
//! Distinguishes "correct J2000" from "self-consistently wrong"; it is
//! non-negotiable and not kernel-gated.
//!
//! # Coverage reconciliation
//!
//! VSOP87 (`Vsop87Backend`) natively covers the Sun + the 7 planets used here
//! (Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune) in the J2000
//! frame. Pluto is EXCLUDED: although `Vsop87Backend` has a Pluto arm, it uses
//! approximate mean orbital elements that diverge from DE440 by ~428″ at 1900 —
//! far above the ~46″ frame-error signal — so it cannot discriminate the frame;
//! the snapshot has no Pluto data at 1900/2100 either.
//!
//! The JPL reference snapshot has an exact Moon data point at JD 2415020.5
//! (1900-01-01 era) but no 2100 Moon data, so the Moon is checked at 1900 only;
//! EPOCH_1900_JD_TT is therefore set to 2415020.5 to hit that exact snapshot
//! row. The 8 VSOP87 bodies × 2 epochs (16 rows) already prove the frame
//! unambiguously; the single Moon row is an additional independent cross-check,
//! for 17 rows total.
#![forbid(unsafe_code)]

use core::fmt;

use pleiades_backend::{EphemerisBackend, EphemerisRequest};
use pleiades_data::PackagedDataBackend;
use pleiades_jpl::JplSnapshotBackend;
use pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale};
use pleiades_vsop87::Vsop87Backend;

// Far epoch matching the snapshot's exact Moon data point (1900-01-01 era).
// Changed from 2415025.5 to 2415020.5 so the Moon check hits an exact snapshot
// row rather than requiring cross-range interpolation over a 36500-day gap.
const EPOCH_1900_JD_TT: f64 = 2_415_020.5;
const EPOCH_2100_JD_TT: f64 = 2_488_065.5;

// Tight ceiling on the packaged-vs-independent J2000 latitude residual. The
// residual is the genuine DE440-vs-(VSOP87/snapshot) model difference once both
// sides are J2000; the ~46″ of-date frame error is far above this. Pinned from
// the measured max_residual_lat_arcsec = 0.06″: ceil(0.06) = 1, +1″ headroom
// → 2.0″, well below the ~46″ frame-error signal.
const FRAME_LAT_TOLERANCE_ARCSEC: f64 = 2.0;

// Sentinel: the Sun's J2000 latitude at 1900 is ~ −46″·sin λ ≈ −45″. Require it
// to exceed this floor in magnitude so a silent revert to the of-date frame
// (Sun lat ≈ 0) fails the gate even if a future reference drift loosens the
// tolerance above.
const SUN_1900_LAT_SENTINEL_ARCSEC: f64 = 30.0;

#[derive(Clone, Debug, PartialEq)]
pub struct FrameConsistencyReport {
    pub rows_validated: usize,
    pub max_residual_lat_arcsec: f64,
    summary_line: String,
}

impl FrameConsistencyReport {
    pub fn summary_line(&self) -> &str {
        &self.summary_line
    }
}

impl fmt::Display for FrameConsistencyReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum FrameConsistencyError {
    PackagedUnavailable { body: String, jd_tt: f64, message: String },
    ReferenceUnavailable { body: String, jd_tt: f64, source: &'static str, message: String },
    MissingEcliptic { body: String, jd_tt: f64, which: &'static str },
    SentinelTooSmall { jd_tt: f64, got_arcsec: f64, floor_arcsec: f64 },
    ToleranceExceeded {
        body: String,
        jd_tt: f64,
        packaged_lat_deg: f64,
        reference_lat_deg: f64,
        residual_arcsec: f64,
        tolerance_arcsec: f64,
    },
}

impl fmt::Display for FrameConsistencyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PackagedUnavailable { body, jd_tt, message } => write!(
                f,
                "packaged backend unavailable for {body} @ JD {jd_tt}: {message}"
            ),
            Self::ReferenceUnavailable { body, jd_tt, source, message } => write!(
                f,
                "independent reference ({source}) unavailable for {body} @ JD {jd_tt}: {message}"
            ),
            Self::MissingEcliptic { body, jd_tt, which } => {
                write!(f, "{which} ecliptic latitude absent for {body} @ JD {jd_tt}")
            }
            Self::SentinelTooSmall { jd_tt, got_arcsec, floor_arcsec } => write!(
                f,
                "frame sentinel failed: Sun J2000 latitude @ JD {jd_tt} is {got_arcsec:.2}\u{2033} \
                 (|·| must exceed {floor_arcsec:.1}\u{2033}); the backend looks of-date, not J2000"
            ),
            Self::ToleranceExceeded {
                body,
                jd_tt,
                packaged_lat_deg,
                reference_lat_deg,
                residual_arcsec,
                tolerance_arcsec,
            } => write!(
                f,
                "frame latitude mismatch for {body} @ JD {jd_tt}: packaged {packaged_lat_deg:.7}\u{00b0} \
                 vs independent J2000 {reference_lat_deg:.7}\u{00b0}, residual {residual_arcsec:.2}\u{2033} \
                 > tol {tolerance_arcsec:.1}\u{2033}"
            ),
        }
    }
}

impl std::error::Error for FrameConsistencyError {}

#[derive(Clone, Copy)]
enum Reference {
    Vsop87,
    Snapshot,
}

/// Failure modes of a single raw-latitude read, kept distinct so callers can
/// map an absent ecliptic channel to the precise `MissingEcliptic` variant
/// rather than folding it into a generic backend-error string.
enum RawLatError {
    /// The backend's `position` call itself failed.
    Backend(String),
    /// The position succeeded but carried no ecliptic channel.
    MissingEcliptic,
}

fn raw_lat_deg(
    backend: &dyn EphemerisBackend,
    body: &CelestialBody,
    instant: Instant,
) -> Result<f64, RawLatError> {
    let result = backend
        .position(&EphemerisRequest::new(body.clone(), instant))
        .map_err(|e| RawLatError::Backend(e.to_string()))?;
    result
        .ecliptic
        .map(|ec| ec.latitude.degrees())
        .ok_or(RawLatError::MissingEcliptic)
}

/// The Sun-1900 frame sentinel, extracted so the gate and its discriminator
/// test exercise the same code path: the Sun's J2000 ecliptic latitude at the
/// 1900 far epoch must be genuinely non-zero (|·| > floor ≈ −45″), so a silent
/// revert to the of-date frame (Sun lat ≈ 0) trips `SentinelTooSmall`.
fn check_sun_1900_sentinel(lat_deg: f64) -> Result<(), FrameConsistencyError> {
    let got_arcsec = lat_deg * 3600.0;
    if got_arcsec.abs() < SUN_1900_LAT_SENTINEL_ARCSEC {
        return Err(FrameConsistencyError::SentinelTooSmall {
            jd_tt: EPOCH_1900_JD_TT,
            got_arcsec,
            floor_arcsec: SUN_1900_LAT_SENTINEL_ARCSEC,
        });
    }
    Ok(())
}

pub fn validate_frame_consistency() -> Result<FrameConsistencyReport, FrameConsistencyError> {
    let packaged = PackagedDataBackend::new();
    let vsop = Vsop87Backend::new();
    let snapshot = JplSnapshotBackend::new();

    // Flat list of (body, reference, epoch_jd_tt) — one row per check.
    //
    // VSOP87 references the Sun + 7 planets (Mercury, Venus, Mars, Jupiter,
    // Saturn, Uranus, Neptune). Pluto is excluded: although Vsop87Backend has a
    // Pluto arm (approximate mean orbital elements), it diverges from DE440 by
    // ~428″ at 1900 — far above the ~46″ frame-error signal — so it cannot serve
    // as a discriminating reference. The snapshot has no Pluto data at 1900 or
    // 2100. Sun + 7 planets + Moon already prove the frame unambiguously; Pluto
    // is dropped per the reconciliation note.
    //
    // The JPL snapshot has an exact Moon data point at JD 2415020.5 (= EPOCH_1900).
    // The snapshot has no 2100 Moon data, so Moon is excluded at 2100.
    //
    // Total: (Sun + 7 planets) × 2 epochs + Moon × 1 epoch = 8 × 2 + 1 = 17 rows.
    let checks: &[(CelestialBody, Reference, f64)] = &[
        // 1900 epoch — VSOP87 bodies
        (CelestialBody::Sun,     Reference::Vsop87,   EPOCH_1900_JD_TT),
        (CelestialBody::Mercury, Reference::Vsop87,   EPOCH_1900_JD_TT),
        (CelestialBody::Venus,   Reference::Vsop87,   EPOCH_1900_JD_TT),
        (CelestialBody::Mars,    Reference::Vsop87,   EPOCH_1900_JD_TT),
        (CelestialBody::Jupiter, Reference::Vsop87,   EPOCH_1900_JD_TT),
        (CelestialBody::Saturn,  Reference::Vsop87,   EPOCH_1900_JD_TT),
        (CelestialBody::Uranus,  Reference::Vsop87,   EPOCH_1900_JD_TT),
        (CelestialBody::Neptune, Reference::Vsop87,   EPOCH_1900_JD_TT),
        // 1900 epoch — Moon via snapshot (exact row at JD 2415020.5)
        (CelestialBody::Moon,    Reference::Snapshot, EPOCH_1900_JD_TT),
        // 2100 epoch — VSOP87 bodies (Moon excluded: no snapshot 2100 data)
        (CelestialBody::Sun,     Reference::Vsop87,   EPOCH_2100_JD_TT),
        (CelestialBody::Mercury, Reference::Vsop87,   EPOCH_2100_JD_TT),
        (CelestialBody::Venus,   Reference::Vsop87,   EPOCH_2100_JD_TT),
        (CelestialBody::Mars,    Reference::Vsop87,   EPOCH_2100_JD_TT),
        (CelestialBody::Jupiter, Reference::Vsop87,   EPOCH_2100_JD_TT),
        (CelestialBody::Saturn,  Reference::Vsop87,   EPOCH_2100_JD_TT),
        (CelestialBody::Uranus,  Reference::Vsop87,   EPOCH_2100_JD_TT),
        (CelestialBody::Neptune, Reference::Vsop87,   EPOCH_2100_JD_TT),
    ];

    let mut rows_validated = 0usize;
    let mut max_residual_lat_arcsec = 0.0_f64;

    for (body, reference, jd_tt) in checks {
        let jd_tt = *jd_tt;
        let instant = Instant::new(JulianDay::from_days(jd_tt), TimeScale::Tt);
        let body_label = format!("{body:?}");

        let packaged_lat =
            raw_lat_deg(&packaged, body, instant).map_err(|err| match err {
                RawLatError::Backend(message) => FrameConsistencyError::PackagedUnavailable {
                    body: body_label.clone(),
                    jd_tt,
                    message,
                },
                RawLatError::MissingEcliptic => FrameConsistencyError::MissingEcliptic {
                    body: body_label.clone(),
                    jd_tt,
                    which: "packaged",
                },
            })?;

        // Sentinel: the Sun's J2000 latitude at 1900 must be genuinely non-zero
        // (~−45″). A silent revert to of-date obliquity would make it ~0 and fail.
        if *body == CelestialBody::Sun && jd_tt == EPOCH_1900_JD_TT {
            check_sun_1900_sentinel(packaged_lat)?;
        }

        let (source_name, reference_backend): (&'static str, &dyn EphemerisBackend) = match reference
        {
            Reference::Vsop87 => ("vsop87", &vsop),
            Reference::Snapshot => ("jpl-snapshot", &snapshot),
        };
        let reference_lat =
            raw_lat_deg(reference_backend, body, instant).map_err(|err| match err {
                RawLatError::Backend(message) => FrameConsistencyError::ReferenceUnavailable {
                    body: body_label.clone(),
                    jd_tt,
                    source: source_name,
                    message,
                },
                RawLatError::MissingEcliptic => FrameConsistencyError::MissingEcliptic {
                    body: body_label.clone(),
                    jd_tt,
                    which: source_name,
                },
            })?;

        let residual = ((packaged_lat - reference_lat) * 3600.0).abs();
        if residual > FRAME_LAT_TOLERANCE_ARCSEC {
            return Err(FrameConsistencyError::ToleranceExceeded {
                body: body_label,
                jd_tt,
                packaged_lat_deg: packaged_lat,
                reference_lat_deg: reference_lat,
                residual_arcsec: residual,
                tolerance_arcsec: FRAME_LAT_TOLERANCE_ARCSEC,
            });
        }
        max_residual_lat_arcsec = max_residual_lat_arcsec.max(residual);
        rows_validated += 1;
    }

    let summary_line = format!(
        "Frame-consistency gate: {rows_validated} rows validated \
         (packaged raw J2000 latitude vs VSOP87/snapshot at 1900/2100), \
         max lat residual {max_residual_lat_arcsec:.2}\u{2033}"
    );
    Ok(FrameConsistencyReport { rows_validated, max_residual_lat_arcsec, summary_line })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_consistency_gate_passes() {
        let report = validate_frame_consistency().expect("frame-consistency gate passes");
        // Exact expected count: 8 VSOP87 bodies x 2 epochs + Moon x 1 epoch = 17.
        // A positive, non-trivial latitude is already proven inside the gate loop by
        // the Sun@1900 sentinel (this file, ~lines 236-240: |ecliptic latitude| ~= 45"),
        // so it is not re-asserted here.
        assert_eq!(
            report.rows_validated, 17,
            "unexpected row count: {}",
            report.rows_validated
        );
        // Print the measured maximum so FRAME_LAT_TOLERANCE_ARCSEC can be tightened.
        eprintln!("{}", report.summary_line());
    }

    /// Step 4 discriminator proof (manual red/green check).
    ///
    /// `PackagedDataBackend::position` reads pre-stored ecliptic values from the
    /// committed artifact binary — it does NOT call `icrf_to_ecliptic` at query
    /// time. Therefore reverting `icrf_to_ecliptic` to of-date obliquity in
    /// `chain.rs` has no effect on this backend. Instead, the discriminator proof
    /// is verified directly here:
    ///
    /// 1. GREEN: the Sun's packaged J2000 latitude at JD 2415020.5 (1900) is
    ///    ~−45″; driven through the real `check_sun_1900_sentinel` it returns
    ///    `Ok(())`, proving the artifact is in J2000.
    /// 2. RED: an of-date-like latitude (~1.5″) driven through the SAME real
    ///    `check_sun_1900_sentinel` returns `Err(SentinelTooSmall)`, exercising
    ///    the genuine gate path (not a hand-built error literal).
    /// 3. The pinned tolerance (FRAME_LAT_TOLERANCE_ARCSEC = 2.0″) is far below
    ///    the ~46″ frame-error signal, so a pre-B2 of-date artifact would also
    ///    trip `ToleranceExceeded`.
    #[test]
    fn discriminator_proof_sun_1900_sentinel_is_real() {
        let packaged = PackagedDataBackend::new();
        let instant_1900 = Instant::new(JulianDay::from_days(EPOCH_1900_JD_TT), TimeScale::Tt);

        let sun_lat_deg =
            raw_lat_deg(&packaged, &CelestialBody::Sun, instant_1900)
                .unwrap_or_else(|_| panic!("packaged Sun at 1900 should succeed"));
        let sun_lat_arcsec = sun_lat_deg * 3600.0;

        // GREEN: drive the REAL artifact's Sun-1900 latitude through the ACTUAL
        // sentinel-check code path; with the J2000 artifact it must return Ok.
        eprintln!(
            "Sun J2000 lat at JD {EPOCH_1900_JD_TT}: {sun_lat_arcsec:.2}\u{2033} \
             (must be |·| > {SUN_1900_LAT_SENTINEL_ARCSEC:.1}\u{2033})"
        );
        assert_eq!(
            check_sun_1900_sentinel(sun_lat_deg),
            Ok(()),
            "J2000 artifact Sun lat {sun_lat_arcsec:.2}\u{2033} should pass the sentinel"
        );

        // RED: drive an of-date-like latitude (~1.5″ — an of-date artifact stores
        // the Sun near lat=0 since the of-date ecliptic plane contains the Sun)
        // through the SAME real sentinel-check path and assert it returns
        // Err(SentinelTooSmall). This exercises the genuine gate logic, not a
        // hand-built error literal.
        let of_date_lat_deg = 1.5 / 3600.0; // ~1.5″ in degrees
        let red = check_sun_1900_sentinel(of_date_lat_deg);
        assert!(
            matches!(red, Err(FrameConsistencyError::SentinelTooSmall { .. })),
            "of-date Sun lat (~1.5\u{2033}) must trip the real sentinel, got {red:?}"
        );
        eprintln!(
            "Discriminator RED: of-date lat ~1.5\u{2033} through real sentinel \u{2192} \
             {red:?} (gate fails SentinelTooSmall)"
        );
        eprintln!(
            "Discriminator RED: residual vs VSOP87 would be ~45\u{2033} > tol \
             {FRAME_LAT_TOLERANCE_ARCSEC:.1}\u{2033} — gate would also fail ToleranceExceeded"
        );
    }
}
