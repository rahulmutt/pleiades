//! Samples an `SpkBackend` into the checked-in CSV corpus schema with
//! provenance comments, so the broader reference corpus is reproducible.

use pleiades_backend::{CelestialBody, EphemerisBackend, EphemerisRequest};
use pleiades_types::{Instant, JulianDay, TimeScale};

use super::backend::SpkBackend;

/// One body sampled across a list of Julian-Day epochs.
pub struct CorpusRequest {
    /// Bodies to sample at every epoch.
    pub bodies: Vec<CelestialBody>,
    /// Julian-Day (TDB) epochs to sample.
    pub epoch_jds: Vec<f64>,
    /// Human-readable provenance label for the kernel source.
    pub source_label: String,
    /// SHA-256 of the source kernel, recorded for reproducibility.
    pub kernel_sha256: String,
}

/// Emits CSV text matching the `epoch_jd,body,x_km,y_km,z_km` snapshot schema,
/// with provenance comment lines, by querying the SPK backend.
///
/// The x/y/z columns store the geocentric **ecliptic** Cartesian vector (km),
/// consistent with how `SnapshotEntry::ecliptic()` reconstructs longitude.
///
/// The `body` column is written via the `Display` impl of [`CelestialBody`],
/// which produces exactly the tokens (`Sun`, `Moon`, `Mercury`, ...,
/// `catalog:designation`) accepted by the snapshot corpus loader's `parse_body`,
/// so the generated CSV round-trips through that loader.
pub fn generate_corpus_csv(backend: &SpkBackend, req: &CorpusRequest) -> Result<String, String> {
    let mut out = corpus_header(&req.source_label, &req.kernel_sha256);
    for &jd in &req.epoch_jds {
        for body in &req.bodies {
            push_corpus_row(&mut out, backend, body, jd)?;
        }
    }
    Ok(out)
}

const AU_IN_KM: f64 = 149_597_870.7;

/// Standard provenance header for every corpus slice CSV.
fn corpus_header(source_label: &str, kernel_sha256: &str) -> String {
    let mut out = String::new();
    out.push_str("#Pleiades SPK Reference Corpus\n");
    out.push_str(&format!("#Source: {source_label}\n"));
    out.push_str(&format!("#Kernel-SHA256: {kernel_sha256}\n"));
    out.push_str("#Coverage: geocentric ecliptic (mean geometric), TDB epochs\n");
    out.push_str(
        "#Redistribution: derived from public-domain JPL DE kernel; corpus is redistributable\n",
    );
    out.push_str("#Columns:epoch_jd,body,x_km,y_km,z_km\n");
    out
}

/// Samples one body at one epoch and appends its `epoch_jd,body,x,y,z` row.
fn push_corpus_row(
    out: &mut String,
    backend: &SpkBackend,
    body: &CelestialBody,
    jd: f64,
) -> Result<(), String> {
    let inst = Instant::new(JulianDay::from_days(jd), TimeScale::Tdb);
    let res = backend
        .position(&EphemerisRequest::new(body.clone(), inst))
        .map_err(|e| format!("body {body:?} at jd {jd}: {}", e.message))?;
    let ec = res
        .ecliptic
        .ok_or_else(|| format!("no ecliptic for {body:?}"))?;
    let r_km = ec.distance_au.unwrap_or(0.0) * AU_IN_KM;
    let lon = ec.longitude.degrees().to_radians();
    let lat = ec.latitude.degrees().to_radians();
    let x = r_km * lat.cos() * lon.cos();
    let y = r_km * lat.cos() * lon.sin();
    let z = r_km * lat.sin();
    out.push_str(&format!("{jd},{body},{x:.6},{y:.6},{z:.6}\n"));
    Ok(())
}

