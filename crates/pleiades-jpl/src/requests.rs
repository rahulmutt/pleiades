//! Reference-snapshot request-corpus builders for the checked-in fixture.

use pleiades_backend::EphemerisRequest;
use pleiades_types::CoordinateFrame;

use crate::*;

/// Returns the reference-snapshot request corpus in the requested frame.
///
/// The requests preserve the checked-in row order and stored epochs from the
/// derivative CSV. Callers can reuse this corpus for exact batch checks or
/// retag the returned instants with a different time-scale policy if needed.
pub fn reference_snapshot_requests(frame: CoordinateFrame) -> Option<Vec<EphemerisRequest>> {
    snapshot_entries().map(|entries| {
        entries
            .iter()
            .map(|entry| EphemerisRequest {
                body: entry.body.clone(),
                instant: entry.epoch,
                observer: None,
                frame,
                zodiac_mode: ZodiacMode::Tropical,
                apparent: Apparentness::Mean,
            })
            .collect()
    })
}

/// This is a compatibility alias for [`reference_snapshot_requests`].
#[doc(alias = "reference_snapshot_requests")]
pub fn reference_snapshot_request_corpus(frame: CoordinateFrame) -> Option<Vec<EphemerisRequest>> {
    reference_snapshot_requests(frame)
}

/// Returns the ecliptic reference-snapshot request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`reference_snapshot_requests`].
#[doc(alias = "reference_snapshot_requests")]
pub fn reference_snapshot_ecliptic_request_corpus() -> Option<Vec<EphemerisRequest>> {
    reference_snapshot_requests(CoordinateFrame::Ecliptic)
}

/// Returns the ecliptic reference-snapshot request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`reference_snapshot_ecliptic_request_corpus`].
#[doc(alias = "reference_snapshot_ecliptic_request_corpus")]
pub fn reference_snapshot_ecliptic_requests() -> Option<Vec<EphemerisRequest>> {
    reference_snapshot_ecliptic_request_corpus()
}

/// Returns the equatorial reference-snapshot request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`reference_snapshot_requests`].
#[doc(alias = "reference_snapshot_requests")]
pub fn reference_snapshot_equatorial_parity_requests() -> Option<Vec<EphemerisRequest>> {
    reference_snapshot_requests(CoordinateFrame::Equatorial)
}

/// Returns the equatorial reference-snapshot request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`reference_snapshot_equatorial_parity_requests`].
#[doc(alias = "reference_snapshot_equatorial_parity_requests")]
pub fn reference_snapshot_equatorial_batch_parity_requests() -> Option<Vec<EphemerisRequest>> {
    reference_snapshot_equatorial_parity_requests()
}

/// Returns the equatorial reference-snapshot request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`reference_snapshot_equatorial_batch_parity_requests`].
#[doc(alias = "reference_snapshot_equatorial_batch_parity_requests")]
pub fn reference_snapshot_equatorial_batch_parity_request_corpus() -> Option<Vec<EphemerisRequest>>
{
    reference_snapshot_equatorial_batch_parity_requests()
}

/// Returns the equatorial reference-snapshot request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`reference_snapshot_equatorial_request_corpus`].
#[doc(alias = "reference_snapshot_equatorial_request_corpus")]
pub fn reference_snapshot_equatorial_requests() -> Option<Vec<EphemerisRequest>> {
    reference_snapshot_equatorial_request_corpus()
}

/// Returns the equatorial reference-snapshot request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`reference_snapshot_equatorial_parity_requests`].
#[doc(alias = "reference_snapshot_equatorial_parity_requests")]
pub fn reference_snapshot_equatorial_request_corpus() -> Option<Vec<EphemerisRequest>> {
    reference_snapshot_equatorial_parity_requests()
}

/// Returns the equatorial reference-snapshot request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`reference_snapshot_equatorial_parity_requests`].
#[doc(alias = "reference_snapshot_equatorial_parity_requests")]
pub fn reference_snapshot_equatorial_parity_request_corpus() -> Option<Vec<EphemerisRequest>> {
    reference_snapshot_equatorial_parity_requests()
}

