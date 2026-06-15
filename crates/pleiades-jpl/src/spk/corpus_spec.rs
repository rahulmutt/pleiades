//! Single source of truth for the production reference corpus: slice roles,
//! per-body cadence, the deterministic epoch grid, release/constrained body
//! sets, the completeness matrix, and cross-check tolerances. Both the
//! generator and the validation gate read this module so coverage cannot drift.

use pleiades_backend::CelestialBody;

/// Target packaged range, as TDB Julian Days (1600-01-01 .. 2600-01-01).
pub const RANGE_START_JD: f64 = 2_305_447.5;
pub const RANGE_END_JD: f64 = 2_670_690.5;

/// Pinned identity of the reference kernel. SHA-256 is computed externally via
/// `shasum -a 256 de440.bsp` and recorded here + in docs/spk-kernel-sourcing.md.
pub const KERNEL_LABEL: &str = "JPL DE SPK kernel: de440.bsp";
pub const KERNEL_SHA256: &str = "<pinned-after-download>";

/// Role of a corpus slice, preserving the reference/holdout/boundary/
/// fixture-exactness separation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SliceRole {
    Boundary,
    InteriorBackbone,
    FastCluster,
    Holdout,
    FixtureGolden,
}

impl SliceRole {
    /// The token written to the `#Slice-Role:` header and the manifest.
    pub fn token(self) -> &'static str {
        match self {
            SliceRole::Boundary => "boundary",
            SliceRole::InteriorBackbone => "interior",
            SliceRole::FastCluster => "fast_cluster",
            SliceRole::Holdout => "holdout",
            SliceRole::FixtureGolden => "fixture_golden",
        }
    }
}

/// Bodies that must be fully covered by the completeness matrix.
pub fn release_bodies() -> Vec<CelestialBody> {
    vec![
        CelestialBody::Sun,
        CelestialBody::Moon,
        CelestialBody::Mercury,
        CelestialBody::Venus,
        CelestialBody::Mars,
        CelestialBody::Jupiter,
        CelestialBody::Saturn,
        CelestialBody::Uranus,
        CelestialBody::Neptune,
    ]
}

/// Bodies carried but tagged constrained/approximate and excluded from
/// release-grade tolerance evidence (Pluto + selected asteroids).
pub fn constrained_bodies() -> Vec<CelestialBody> {
    vec![CelestialBody::Pluto]
}

/// Maximum allowed epoch gap (TDB days) for a body in the interior backbone,
/// scaled to body speed: fast bodies sampled densely, slow bodies sparsely.
pub fn max_gap_days(body: &CelestialBody) -> f64 {
    match body {
        CelestialBody::Moon => 30.0,
        CelestialBody::Mercury => 60.0,
        CelestialBody::Venus => 120.0,
        CelestialBody::Sun => 180.0,
        CelestialBody::Mars => 365.0,
        CelestialBody::Jupiter => 1_825.0, // ~5 yr
        CelestialBody::Saturn => 3_650.0,  // ~10 yr
        CelestialBody::Uranus | CelestialBody::Neptune => 7_300.0, // ~20 yr
        CelestialBody::Pluto => 7_300.0,
        _ => 365.0,
    }
}

/// Deterministic, strictly-increasing interior backbone epochs for a body:
/// from RANGE_START_JD to RANGE_END_JD inclusive, stepping by `max_gap_days`.
pub fn interior_backbone_epochs(body: &CelestialBody) -> Vec<f64> {
    let step = max_gap_days(body);
    let mut epochs = Vec::new();
    let mut jd = RANGE_START_JD;
    while jd < RANGE_END_JD {
        epochs.push(jd);
        jd += step;
    }
    if epochs
        .last()
        .is_none_or(|&last| (RANGE_END_JD - last).abs() > 1e-6)
    {
        epochs.push(RANGE_END_JD);
    }
    epochs
}

/// Guard epochs just inside and just outside each end of the target range.
pub fn boundary_epochs() -> Vec<f64> {
    vec![
        RANGE_START_JD - 365.0,
        RANGE_START_JD,
        RANGE_START_JD + 365.0,
        RANGE_END_JD - 365.0,
        RANGE_END_JD,
        RANGE_END_JD + 365.0,
    ]
}