/// Emits a corpus CSV where each body is sampled at its own epoch list.
/// Bodies are emitted in the given order; epochs in the given (already sorted)
/// order. Used by the interior backbone so slow bodies are not over-sampled.
///
/// Rows are therefore grouped by body (body-outer, epoch-inner) and are NOT
/// globally sorted by epoch across bodies — this is intentional for the
/// interior backbone, where each body has its own cadence. The
/// verify-from-kernel test regenerates with this same function, so the
/// ordering is stable and reproducible across runs.
pub(crate) fn generate_corpus_csv_per_body(
    backend: &SpkBackend,
    per_body: &[(CelestialBody, Vec<f64>)],
    source_label: &str,
    kernel_sha256: &str,
) -> Result<String, String> {
    let mut out = corpus_header(source_label, kernel_sha256);
    for (body, epochs) in per_body {
        for &jd in epochs {
            push_corpus_row(&mut out, backend, body, jd)?;
        }
    }
    Ok(out)
}

use crate::spk::corpus_manifest::{corpus_checksum64, CorpusManifest, SliceEntry};
use crate::spk::corpus_spec::{self, SliceRole};

/// One generated slice: its role, file name, and CSV text.
pub struct GeneratedSlice {
    pub role: SliceRole,
    pub file: String,
    pub csv: String,
}

/// Generates one slice's CSV by sampling the backend at the spec-defined epochs
/// for that role, reusing the existing single-slice CSV emitter.
///
/// For production use this always requests `all_bodies()` (or the fast-cluster
/// set). Tests that need a narrowed body set should call
/// [`generate_slice_with_bodies`] directly.
pub fn generate_slice(backend: &SpkBackend, role: SliceRole) -> Result<GeneratedSlice, String> {
    match role {
        SliceRole::FastCluster => generate_slice_with_bodies(
            backend,
            role,
            vec![
                CelestialBody::Moon,
                CelestialBody::Mercury,
                CelestialBody::Venus,
            ],
        ),
        SliceRole::FixtureGolden => {
            Err("fixture_golden is sourced from existing fixtures, not generated".into())
        }
        SliceRole::AsteroidConstrained => {
            Err("asteroid_constrained is sourced from Horizons, not generated".into())
        }
        SliceRole::AsteroidReference => generate_asteroid_reference_slice(backend),
        other => generate_slice_with_bodies(backend, other, all_bodies()),
    }
}

/// Inner slice generator that accepts an explicit body list. Production code
/// calls [`generate_slice`] which always uses the canonical body sets; test
/// code may call this directly with a narrowed set so the synthetic backend
/// does not need to cover every release body.
pub(crate) fn generate_slice_with_bodies(
    backend: &SpkBackend,
    role: SliceRole,
    bodies: Vec<CelestialBody>,
) -> Result<GeneratedSlice, String> {
    if role == SliceRole::InteriorBackbone {
        let per_body: Vec<(CelestialBody, Vec<f64>)> = bodies
            .iter()
            .map(|b| (b.clone(), corpus_spec::interior_epochs_for(b)))
            .collect();
        let mut csv = generate_corpus_csv_per_body(
            backend,
            &per_body,
            corpus_spec::KERNEL_LABEL,
            corpus_spec::KERNEL_SHA256,
        )?;
        csv = csv.replace(
            "#Columns:",
            &format!("#Slice-Role: {}\n#Columns:", role.token()),
        );
        return Ok(GeneratedSlice {
            role,
            file: "interior.csv".to_string(),
            csv,
        });
    }

    let (file, epochs) = match role {
        SliceRole::Boundary => ("boundary.csv", corpus_spec::boundary_epochs()),
        SliceRole::FastCluster => ("fast_clusters.csv", corpus_spec::fast_cluster_epochs()),
        SliceRole::Holdout => ("holdout.csv", corpus_spec::holdout_epochs(50)),
        SliceRole::InteriorBackbone => unreachable!("interior handled above"),
        SliceRole::FixtureGolden => {
            return Err("fixture_golden is sourced from existing fixtures, not generated".into())
        }
        SliceRole::AsteroidConstrained => {
            return Err(
                "asteroid_constrained is sourced from Horizons, not generated".into(),
            )
        }
        SliceRole::AsteroidReference => {
            return generate_asteroid_reference_slice(backend)
        }
    };
    let req = CorpusRequest {
        bodies,
        epoch_jds: epochs,
        source_label: corpus_spec::KERNEL_LABEL.to_string(),
        kernel_sha256: corpus_spec::KERNEL_SHA256.to_string(),
    };
    let mut csv = generate_corpus_csv(backend, &req)?;
    // Insert the slice-role header line after the redistribution line.
    csv = csv.replace(
        "#Columns:",
        &format!("#Slice-Role: {}\n#Columns:", role.token()),
    );
    Ok(GeneratedSlice {
        role,
        file: file.to_string(),
        csv,
    })
}

