//! Per-body fitting cadence model for dense de440-backed artifact generation.
//!
//! Spans are accuracy-safe initial defaults (SP1); SP2 tunes them against the
//! measured accuracy baseline. Within-span sampling oversamples the polynomial
//! degree so each segment's least-squares fit is over-determined.

use pleiades_backend::CelestialBody;

/// Oversample factor: within-span sample count = (degree + 1) * this.
pub const FITTING_OVERSAMPLE: usize = 3;

/// Per-body segment span in days (initial SP1 defaults; tuned in SP2).
pub fn fitting_segment_span_days(body: &CelestialBody) -> f64 {
    match body {
        CelestialBody::Moon => 4.0,
        CelestialBody::Mercury => 8.0,
        CelestialBody::Venus | CelestialBody::Sun => 16.0,
        CelestialBody::Mars => 32.0,
        CelestialBody::Jupiter => 128.0,
        CelestialBody::Saturn => 256.0,
        CelestialBody::Uranus | CelestialBody::Neptune | CelestialBody::Pluto => 512.0,
        // Constrained asteroids (e.g. Eros) use a Mars-like span; only generated
        // within their own corpus window by the caller.
        _ => 16.0,
    }
}

/// Per-body polynomial degree for the within-span fit (SP1 default).
pub fn fitting_degree(_body: &CelestialBody) -> usize {
    8
}

/// Number of de440 samples taken within each segment span.
pub fn fitting_within_span_sample_count(body: &CelestialBody) -> usize {
    (fitting_degree(body) + 1) * FITTING_OVERSAMPLE
}

/// Contiguous `[t0, t1]` spans tiling `[start_jd, end_jd]`, last clamped to `end_jd`.
pub fn fitting_segment_boundaries(
    body: &CelestialBody,
    start_jd: f64,
    end_jd: f64,
) -> Vec<(f64, f64)> {
    let span = fitting_segment_span_days(body);
    let mut spans = Vec::new();
    let mut t0 = start_jd;
    while t0 < end_jd {
        let t1 = (t0 + span).min(end_jd);
        if t1 > t0 {
            spans.push((t0, t1));
        }
        t0 = t1;
    }
    spans
}

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_backend::CelestialBody;

    #[test]
    fn spans_match_documented_defaults() {
        assert_eq!(fitting_segment_span_days(&CelestialBody::Moon), 4.0);
        assert_eq!(fitting_segment_span_days(&CelestialBody::Mercury), 8.0);
        assert_eq!(fitting_segment_span_days(&CelestialBody::Venus), 16.0);
        assert_eq!(fitting_segment_span_days(&CelestialBody::Sun), 16.0);
        assert_eq!(fitting_segment_span_days(&CelestialBody::Mars), 32.0);
        assert_eq!(fitting_segment_span_days(&CelestialBody::Jupiter), 128.0);
        assert_eq!(fitting_segment_span_days(&CelestialBody::Saturn), 256.0);
        assert_eq!(fitting_segment_span_days(&CelestialBody::Uranus), 512.0);
        assert_eq!(fitting_segment_span_days(&CelestialBody::Neptune), 512.0);
        assert_eq!(fitting_segment_span_days(&CelestialBody::Pluto), 512.0);
    }

    #[test]
    fn within_span_sample_count_oversamples_degree() {
        let n = fitting_within_span_sample_count(&CelestialBody::Moon);
        assert_eq!(
            n,
            (fitting_degree(&CelestialBody::Moon) + 1) * FITTING_OVERSAMPLE
        );
        assert!(
            n > fitting_degree(&CelestialBody::Moon) + 1,
            "must oversample"
        );
    }

    #[test]
    fn boundaries_tile_the_window_without_gaps_or_overlap() {
        let spans = fitting_segment_boundaries(&CelestialBody::Jupiter, 1000.0, 1500.0);
        assert_eq!(spans.first().unwrap().0, 1000.0);
        assert_eq!(spans.last().unwrap().1, 1500.0);
        for pair in spans.windows(2) {
            assert_eq!(pair[0].1, pair[1].0, "spans must be contiguous");
        }
        for (t0, t1) in &spans {
            assert!(t1 > t0, "each span is non-empty");
        }
    }
}
