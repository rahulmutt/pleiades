//! Typed accessors over the committed production reference corpus.
//!
//! The CSV slices live under `crates/pleiades-jpl/data/corpus/` and share the
//! `epoch_jd,body,x_km,y_km,z_km` schema. These accessors parse them once into
//! `SnapshotEntry` values so both the artifact generator (`pleiades-data`) and
//! the `validate-corpus` gate (`pleiades-validate`) consume one source.

use std::sync::OnceLock;

use pleiades_backend::CelestialBody;

use crate::backend::{parse_snapshot_entries, SnapshotEntry};

const INTERIOR_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/corpus/interior.csv"
));
const BOUNDARY_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/corpus/boundary.csv"
));
const FAST_CLUSTERS_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/corpus/fast_clusters.csv"
));
const HOLDOUT_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/corpus/holdout.csv"
));
const FIXTURE_GOLDEN_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/corpus/fixture_golden.csv"
));
const ASTEROID_REFERENCE_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/corpus/asteroid_reference.csv"
));
const ASTEROID_CONSTRAINED_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/corpus/asteroid_constrained.csv"
));

fn parse_or_panic(label: &str, source: &str) -> Vec<SnapshotEntry> {
    parse_snapshot_entries(source)
        .unwrap_or_else(|error| panic!("committed corpus slice `{label}` failed to parse: {error}"))
}

/// Base-body fitting rows: interior ∪ boundary ∪ fast_clusters.
pub fn production_reference_corpus() -> &'static [SnapshotEntry] {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    ENTRIES.get_or_init(|| {
        let mut entries = parse_or_panic("interior", INTERIOR_CSV);
        entries.extend(parse_or_panic("boundary", BOUNDARY_CSV));
        entries.extend(parse_or_panic("fast_clusters", FAST_CLUSTERS_CSV));
        entries
    })
}

/// Independent hold-out rows (excluded from fitting).
pub fn production_holdout_corpus() -> &'static [SnapshotEntry] {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    ENTRIES
        .get_or_init(|| parse_or_panic("holdout", HOLDOUT_CSV))
        .as_slice()
}

/// Fixture-exactness cross-check rows.
pub fn fixture_golden_corpus() -> &'static [SnapshotEntry] {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    ENTRIES
        .get_or_init(|| parse_or_panic("fixture_golden", FIXTURE_GOLDEN_CSV))
        .as_slice()
}

/// Tier A asteroid/TNO/centaur reference rows: sb441-n373s perturber kernel
/// plus per-object pinned JPL SPKs for the centaurs and NEA/personal bodies
/// promoted in slice 3.
pub fn asteroid_reference_corpus() -> &'static [SnapshotEntry] {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    ENTRIES
        .get_or_init(|| parse_or_panic("asteroid_reference", ASTEROID_REFERENCE_CSV))
        .as_slice()
}

/// Tier B constrained asteroid rows (Horizons, 1900–2100). Empty (header-only)
/// after slice-3 promoted every former Tier-B body to Tier-A via per-object
/// SPKs; retained as a stable accessor.
pub fn asteroid_constrained_corpus() -> &'static [SnapshotEntry] {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    ENTRIES
        .get_or_init(|| parse_or_panic("asteroid_constrained", ASTEROID_CONSTRAINED_CSV))
        .as_slice()
}