fn all_bodies() -> Vec<CelestialBody> {
    let mut bodies = corpus_spec::release_bodies();
    bodies.extend(corpus_spec::constrained_bodies());
    bodies
}

/// Placeholder: full implementation lands in Task 5.
fn generate_asteroid_reference_slice(_backend: &SpkBackend) -> Result<GeneratedSlice, String> {
    Err("implemented in Task 5".into())
}

/// Builds the manifest for a set of generated slices.
pub fn build_manifest(slices: &[GeneratedSlice]) -> CorpusManifest {
    let entries = slices
        .iter()
        .map(|s| SliceEntry {
            name: s.role.token().to_string(),
            file: s.file.clone(),
            role: s.role.token().to_string(),
            rows: s
                .csv
                .lines()
                .filter(|l| !l.starts_with('#') && !l.is_empty())
                .count(),
            checksum: corpus_checksum64(&s.csv),
        })
        .collect();
    CorpusManifest {
        kernel: "de440.bsp".to_string(),
        kernel_sha256: corpus_spec::KERNEL_SHA256.to_string(),
        slices: entries,
    }
}

#[cfg(test)]
mod slice_tests {
    use super::*;
    use crate::spk::test_support::{build_daf, type2_record, type2_segment_data, SegmentSpec};

    fn const_seg(target: i32, center: i32, pos: [f64; 3]) -> SegmentSpec {
        let rec = type2_record(0.0, 1.0e12, &[pos[0], 0.0], &[pos[1], 0.0], &[pos[2], 0.0]);
        let data = type2_segment_data(-1.0e12, 2.0e12, rec.len(), &[rec]);
        SegmentSpec {
            start_et: -1.0e12,
            stop_et: 1.0e12,
            target,
            center,
            frame: 1,
            data_type: 2,
            data,
            name: "C".to_string(),
        }
    }

    fn backend() -> SpkBackend {
        // Minimal chain so Sun resolves; other bodies share the const segment.
        let blob = build_daf(&[
            const_seg(10, 0, [1.0e8, 2.0e7, 0.0]),
            const_seg(399, 3, [0.0, 0.0, 0.0]),
            const_seg(3, 0, [0.0, 0.0, 0.0]),
        ]);
        SpkBackend::builder()
            .add_kernel_bytes(blob, "syn")
            .unwrap()
            .build()
    }

    #[test]
    fn boundary_slice_has_role_header_and_rows() {
        // Use narrowed body list (Sun only) so the synthetic backend resolves.
        let slice =
            generate_slice_with_bodies(&backend(), SliceRole::Boundary, vec![CelestialBody::Sun])
                .unwrap();
        assert!(slice.csv.contains("#Slice-Role: boundary"));
        assert!(slice.csv.contains("#Columns:epoch_jd,body,x_km,y_km,z_km"));
        assert!(slice
            .csv
            .lines()
            .any(|l| l.starts_with("23") && l.contains("Sun")));
    }

