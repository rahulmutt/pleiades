//! Fail-closed gate over the checked-in production corpus slices.

use pleiades_backend::CelestialBody;
use pleiades_jpl::parse_snapshot_entries;
use pleiades_jpl::spk::corpus_manifest::{corpus_checksum64, CorpusManifest};
use pleiades_jpl::spk::corpus_spec;

/// A loaded slice: its role token and CSV text.
pub struct LoadedSlice {
    pub role: String,
    pub csv: String,
}

/// Validates that every release-claimed body appears in the corpus and that the
/// boundary, interior, and hold-out roles are all present and non-empty.
/// Returns the first violation as `Err`, fail-closed.
pub fn validate_completeness(slices: &[LoadedSlice]) -> Result<(), String> {
    let required_roles = ["boundary", "interior", "holdout"];
    for role in required_roles {
        let slice = slices
            .iter()
            .find(|s| s.role == role)
            .ok_or(format!("missing required slice role: {role}"))?;
        let entries = parse_snapshot_entries(&slice.csv)
            .map_err(|e| format!("slice {role} failed to parse: {e:?}"))?;
        if entries.is_empty() {
            return Err(format!("slice {role} has no data rows"));
        }
    }

    // Every release body must appear somewhere in the corpus.
    let mut seen: Vec<CelestialBody> = Vec::new();
    for slice in slices {
        if let Ok(entries) = parse_snapshot_entries(&slice.csv) {
            for e in entries {
                if !seen.contains(&e.body) {
                    seen.push(e.body);
                }
            }
        }
    }
    for body in corpus_spec::release_bodies() {
        if !seen.contains(&body) {
            return Err(format!(
                "release-claimed body missing from corpus: {body:?}"
            ));
        }
    }
    Ok(())
}

/// Validates schema header presence, finite numeric fields, and that the
/// `#Kernel-SHA256` is not the unfilled placeholder.
pub fn validate_schema_and_provenance(slices: &[LoadedSlice]) -> Result<(), String> {
    for s in slices {
        if !s.csv.contains("#Columns:epoch_jd,body,x_km,y_km,z_km") {
            return Err(format!("slice {} missing column header", s.role));
        }
        if s.csv.contains("#Kernel-SHA256: <pinned-after-download>")
            || s.csv.contains("#Kernel-SHA256: <run shasum")
        {
            return Err(format!("slice {} has placeholder kernel SHA-256", s.role));
        }
        for line in s
            .csv
            .lines()
            .filter(|l| !l.starts_with('#') && !l.is_empty())
        {
            let fields: Vec<&str> = line.split(',').collect();
            // Accept 5-column rows (position only) or 8-column rows (position +
            // velocity), which the holdout slice carries as of Task 7.
            if fields.len() != 5 && fields.len() != 8 {
                return Err(format!(
                    "malformed row in {} (expected 5 or 8 fields, found {}): {line}",
                    s.role,
                    fields.len()
                ));
            }
            for field in &fields[2..] {
                let v: f64 = field
                    .parse()
                    .map_err(|_| format!("non-numeric field in {}", s.role))?;
                if !v.is_finite() {
                    return Err(format!("non-finite field in {}", s.role));
                }
            }
        }
    }
    Ok(())
}

/// Per-body-class position tolerance in km for the fixture-golden cross-check.
fn tolerance_km(body: &CelestialBody) -> f64 {
    match body {
        CelestialBody::Moon => 5.0,
        // de440 provides only system barycenters (NAIF 5/6/7/8) for the giant
        // planets, not planet-center IDs (599/699/799/899), so the reader
        // resolves them to the barycenter while the Horizons fixture_golden uses
        // the planet center. The barycenter<->center offset is real and body-
        // specific (Jupiter ~64 km, Saturn ~300 km, Uranus ~352 km, Neptune
        // ~74 km at J2000; worst-case aligned-moon geometry is larger) but
        // astrologically negligible (<0.1"). 600 km clears that offset with
        // margin while still rejecting gross errors (wrong NAIF id, axis flip,
        // 180-degree rotation). It does NOT catch subtle sub-600 km reader
        // errors for these bodies; the tight inner-planet tolerances below do
        // that for bodies whose center coincides with their barycenter.
        CelestialBody::Jupiter
        | CelestialBody::Saturn
        | CelestialBody::Uranus
        | CelestialBody::Neptune => 600.0,
        CelestialBody::Pluto => 5_000.0, // constrained/approximate (Charon barycenter)
        _ => 50.0,
    }
}