/// Returns the constrained-corpus rows for a single body (e.g. Eros).
pub fn asteroid_constrained_entries_for(body: &CelestialBody) -> Vec<SnapshotEntry> {
    asteroid_constrained_corpus()
        .iter()
        .filter(|entry| &entry.body == body)
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_backend::{CelestialBody, CustomBodyId};

    fn base_bodies() -> Vec<CelestialBody> {
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
            CelestialBody::Pluto,
        ]
    }

    #[test]
    fn production_reference_corpus_covers_base_bodies_only() {
        let entries = production_reference_corpus();
        assert!(
            !entries.is_empty(),
            "reference corpus should parse non-empty"
        );
        // Dedup by `CelestialBody` equality directly (order-independent) rather
        // than sort-then-dedup, since `CelestialBody` has no `Ord` impl.
        let bodies: std::collections::HashSet<_> = entries.iter().map(|e| &e.body).collect();
        for body in base_bodies() {
            assert!(bodies.contains(&body), "missing base body {body}");
        }
        assert!(
            bodies
                .iter()
                .all(|b| !matches!(b, CelestialBody::Custom(_))),
            "reference corpus must not contain custom/asteroid bodies"
        );
    }

    #[test]
    fn reference_corpus_row_count_matches_manifest_sum() {
        // interior(4984) + boundary(60) + fast_cluster(270) = 5314
        assert_eq!(production_reference_corpus().len(), 5_314);
    }

    #[test]
    fn holdout_corpus_is_separate_and_nonempty() {
        assert_eq!(production_holdout_corpus().len(), 500);
    }

    #[test]
    fn single_file_corpora_parse_non_empty() {
        assert!(
            !fixture_golden_corpus().is_empty(),
            "fixture_golden corpus should parse non-empty"
        );
        assert!(
            !asteroid_reference_corpus().is_empty(),
            "asteroid_reference corpus should parse non-empty"
        );
        // asteroid_constrained is header-only (empty) after slice-3 promoted all
        // former Tier-B bodies to Tier-A via per-object SPKs.
        assert!(
            asteroid_constrained_corpus().is_empty(),
            "asteroid_constrained corpus should be empty after slice-3 promotion"
        );
    }

    #[test]
    fn asteroid_reference_includes_eros() {
        // 433-Eros was promoted from Tier-B/constrained to Tier-A/PinnedKernel
        // in asteroid slice 2 (sb441-n373s). It must appear in the reference corpus.
        let eros = CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros"));
        let rows: Vec<_> = asteroid_reference_corpus()
            .iter()
            .filter(|e| e.body == eros)
            .collect();
        assert!(
            !rows.is_empty(),
            "asteroid_reference should contain Eros rows (promoted to Tier-A in slice 2)"
        );
        assert!(rows.iter().all(|e| e.body == eros));
    }

    #[test]
    fn asteroid_reference_agrees_with_horizons_snapshot_rows() {
        // Issue #201: before #233 the sb441-n373s rows differed from the
        // Horizons snapshot rows at JD 2453000.5 by up to 1.0″ in longitude and
        // 1.9″ in latitude, the old corpus's stale ecliptic-of-date frame. Both
        // are now geocentric ecliptic J2000 and agree within 0.032″ (Juno; the
        // others within 0.007″). A frame regression would move them by arcseconds.
        const EPOCH_JD: f64 = 2_453_000.5;
        const TOLERANCE_ARCSEC: f64 = 0.1;
        let row_at = |rows: &[SnapshotEntry], body: &CelestialBody| {
            rows.iter()
                .find(|e| &e.body == body && e.epoch.julian_day.days() == EPOCH_JD)
                .map(|e| [e.x_km, e.y_km, e.z_km])
                .unwrap_or_else(|| panic!("no {body} row at JD {EPOCH_JD}"))
        };
        let horizons = crate::backend::snapshot_entries().expect("reference snapshot parses");
        for body in [
            CelestialBody::Ceres,
            CelestialBody::Pallas,
            CelestialBody::Juno,
            CelestialBody::Vesta,
            CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros")),
        ] {
            let a = row_at(asteroid_reference_corpus(), &body);
            let b = row_at(horizons, &body);
            let cross = [
                a[1] * b[2] - a[2] * b[1],
                a[2] * b[0] - a[0] * b[2],
                a[0] * b[1] - a[1] * b[0],
            ];
            let cross_norm = cross.iter().map(|c| c * c).sum::<f64>().sqrt();
            let dot = a.iter().zip(b).map(|(p, q)| p * q).sum::<f64>();
            let separation_arcsec = cross_norm.atan2(dot).to_degrees() * 3600.0;
            assert!(
                separation_arcsec < TOLERANCE_ARCSEC,
                "{body}: sb441 and Horizons rows {separation_arcsec}″ apart"
            );
        }
    }
}