    #[test]
    fn interior_uses_per_body_cadence_and_includes_anchor() {
        // Synthetic backend resolves Sun via the const segment chain.
        let slice = generate_slice_with_bodies(
            &backend(),
            SliceRole::InteriorBackbone,
            vec![CelestialBody::Sun],
        )
        .unwrap();
        assert!(slice.csv.contains("#Slice-Role: interior"));
        // Anchor epoch J2000 must appear.
        assert!(
            slice
                .csv
                .lines()
                .any(|l| l.starts_with("2451545") && l.contains("Sun")),
            "interior must include the J2000 anchor"
        );
        // Body-outer ordering: all Sun rows are contiguous (only Sun here, so just
        // assert rows exist and are sorted by epoch).
        let epochs: Vec<f64> = slice
            .csv
            .lines()
            .filter(|l| !l.starts_with('#') && !l.is_empty())
            .map(|l| l.split(',').next().unwrap().parse().unwrap())
            .collect();
        assert!(
            epochs.windows(2).all(|w| w[1] >= w[0]),
            "epochs ascending per body"
        );
    }

    #[test]
    fn manifest_counts_data_rows_and_checksums() {
        let slice =
            generate_slice_with_bodies(&backend(), SliceRole::Boundary, vec![CelestialBody::Sun])
                .unwrap();
        let manifest = build_manifest(std::slice::from_ref(&slice));
        assert_eq!(manifest.slices.len(), 1);
        assert!(manifest.slices[0].rows > 0);
        assert_ne!(manifest.slices[0].checksum, 0);
    }

    #[test]
    fn fixture_golden_is_not_generated() {
        assert!(generate_slice(&backend(), SliceRole::FixtureGolden).is_err());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::parse_snapshot_entries;
    use crate::spk::test_support::{build_daf, type2_record, type2_segment_data, SegmentSpec};

    fn const_seg(target: i32, center: i32, pos: [f64; 3]) -> SegmentSpec {
        let rec = type2_record(0.0, 1.0e12, &[pos[0], 0.0], &[pos[1], 0.0], &[pos[2], 0.0]);
        let data = type2_segment_data(-1.0e12, 2.0e12, rec.len(), &[rec]);
        SegmentSpec {
            start_et: -1.0e12,
            stop_et: 1.0e12,
            target,
            center,
            frame: 1,
            data_type: 2,
            data,
            name: "C".to_string(),
        }
    }

    #[test]
    fn generates_csv_with_provenance_and_rows() {
        let blob = build_daf(&[
            const_seg(10, 0, [1.0e8, 2.0e7, 0.0]),
            const_seg(399, 3, [0.0, 0.0, 0.0]),
            const_seg(3, 0, [0.0, 0.0, 0.0]),
        ]);
        let backend = SpkBackend::builder()
            .add_kernel_bytes(blob, "syn")
            .unwrap()
            .build();
        let req = CorpusRequest {
            bodies: vec![CelestialBody::Sun],
            epoch_jds: vec![2_451_545.0],
            source_label: "de440 (synthetic test)".to_string(),
            kernel_sha256: "deadbeef".to_string(),
        };
        let csv = generate_corpus_csv(&backend, &req).unwrap();
        assert!(csv.contains("#Columns:epoch_jd,body,x_km,y_km,z_km"));
        assert!(csv.contains("#Kernel-SHA256: deadbeef"));
        assert!(csv
            .lines()
            .any(|l| l.starts_with("2451545") && l.contains("Sun")));

        // Round-trip: the generated CSV must parse back through the existing
        // snapshot corpus loader, yielding the same body and epoch. This is the
        // whole point of writing the corpus schema, so the body token written
        // via `{body}` must be exactly what `parse_body` accepts.
        let entries = parse_snapshot_entries(&csv).expect("generated CSV must round-trip");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].body, CelestialBody::Sun);
        assert_eq!(entries[0].epoch.julian_day.days(), 2_451_545.0);
        // The body token emitted by `{body}` equals the loader's expected token.
        assert_eq!(format!("{}", CelestialBody::Sun), "Sun");
    }
}