/// Returns the mixed-frame reference-snapshot request corpus used by batch parity checks.
///
/// The requests preserve the checked-in row order and alternate between ecliptic
/// and equatorial frames so downstream tooling can reuse the exact release-facing
/// batch shape without reconstructing it from snapshot metadata.
pub fn reference_snapshot_batch_parity_requests() -> Option<Vec<EphemerisRequest>> {
    snapshot_entries().map(|entries| {
        entries
            .iter()
            .enumerate()
            .map(|(index, entry)| EphemerisRequest {
                body: entry.body.clone(),
                instant: entry.epoch,
                observer: None,
                frame: if index % 2 == 0 {
                    CoordinateFrame::Ecliptic
                } else {
                    CoordinateFrame::Equatorial
                },
                zodiac_mode: ZodiacMode::Tropical,
                apparent: Apparentness::Mean,
            })
            .collect()
    })
}

/// Returns the mixed-frame reference-snapshot request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`reference_snapshot_batch_parity_requests`].
#[doc(alias = "reference_snapshot_batch_parity_requests")]
pub fn reference_snapshot_batch_parity_request_corpus() -> Option<Vec<EphemerisRequest>> {
    reference_snapshot_batch_parity_requests()
}

/// Returns the mixed TT/TDB reference-snapshot request corpus used by batch parity checks.
///
/// The requests preserve the checked-in row order, keep the ecliptic frame,
/// and alternate TT/TDB labels so downstream tooling can reuse the exact
/// release-facing batch shape without reconstructing it from snapshot metadata.
pub fn reference_snapshot_mixed_time_scale_batch_parity_requests() -> Option<Vec<EphemerisRequest>>
{
    snapshot_entries().map(|entries| {
        entries
            .iter()
            .enumerate()
            .map(|(index, entry)| EphemerisRequest {
                body: entry.body.clone(),
                instant: Instant::new(
                    entry.epoch.julian_day,
                    if index % 2 == 0 {
                        TimeScale::Tt
                    } else {
                        TimeScale::Tdb
                    },
                ),
                observer: None,
                frame: CoordinateFrame::Ecliptic,
                zodiac_mode: ZodiacMode::Tropical,
                apparent: Apparentness::Mean,
            })
            .collect()
    })
}

/// Returns the mixed TT/TDB reference-snapshot request corpus used by batch parity checks.
///
/// This is a compatibility alias for
/// [`reference_snapshot_mixed_time_scale_batch_parity_requests`].
#[doc(alias = "reference_snapshot_mixed_time_scale_batch_parity_requests")]
pub fn reference_snapshot_mixed_time_scale_batch_parity_request_corpus(
) -> Option<Vec<EphemerisRequest>> {
    reference_snapshot_mixed_time_scale_batch_parity_requests()
}

/// Returns the mixed TT/TDB reference-snapshot request corpus used by batch parity checks.
///
/// This is a compatibility alias for
/// [`reference_snapshot_mixed_time_scale_batch_parity_requests`].
#[doc(alias = "reference_snapshot_mixed_time_scale_batch_parity_requests")]
pub fn reference_snapshot_mixed_tt_tdb_batch_parity_requests() -> Option<Vec<EphemerisRequest>> {
    reference_snapshot_mixed_time_scale_batch_parity_requests()
}

/// Returns the mixed TT/TDB reference-snapshot request corpus used by batch parity checks.
///
/// This is a compatibility alias for
/// [`reference_snapshot_mixed_tt_tdb_batch_parity_requests`].
#[doc(alias = "reference_snapshot_mixed_tt_tdb_batch_parity_requests")]
pub fn reference_snapshot_mixed_tt_tdb_batch_parity_request_corpus() -> Option<Vec<EphemerisRequest>>
{
    reference_snapshot_mixed_tt_tdb_batch_parity_requests()
}

/// Returns the mixed TT/TDB reference-snapshot request corpus used by batch parity checks.
///
/// This is a compatibility alias for
/// [`reference_snapshot_mixed_time_scale_request_corpus`].
#[doc(alias = "reference_snapshot_mixed_time_scale_request_corpus")]
pub fn reference_snapshot_mixed_tt_tdb_request_corpus() -> Option<Vec<EphemerisRequest>> {
    reference_snapshot_mixed_tt_tdb_batch_parity_requests()
}

/// Returns the mixed TT/TDB reference-snapshot request corpus used by batch parity checks.
///
/// This is a compatibility alias for
/// [`reference_snapshot_mixed_tt_tdb_batch_parity_requests`].
#[doc(alias = "reference_snapshot_mixed_tt_tdb_batch_parity_requests")]
pub fn reference_snapshot_mixed_time_scale_request_corpus() -> Option<Vec<EphemerisRequest>> {
    reference_snapshot_mixed_time_scale_batch_parity_requests()
}