/// Fine-cadence windows for fast bodies. Each anchor expands to daily samples
/// across `window_days`. Anchors are spread across the range.
pub fn fast_cluster_epochs() -> Vec<f64> {
    let anchors = [
        RANGE_START_JD + 5_000.0,
        (RANGE_START_JD + RANGE_END_JD) / 2.0,
        RANGE_END_JD - 5_000.0,
    ];
    let window_days = 30;
    let mut epochs = Vec::new();
    for anchor in anchors {
        for day in 0..window_days {
            epochs.push(anchor + day as f64);
        }
    }
    epochs
}

/// Deterministic pseudo-random hold-out epochs (seeded LCG), disjoint from the
/// interior backbone of every body. Used for unbiased artifact error.
pub fn holdout_epochs(count: usize) -> Vec<f64> {
    // Numerical Recipes LCG constants; deterministic across runs.
    let mut state: u64 = 0x5DEE_CE66_D000_1234;
    let mut epochs = Vec::with_capacity(count);
    let span = RANGE_END_JD - RANGE_START_JD;
    while epochs.len() < count {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let frac = (state >> 11) as f64 / (1u64 << 53) as f64;
        let jd = RANGE_START_JD + frac * span;
        // Keep hold-out off the coarsest backbone grid lines.
        let on_grid = release_bodies()
            .iter()
            .any(|b| ((jd - RANGE_START_JD) % max_gap_days(b)).abs() < 0.5);
        if !on_grid {
            epochs.push(jd);
        }
    }
    epochs
}

#[cfg(test)]
mod epoch_tests {
    use super::*;

    #[test]
    fn boundary_brackets_both_ends() {
        let e = boundary_epochs();
        assert!(e.iter().any(|&j| j < RANGE_START_JD));
        assert!(e.iter().any(|&j| j > RANGE_END_JD));
    }

    #[test]
    fn fast_clusters_are_daily() {
        let e = fast_cluster_epochs();
        assert_eq!(e.len(), 90); // 3 anchors x 30 days
        assert!((e[1] - e[0] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn holdout_is_deterministic_and_in_range() {
        let a = holdout_epochs(50);
        let b = holdout_epochs(50);
        assert_eq!(a, b, "hold-out must be reproducible");
        assert_eq!(a.len(), 50);
        for jd in a {
            assert!(jd > RANGE_START_JD && jd < RANGE_END_JD);
        }
    }
}

#[cfg(test)]
mod backbone_tests {
    use super::*;

    #[test]
    fn backbone_is_within_range_and_increasing() {
        let epochs = interior_backbone_epochs(&CelestialBody::Mars);
        assert!(epochs.len() >= 2);
        assert_eq!(*epochs.first().unwrap(), RANGE_START_JD);
        assert_eq!(*epochs.last().unwrap(), RANGE_END_JD);
        for pair in epochs.windows(2) {
            assert!(pair[1] > pair[0], "epochs must strictly increase");
        }
    }

    #[test]
    fn backbone_respects_max_gap() {
        for body in release_bodies() {
            let gap = max_gap_days(&body);
            let epochs = interior_backbone_epochs(&body);
            for pair in epochs.windows(2) {
                assert!(
                    pair[1] - pair[0] <= gap + 1e-6,
                    "gap exceeds cadence for {body:?}"
                );
            }
        }
    }

    #[test]
    fn faster_bodies_have_more_samples() {
        let moon = interior_backbone_epochs(&CelestialBody::Moon).len();
        let neptune = interior_backbone_epochs(&CelestialBody::Neptune).len();
        assert!(moon > neptune);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_spans_target_window() {
        const { assert!(RANGE_START_JD < RANGE_END_JD) };
        // 1600-01-01 .. 2600-01-01 spans 365_243 days.
        assert!((RANGE_END_JD - RANGE_START_JD - 365_243.0).abs() < 2.0);
    }

    #[test]
    fn release_and_constrained_bodies_are_disjoint() {
        for body in release_bodies() {
            assert!(!constrained_bodies().contains(&body));
        }
    }

    #[test]
    fn slice_role_tokens_are_unique() {
        let roles = [
            SliceRole::Boundary,
            SliceRole::InteriorBackbone,
            SliceRole::FastCluster,
            SliceRole::Holdout,
            SliceRole::FixtureGolden,
        ];
        let mut tokens: Vec<&str> = roles.iter().map(|r| r.token()).collect();
        tokens.sort_unstable();
        tokens.dedup();
        assert_eq!(tokens.len(), roles.len());
    }
}