/// Cross-checks fixture-golden values against backend-sourced slices at any
/// shared (body, epoch). Release bodies failing tolerance are an error;
/// constrained bodies (Pluto) are reported but not fatal. Zero overlap is Ok.
pub fn cross_check_fixture_golden(slices: &[LoadedSlice]) -> Result<(), String> {
    // Find the fixture_golden slice; if absent, return Ok.
    let golden_slice = match slices.iter().find(|s| s.role == "fixture_golden") {
        Some(s) => s,
        None => return Ok(()),
    };

    let golden_entries = parse_snapshot_entries(&golden_slice.csv)
        .map_err(|e| format!("fixture_golden failed to parse: {e:?}"))?;

    // Parse all non-fixture_golden slices into a flat lookup vec.
    let mut backend_entries: Vec<pleiades_jpl::SnapshotEntry> = Vec::new();
    for s in slices {
        if s.role == "fixture_golden" {
            continue;
        }
        if let Ok(entries) = parse_snapshot_entries(&s.csv) {
            backend_entries.extend(entries);
        }
    }

    let constrained = corpus_spec::constrained_bodies();

    for golden in &golden_entries {
        let golden_jd = golden.epoch.julian_day.days();
        // Find a backend entry with the same body and JD within 1e-6.
        let matching = backend_entries.iter().find(|e| {
            e.body == golden.body && (e.epoch.julian_day.days() - golden_jd).abs() < 1e-6
        });
        if let Some(backend) = matching {
            let dx = backend.x_km - golden.x_km;
            let dy = backend.y_km - golden.y_km;
            let dz = backend.z_km - golden.z_km;
            let distance = (dx * dx + dy * dy + dz * dz).sqrt();
            let tol = tolerance_km(&golden.body);
            if distance > tol {
                if constrained.contains(&golden.body) {
                    // Constrained body mismatch: not fatal, just note it.
                    // (Keep it simple — drop the note per spec.)
                } else {
                    return Err(format!(
                        "fixture-golden cross-check failed for {:?} at JD {:.6}: \
                         distance {:.3} km exceeds tolerance {:.1} km",
                        golden.body, golden_jd, distance, tol
                    ));
                }
            }
        }
    }

    Ok(())
}

/// Validates each slice's content checksum against the manifest.
pub fn validate_drift(slices: &[LoadedSlice], manifest: &CorpusManifest) -> Result<(), String> {
    for s in slices {
        let entry = manifest
            .slices
            .iter()
            .find(|e| e.role == s.role)
            .ok_or(format!("manifest missing slice for role {}", s.role))?;
        let actual = corpus_checksum64(&s.csv);
        if actual != entry.checksum {
            return Err(format!(
                "checksum drift for {}: manifest {} != actual {}",
                s.role, entry.checksum, actual
            ));
        }
    }
    Ok(())
}

/// Loads the checked-in corpus slices + manifest and runs every gate.
/// Returns a one-line success summary or the first violation.
pub fn run_corpus_gate() -> Result<String, String> {
    let slices = embedded_slices();
    let manifest = CorpusManifest::parse(EMBEDDED_MANIFEST)?;
    validate_completeness(&slices)?;
    validate_schema_and_provenance(&slices)?;
    validate_drift(&slices, &manifest)?;
    cross_check_fixture_golden(&slices)?;
    crate::corpus::asteroid::validate_asteroid_slices(&slices)?;
    let rows: usize = slices
        .iter()
        .map(|s| {
            s.csv
                .lines()
                .filter(|l| !l.starts_with('#') && !l.is_empty())
                .count()
        })
        .sum();
    Ok(format!(
        "corpus gate ok: {} slices, {} data rows, kernel {}",
        slices.len(),
        rows,
        manifest.kernel
    ))
}

const EMBEDDED_MANIFEST: &str = include_str!("../../../pleiades-jpl/data/corpus/manifest.txt");

fn embedded_slices() -> Vec<LoadedSlice> {
    let files = [
        (
            "boundary",
            include_str!("../../../pleiades-jpl/data/corpus/boundary.csv"),
        ),
        (
            "interior",
            include_str!("../../../pleiades-jpl/data/corpus/interior.csv"),
        ),
        (
            "fast_cluster",
            include_str!("../../../pleiades-jpl/data/corpus/fast_clusters.csv"),
        ),
        (
            "holdout",
            include_str!("../../../pleiades-jpl/data/corpus/holdout.csv"),
        ),
        (
            "fixture_golden",
            include_str!("../../../pleiades-jpl/data/corpus/fixture_golden.csv"),
        ),
        (
            "asteroid_reference",
            include_str!("../../../pleiades-jpl/data/corpus/asteroid_reference.csv"),
        ),
        (
            "asteroid_constrained",
            include_str!("../../../pleiades-jpl/data/corpus/asteroid_constrained.csv"),
        ),
    ];
    files
        .iter()
        .map(|(role, csv)| LoadedSlice {
            role: role.to_string(),
            csv: csv.to_string(),
        })
        .collect()
}

