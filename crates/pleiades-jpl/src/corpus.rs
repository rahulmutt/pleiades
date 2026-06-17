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

/// Tier A asteroid reference rows (sb441-n16).
pub fn asteroid_reference_corpus() -> &'static [SnapshotEntry] {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    ENTRIES
        .get_or_init(|| parse_or_panic("asteroid_reference", ASTEROID_REFERENCE_CSV))
        .as_slice()
}

/// Tier B constrained asteroid rows (Horizons, 1900–2100).
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
        // interior(24813) + boundary(60) + fast_cluster(270) = 25143
        assert_eq!(production_reference_corpus().len(), 25_143);
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
        assert!(
            !asteroid_constrained_corpus().is_empty(),
            "asteroid_constrained corpus should parse non-empty"
        );
    }

    #[test]
    fn asteroid_constrained_includes_eros() {
        let eros = CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros"));
        let rows = asteroid_constrained_entries_for(&eros);
        assert!(
            !rows.is_empty(),
            "asteroid_constrained should contain Eros rows"
        );
        assert!(rows.iter().all(|e| e.body == eros));
    }
}
