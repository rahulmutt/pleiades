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
    const AU_IN_KM: f64 = 149_597_870.7;
    let mut out = String::new();
    out.push_str("#Pleiades SPK Reference Corpus\n");
    out.push_str(&format!("#Source: {}\n", req.source_label));
    out.push_str(&format!("#Kernel-SHA256: {}\n", req.kernel_sha256));
    out.push_str("#Coverage: geocentric ecliptic (mean geometric), TDB epochs\n");
    out.push_str(
        "#Redistribution: derived from public-domain JPL DE kernel; corpus is redistributable\n",
    );
    out.push_str("#Columns:epoch_jd,body,x_km,y_km,z_km\n");

    for &jd in &req.epoch_jds {
        let inst = Instant::new(JulianDay::from_days(jd), TimeScale::Tdb);
        for body in &req.bodies {
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
        }
    }
    Ok(out)
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