#[cfg(test)]
mod drift_tests {
    use super::*;
    use pleiades_jpl::spk::corpus_manifest::SliceEntry;

    fn slice(role: &str, sha: &str) -> LoadedSlice {
        LoadedSlice {
            role: role.to_string(),
            csv: format!(
                "#Kernel-SHA256: {sha}\n#Columns:epoch_jd,body,x_km,y_km,z_km\n2451545,Sun,1.0,2.0,3.0\n"
            ),
        }
    }

    #[test]
    fn placeholder_sha_fails() {
        let s = vec![slice("boundary", "<pinned-after-download>")];
        assert!(validate_schema_and_provenance(&s).is_err());
    }

    #[test]
    fn real_sha_passes_schema() {
        let s = vec![slice("boundary", "deadbeef")];
        assert!(validate_schema_and_provenance(&s).is_ok());
    }

    #[test]
    fn checksum_mismatch_fails() {
        let s = slice("boundary", "deadbeef");
        let manifest = CorpusManifest {
            kernel: "de440.bsp".to_string(),
            kernel_sha256: "deadbeef".to_string(),
            slices: vec![SliceEntry {
                name: "boundary".to_string(),
                file: "boundary.csv".to_string(),
                role: "boundary".to_string(),
                rows: 1,
                checksum: 12345, // deliberately wrong
            }],
        };
        assert!(validate_drift(&[s], &manifest).is_err());
    }

    #[test]
    fn matching_checksum_passes() {
        let s = slice("boundary", "deadbeef");
        let checksum = corpus_checksum64(&s.csv);
        let manifest = CorpusManifest {
            kernel: "de440.bsp".to_string(),
            kernel_sha256: "deadbeef".to_string(),
            slices: vec![SliceEntry {
                name: "boundary".to_string(),
                file: "boundary.csv".to_string(),
                role: "boundary".to_string(),
                rows: 1,
                checksum,
            }],
        };
        assert!(validate_drift(&[s], &manifest).is_ok());
    }

    #[test]
    fn malformed_short_row_fails() {
        let s = vec![LoadedSlice {
            role: "boundary".to_string(),
            csv: "#Columns:epoch_jd,body,x_km,y_km,z_km\n2451545,Sun\n".to_string(),
        }];
        assert!(validate_schema_and_provenance(&s).is_err());
    }

    #[test]
    fn non_numeric_coordinate_fails() {
        let s = vec![LoadedSlice {
            role: "boundary".to_string(),
            csv: "#Columns:epoch_jd,body,x_km,y_km,z_km\n2451545,Sun,1.0,abc,3.0\n".to_string(),
        }];
        assert!(validate_schema_and_provenance(&s).is_err());
    }

    #[test]
    fn run_shasum_placeholder_form_fails() {
        let s = vec![LoadedSlice {
            role: "boundary".to_string(),
            csv: "#Kernel-SHA256: <run shasum -a 256 de440.bsp>\n#Columns:epoch_jd,body,x_km,y_km,z_km\n2451545,Sun,1.0,2.0,3.0\n".to_string(),
        }];
        assert!(validate_schema_and_provenance(&s).is_err());
    }
}

#[cfg(test)]
mod tolerance_tests {
    use super::*;

    #[test]
    fn tolerance_is_looser_for_constrained_bodies() {
        assert!(tolerance_km(&CelestialBody::Pluto) > tolerance_km(&CelestialBody::Mars));
    }

    #[test]
    fn giant_planets_allow_barycenter_offset() {
        // de440 exposes only giant-planet barycenters, so the cross-check vs
        // planet-center Horizons must tolerate the (astrologically negligible)
        // barycenter<->center offset — pinned at 600 km, looser than inner
        // planets, tighter than the constrained Pluto allowance.
        for b in [
            CelestialBody::Jupiter,
            CelestialBody::Saturn,
            CelestialBody::Uranus,
            CelestialBody::Neptune,
        ] {
            assert_eq!(tolerance_km(&b), 600.0, "{b:?} tolerance must be pinned");
            assert!(
                tolerance_km(&b) > tolerance_km(&CelestialBody::Mars),
                "{b:?} > Mars"
            );
            assert!(
                tolerance_km(&b) < tolerance_km(&CelestialBody::Pluto),
                "{b:?} < Pluto"
            );
        }
        assert_eq!(tolerance_km(&CelestialBody::Mars), 50.0);
        assert_eq!(tolerance_km(&CelestialBody::Moon), 5.0);
        assert_eq!(tolerance_km(&CelestialBody::Pluto), 5_000.0);
    }
}

