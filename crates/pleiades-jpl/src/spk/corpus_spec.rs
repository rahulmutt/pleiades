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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_spans_target_window() {
        assert!(RANGE_START_JD < RANGE_END_JD);
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
