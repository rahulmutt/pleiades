//! The interpolation stencil of the snapshot backend and the guard that
//! refuses an interpolation its rows cannot support (issues #158, #200).

use pleiades_backend::{EphemerisError, EphemerisErrorKind};

#[cfg(doc)]
use super::JplSnapshotBackend;
use super::SnapshotEntry;

/// Widest stencil, in days, that [`JplSnapshotBackend`] interpolates across.
///
/// The backend answers between rows only where the rows it would use lie on
/// both sides of the instant and span no more than this. Holding each row out
/// in turn, every stencil this admits reproduces the held-out asteroid row
/// within 0.05″ (`stencils_the_guard_admits_reproduce_held_out_rows`). The
/// next wider bracket in the snapshot is a year, where the same cubic is
/// wrong by tens of degrees (issue #158).
///
/// The 0.05″ figure is measured for Ceres, Pallas, Juno, Vesta and Eros.
/// Apophis's interpolation inside the cluster is served under the same rule,
/// but its cluster rows are too few to hold one out, so that measurement does
/// not cover it.
///
/// The same rule serves the Sun and the planets, whose held-out rows are
/// reproduced within 0.3″ (the worst is Mercury, 0.20″;
/// `stencils_the_guard_admits_reproduce_held_out_major_body_rows`). The Moon
/// is not interpolated at all: a cubic through rows a day apart misplaces it
/// by up to 80″, so it is served only at a row (issue #200).
pub const MAX_STENCIL_SPAN_DAYS: f64 = 5.0;

/// The rows an interpolation at `epoch_jd` uses for `body`, ascending: the
/// four nearest in time, or all three when the body has exactly three. Empty
/// when the body has fewer than three rows.
pub(crate) fn interpolation_stencil<'a>(
    entries: &'a [SnapshotEntry],
    body: &pleiades_backend::CelestialBody,
    epoch_jd: f64,
) -> Vec<&'a SnapshotEntry> {
    let epoch_of = |entry: &SnapshotEntry| entry.epoch.julian_day.days();
    let mut ranked = entries
        .iter()
        .filter(|entry| &entry.body == body)
        .map(|entry| ((epoch_of(entry) - epoch_jd).abs(), entry))
        .collect::<Vec<_>>();
    if ranked.len() < 3 {
        return Vec::new();
    }
    ranked.sort_by(|left, right| {
        left.0
            .partial_cmp(&right.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                epoch_of(left.1)
                    .partial_cmp(&epoch_of(right.1))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });
    let window_size = if ranked.len() >= 4 { 4 } else { 3 };
    let mut stencil = ranked
        .into_iter()
        .take(window_size)
        .map(|(_, entry)| entry)
        .collect::<Vec<_>>();
    stencil.sort_by(|left, right| {
        epoch_of(left)
            .partial_cmp(&epoch_of(right))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    stencil
}

pub(crate) fn interpolate_fixture_state(
    entries: &[SnapshotEntry],
    body: pleiades_backend::CelestialBody,
    epoch_jd: f64,
) -> Option<SnapshotEntry> {
    match *interpolation_stencil(entries, &body, epoch_jd).as_slice() {
        [a, b, c, d] => Some(SnapshotEntry::interpolate_cubic(a, b, c, d, epoch_jd)),
        [a, b, c] => Some(SnapshotEntry::interpolate_quadratic(a, b, c, epoch_jd)),
        _ => None,
    }
}

/// First and last epoch of the rows an interpolation at `epoch_jd` rests on:
/// the stencil, or the two adjacent rows when the body has fewer than three.
pub(crate) fn stencil_bounds(
    entries: &[SnapshotEntry],
    body: &pleiades_backend::CelestialBody,
    epoch_jd: f64,
) -> Option<(f64, f64)> {
    let stencil = interpolation_stencil(entries, body, epoch_jd);
    if let (Some(first), Some(last)) = (stencil.first(), stencil.last()) {
        return Some((first.epoch.julian_day.days(), last.epoch.julian_day.days()));
    }
    let (before, after) = adjacent_epochs(entries, body, epoch_jd);
    Some((before?, after?))
}

/// The body's nearest row epoch before `epoch_jd` and nearest after it.
fn adjacent_epochs(
    entries: &[SnapshotEntry],
    body: &pleiades_backend::CelestialBody,
    epoch_jd: f64,
) -> (Option<f64>, Option<f64>) {
    let epochs = || {
        entries
            .iter()
            .filter(|entry| &entry.body == body)
            .map(|entry| entry.epoch.julian_day.days())
    };
    (
        epochs().filter(|jd| *jd < epoch_jd).reduce(f64::max),
        epochs().filter(|jd| *jd > epoch_jd).reduce(f64::min),
    )
}

/// Whether the rows nearest `epoch_jd` support an interpolation there: they
/// lie on both sides of it and span at most [`MAX_STENCIL_SPAN_DAYS`].
pub(crate) fn stencil_supports(
    entries: &[SnapshotEntry],
    body: &pleiades_backend::CelestialBody,
    epoch_jd: f64,
) -> bool {
    stencil_bounds(entries, body, epoch_jd).is_some_and(|(first, last)| {
        first < epoch_jd && epoch_jd < last && last - first <= MAX_STENCIL_SPAN_DAYS
    })
}

/// Whether [`JplSnapshotBackend`] interpolates `body` between its rows.
///
/// The Moon is served only at a row. Holding each Moon row out in turn, a
/// cubic through the rest misses it by 0.19″ where the rows are a quarter of
/// a day apart and by up to 80″ where they are a day apart
/// (`stencils_the_guard_admits_reproduce_held_out_major_body_rows`), against
/// 0.3″ for the Sun and the planets under the same rule (issue #200).
fn is_interpolated(body: &pleiades_backend::CelestialBody) -> bool {
    *body != pleiades_backend::CelestialBody::Moon
}

/// Refuses an interpolation the rows cannot support (issue #158).
pub(crate) fn require_supported_stencil(
    entries: &[SnapshotEntry],
    body: &pleiades_backend::CelestialBody,
    epoch_jd: f64,
) -> Result<(), EphemerisError> {
    if is_interpolated(body) && stencil_supports(entries, body, epoch_jd) {
        return Ok(());
    }
    let describe = |epoch: Option<f64>| match epoch {
        Some(jd) => format!("JD {jd}"),
        None => "none".to_string(),
    };
    let (before, after) = adjacent_epochs(entries, body, epoch_jd);
    let reason = if !is_interpolated(body) {
        format!(
            "it serves {body} only at a fixture row, because a cubic through rows a day apart \
             misplaces it by up to 80″"
        )
    } else if before.is_some() && after.is_some() {
        format!(
            "it interpolates only between rows on both sides of an instant that span at most \
             {MAX_STENCIL_SPAN_DAYS} days"
        )
    } else {
        "the requested instant is outside adjacent JPL fixture samples for that body".to_string()
    };
    Err(EphemerisError::new(
        EphemerisErrorKind::OutOfRangeInstant,
        format!(
            "the JPL snapshot cannot interpolate {body} at JD {epoch_jd} (nearest row before: {}; nearest after: {}); {reason}. Serve \
             {body} at this instant from pleiades_jpl::SpkBackend with a JPL kernel",
            describe(before),
            describe(after)
        ),
    ))
}