#[cfg(test)]
mod cross_check_tests {
    use super::*;

    fn header(role: &str) -> String {
        format!(
            "#Slice-Role: {role}\n#Kernel-SHA256: abc123\n#Columns:epoch_jd,body,x_km,y_km,z_km\n"
        )
    }

    fn make_slice(role: &str, rows: &str) -> LoadedSlice {
        LoadedSlice {
            role: role.to_string(),
            csv: format!("{}{}", header(role), rows),
        }
    }

    #[test]
    fn identical_coordinates_pass() {
        let slices = vec![
            make_slice("fixture_golden", "2451545.0,Sun,1000.0,2000.0,3000.0\n"),
            make_slice("boundary", "2451545.0,Sun,1000.0,2000.0,3000.0\n"),
        ];
        assert!(cross_check_fixture_golden(&slices).is_ok());
    }

    #[test]
    fn large_coordinate_difference_fails_for_release_body() {
        // Sun differs by 100 km in x => distance 100 > tolerance 50 => Err
        let slices = vec![
            make_slice("fixture_golden", "2451545.0,Sun,1000.0,2000.0,3000.0\n"),
            make_slice("boundary", "2451545.0,Sun,1100.0,2000.0,3000.0\n"),
        ];
        assert!(cross_check_fixture_golden(&slices).is_err());
    }

    #[test]
    fn constrained_body_mismatch_not_fatal() {
        // Pluto differs by 100 km (> 50 km general tolerance, < 5000 km Pluto tolerance) => Ok
        let slices = vec![
            make_slice("fixture_golden", "2451545.0,Pluto,1000.0,2000.0,3000.0\n"),
            make_slice("boundary", "2451545.0,Pluto,1100.0,2000.0,3000.0\n"),
        ];
        // 100 km < 5000 km tolerance => also Ok because within tolerance
        assert!(cross_check_fixture_golden(&slices).is_ok());
    }

    #[test]
    fn constrained_body_far_mismatch_not_fatal() {
        // Pluto differs by 200 km which is > 50 km but < 5000 km. Tolerance is 5000, so still Ok.
        // To test the "not fatal even when over tolerance" path, use > 5000 km difference
        // and verify it still returns Ok (constrained bodies are never fatal).
        let slices = vec![
            make_slice("fixture_golden", "2451545.0,Pluto,0.0,0.0,0.0\n"),
            make_slice("boundary", "2451545.0,Pluto,6000.0,0.0,0.0\n"),
        ];
        // Distance = 6000 km > 5000 km tolerance, but Pluto is constrained => Ok
        assert!(cross_check_fixture_golden(&slices).is_ok());
    }

    #[test]
    fn no_fixture_golden_slice_returns_ok() {
        let slices = vec![
            make_slice("boundary", "2451545.0,Sun,1000.0,2000.0,3000.0\n"),
            make_slice("interior", "2451545.0,Mars,4000.0,5000.0,6000.0\n"),
        ];
        assert!(cross_check_fixture_golden(&slices).is_ok());
    }

    #[test]
    fn giant_planet_over_tolerance_fails_for_release_body() {
        // Jupiter offset of 700 km exceeds the 600 km giant-planet tolerance.
        let slices = vec![
            make_slice("fixture_golden", "2451545.0,Jupiter,0.0,0.0,0.0\n"),
            make_slice("boundary", "2451545.0,Jupiter,700.0,0.0,0.0\n"),
        ];
        assert!(cross_check_fixture_golden(&slices).is_err());
    }

    #[test]
    fn giant_planet_within_tolerance_passes() {
        // 400 km < 600 km giant-planet tolerance => Ok.
        let slices = vec![
            make_slice("fixture_golden", "2451545.0,Jupiter,0.0,0.0,0.0\n"),
            make_slice("boundary", "2451545.0,Jupiter,400.0,0.0,0.0\n"),
        ];
        assert!(cross_check_fixture_golden(&slices).is_ok());
    }

    #[test]
    fn no_overlap_returns_ok() {
        // fixture_golden has Sun at JD 2451545, backend has Sun at different JD => no overlap
        let slices = vec![
            make_slice("fixture_golden", "2451545.0,Sun,1000.0,2000.0,3000.0\n"),
            make_slice("boundary", "2451546.0,Sun,9999.0,9999.0,9999.0\n"),
        ];
        assert!(cross_check_fixture_golden(&slices).is_ok());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header(role: &str) -> String {
        format!("#Slice-Role: {role}\n#Columns:epoch_jd,body,x_km,y_km,z_km\n")
    }

    fn full_corpus() -> Vec<LoadedSlice> {
        let bodies = corpus_spec::release_bodies();
        let mut rows = String::new();
        for b in &bodies {
            rows.push_str(&format!("2451545,{b},1.0,2.0,3.0\n"));
        }
        ["boundary", "interior", "holdout"]
            .iter()
            .map(|r| LoadedSlice {
                role: r.to_string(),
                csv: format!("{}{}", header(r), rows),
            })
            .collect()
    }

    #[test]
    fn full_corpus_passes() {
        assert!(validate_completeness(&full_corpus()).is_ok());
    }

    #[test]
    fn missing_role_fails() {
        let mut corpus = full_corpus();
        corpus.retain(|s| s.role != "holdout");
        assert!(validate_completeness(&corpus).is_err());
    }

    #[test]
    fn missing_body_fails() {
        let mut corpus = full_corpus();
        // Strip Mars from every slice.
        for s in &mut corpus {
            s.csv = s
                .csv
                .lines()
                .filter(|l| !l.contains("Mars"))
                .collect::<Vec<_>>()
                .join("\n");
        }
        assert!(validate_completeness(&corpus).is_err());
    }

    #[test]
    fn empty_slice_fails() {
        let mut corpus = full_corpus();
        corpus[0].csv = header("boundary");
        assert!(validate_completeness(&corpus).is_err());
    }

    #[test]
    fn embedded_corpus_gate_passes() {
        run_corpus_gate().unwrap();
    }

    #[test]
    fn corpus_gate_runs_asteroid_validation() {
        // validate_asteroid_slices must surface asteroid-window breaches.
        // Epoch 2200000.0 is well below AST_RANGE_START_JD (2_415_020.5, ~1900
        // CE), so it falls outside the asteroid window and must be rejected.
        let slices = vec![LoadedSlice {
            role: "asteroid_reference".to_string(),
            csv: "#Columns:epoch_jd,body,x_km,y_km,z_km\n2200000.0,Ceres,1.0,2.0,3.0\n".to_string(),
        }];
        assert!(crate::corpus::asteroid::validate_asteroid_slices(&slices).is_err());
    }

    /// Regression test: the committed holdout slice now carries 8 fields (position +
    /// velocity).  The full corpus gate must pass end-to-end, meaning
    /// `run_corpus_gate()` must return Ok with the expected success-string prefix.
    ///
    /// This test would fail if the gate rejected 8-field rows (wrong-arity branch)
    /// or if the finite-check rejected valid velocity values in the holdout slice.
    #[test]
    fn corpus_gate_accepts_eight_field_velocity_rows() {
        let summary = run_corpus_gate().expect("corpus gate should pass with 8-field holdout slice");
        assert!(
            summary.contains("corpus gate ok"),
            "unexpected success string: {summary}"
        );
    }

    /// Regression test: an 8-field row whose velocity field contains a non-finite
    /// value ("NaN") must be rejected by `validate_schema_and_provenance`.
    ///
    /// The crafted slice has:
    ///  - a valid 8-field `#Columns` header (which contains the 5-field prefix so
    ///    the header check passes),
    ///  - a real kernel SHA so the provenance check passes,
    ///  - one otherwise well-formed 8-field data row whose third velocity field is
    ///    "NaN", triggering the finite-check at fields[2..] iteration.
    ///
    /// This test would fail if the finite-check were removed from the gate logic
    /// (the row would then be accepted instead of rejected).
    #[test]
    fn corpus_gate_rejects_nonfinite_velocity() {
        let slices = vec![LoadedSlice {
            role: "holdout".to_string(),
            csv: concat!(
                "#Kernel-SHA256: deadbeef\n",
                "#Columns:epoch_jd,body,x_km,y_km,z_km,vx_km_s,vy_km_s,vz_km_s\n",
                // epoch_jd, body, x, y, z, vx, vy — all finite, but vz is NaN
                "2451545.0,Sun,1.0,2.0,3.0,0.1,0.2,NaN\n",
            )
            .to_string(),
        }];
        assert!(
            validate_schema_and_provenance(&slices).is_err(),
            "gate must reject a row with a NaN velocity field"
        );
    }
}
