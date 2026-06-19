//! `generate-spk-corpus` command: sample a DE kernel into the corpus CSV.

use pleiades_core::CelestialBody;
use pleiades_jpl::spk::corpus_manifest::{corpus_checksum64, SliceEntry};
use pleiades_jpl::spk::corpus_spec::SliceRole;
use pleiades_jpl::{
    build_manifest, generate_corpus_csv, generate_slice, CorpusRequest, SpkBackend,
};

/// Parses args of the form:
///
///   `generate-spk-corpus <kernel.bsp> <jd1> [jd2 ...]`  — legacy single-shot mode
///   `generate-spk-corpus <kernel.bsp> --emit-slices <out-dir>`  — multi-slice mode
///
/// The legacy mode prints the corpus CSV to stdout.
/// The `--emit-slices` mode generates all four backend-sourced corpus slices
/// (boundary, interior, fast_clusters, holdout) plus `manifest.txt` into
/// `<out-dir>`, using the spec-driven `generate_slice` + `build_manifest` from
/// `pleiades-jpl`.
pub fn render_spk_corpus(args: &[&str]) -> Result<String, String> {
    let kernel = args
        .first()
        .ok_or("generate-spk-corpus requires a kernel path")?;

    // --emit-slices branch
    if args.get(1).copied() == Some("--emit-slices") {
        let out_dir = args
            .get(2)
            .ok_or("--emit-slices requires an output directory")?;
        let backend = SpkBackend::builder()
            .add_kernel(kernel)
            .map_err(|e| e.message)?
            .build();
        return emit_slices(&backend, out_dir);
    }

    // Legacy JD-list mode (unchanged).
    let jds: Vec<f64> = args[1..]
        .iter()
        .map(|s| s.parse::<f64>().map_err(|_| format!("bad JD: {s}")))
        .collect::<Result<_, _>>()?;
    if jds.is_empty() {
        return Err("generate-spk-corpus requires at least one Julian Day".to_string());
    }
    let backend = SpkBackend::builder()
        .add_kernel(kernel)
        .map_err(|e| e.message)?
        .build();
    let bodies = vec![
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
    ];
    let req = CorpusRequest {
        bodies,
        epoch_jds: jds,
        source_label: format!("JPL DE SPK kernel: {kernel}"),
        kernel_sha256: "<run shasum -a 256 on the kernel>".to_string(),
    };
    generate_corpus_csv(&backend, &req)
}

/// Generates the four backend-sourced corpus slices plus `manifest.txt` into
/// `out_dir`.
///
/// Slices written:
/// - `boundary.csv`           — guard epochs just inside/outside the target range
/// - `interior.csv`           — per-body cadence backbone across 1900-2100 CE
/// - `fast_clusters.csv`      — fine-cadence windows for fast bodies
/// - `holdout.csv`            — deterministic pseudo-random hold-out epochs
/// - `manifest.txt`           — provenance manifest for all seven slices (four
///   backend-generated + `fixture_golden` + two asteroid slices)
///
/// `fixture_golden.csv`, `asteroid_reference.csv`, and
/// `asteroid_constrained.csv` must already exist in `out_dir` before this
/// command is run; they are committed inputs, not backend-generated.
///
/// Full emit behavior (against a real de440 kernel) is covered by the
/// `pleiades-jpl` slice tests (`generate_slice`/`build_manifest`) plus the
/// gated maintainer regeneration step. The unit tests here verify arg wiring
/// and the missing-out-dir error surface; they do not require a real kernel.
fn emit_slices(backend: &SpkBackend, out_dir: &str) -> Result<String, String> {
    let roles = [
        SliceRole::Boundary,
        SliceRole::InteriorBackbone,
        SliceRole::FastCluster,
        SliceRole::Holdout,
    ];
    let mut generated = Vec::new();
    for role in roles {
        let slice = generate_slice(backend, role)?;
        std::fs::write(format!("{out_dir}/{}", slice.file), &slice.csv)
            .map_err(|e| format!("write {}: {e}", slice.file))?;
        generated.push(slice);
    }
    let mut manifest = build_manifest(&generated);
    let fg_entry = fixture_golden_manifest_entry(out_dir)?;
    manifest.slices.push(fg_entry);
    let ar_entry = corpus_csv_manifest_entry(
        out_dir,
        "asteroid_reference",
        "asteroid_reference.csv",
        "asteroid_reference",
    )?;
    manifest.slices.push(ar_entry);
    let ac_entry = corpus_csv_manifest_entry(
        out_dir,
        "asteroid_constrained",
        "asteroid_constrained.csv",
        "asteroid_constrained",
    )?;
    manifest.slices.push(ac_entry);
    std::fs::write(format!("{out_dir}/manifest.txt"), manifest.render())
        .map_err(|e| format!("write manifest: {e}"))?;
    Ok(format!(
        "wrote {} slices + manifest (incl. fixture_golden, asteroid_reference, asteroid_constrained) to {out_dir}",
        generated.len() + 3
    ))
}

/// Reads `{out_dir}/{file}`, counts its data rows (non-comment, non-empty
/// lines), and computes its checksum, returning a [`SliceEntry`] ready to
/// append to the manifest.
///
/// Returns an actionable error if the file is absent or unreadable, because
/// committed-input slices (fixture_golden, asteroid_reference,
/// asteroid_constrained) must be present before `--emit-slices` is run.
fn corpus_csv_manifest_entry(
    out_dir: &str,
    name: &str,
    file: &str,
    role: &str,
) -> Result<SliceEntry, String> {
    let path = format!("{out_dir}/{file}");
    let csv = std::fs::read_to_string(&path).map_err(|_| {
        format!(
            "--emit-slices requires {path} to exist; \
             {name} is not backend-generated and must be present before regenerating"
        )
    })?;
    let rows = csv
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .count();
    Ok(SliceEntry {
        name: name.to_string(),
        file: file.to_string(),
        role: role.to_string(),
        rows,
        checksum: corpus_checksum64(&csv),
    })
}

/// Reads `{out_dir}/fixture_golden.csv`, counts its data rows (non-comment,
/// non-empty lines), and computes its checksum, returning a [`SliceEntry`]
/// ready to append to the manifest.
///
/// Returns an actionable error if the file is absent or unreadable, because
/// `fixture_golden` is hand-populated from trusted Horizons fixtures and must
/// be present before `--emit-slices` is run.
fn fixture_golden_manifest_entry(out_dir: &str) -> Result<SliceEntry, String> {
    corpus_csv_manifest_entry(out_dir, "fixture_golden", "fixture_golden.csv", "fixture_golden")
        .map_err(|_| {
            let fg_path = format!("{out_dir}/fixture_golden.csv");
            format!(
                "--emit-slices requires {fg_path} to exist (populate it from the trusted \
                 Horizons fixtures before regenerating); fixture_golden is not backend-generated"
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_kernel_path_errors() {
        assert!(render_spk_corpus(&[]).is_err());
    }

    #[test]
    fn missing_epochs_errors() {
        // A nonexistent path still fails before epoch parsing only if present;
        // here we pass a path but no JDs.
        let err = render_spk_corpus(&["/no/such/kernel.bsp"]).unwrap_err();
        // Either the kernel load fails or the "requires at least one JD" check;
        // both are acceptable error surfaces.
        assert!(!err.is_empty());
    }

    // --emit-slices arg-wiring tests.
    //
    // `pleiades_jpl::spk::test_support` (the synthetic DAF builder) is
    // `pub(crate)` in pleiades-jpl and cannot be reached from pleiades-cli
    // without widening its visibility, which we deliberately avoid.  These
    // tests therefore verify argument parsing and the error surface rather
    // than full slice output.  Full emit behaviour (generating real CSV rows
    // from a synthetic in-memory kernel) is covered by the `slice_tests`
    // module in `pleiades_jpl::spk::generate`.

    #[test]
    fn emit_slices_missing_out_dir_errors() {
        // --emit-slices present but no out-dir argument: must fail with
        // the specific missing-directory message.
        let err = render_spk_corpus(&["k.bsp", "--emit-slices"]).unwrap_err();
        assert!(
            err.contains("--emit-slices requires an output directory"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn emit_slices_bad_kernel_errors_before_writing() {
        // Kernel load fails for a non-existent path, so we never reach the
        // filesystem write step.  This proves the arg wiring reaches the
        // backend-build step.
        let err = render_spk_corpus(&["/no/such/kernel.bsp", "--emit-slices", "/tmp/whatever"])
            .unwrap_err();
        assert!(
            !err.is_empty(),
            "expected a non-empty error from bad kernel"
        );
    }

    // fixture_golden_manifest_entry tests — no kernel required.

    #[test]
    fn fixture_golden_entry_missing_file_errors_with_actionable_message() {
        // A tempdir with no fixture_golden.csv must yield an actionable Err.
        let dir = std::env::temp_dir().join(format!(
            "pleiades_test_missing_fg_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .subsec_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let dir_str = dir.to_string_lossy();
        let err = fixture_golden_manifest_entry(&dir_str).unwrap_err();
        assert!(
            err.contains("fixture_golden is not backend-generated"),
            "error should mention fixture_golden is not backend-generated: {err}"
        );
        assert!(
            err.contains("fixture_golden.csv"),
            "error should name the missing file: {err}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn fixture_golden_entry_present_file_returns_correct_rows_and_nonzero_checksum() {
        // A tempdir containing fixture_golden.csv with 2 data rows + 1 comment.
        let dir = std::env::temp_dir().join(format!(
            "pleiades_test_fg_entry_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .subsec_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let fg_path = dir.join("fixture_golden.csv");
        std::fs::write(
            &fg_path,
            "# header comment\n2451545.0,Sun,1.0,2.0,3.0\n2451546.0,Moon,4.0,5.0,6.0\n",
        )
        .unwrap();
        let dir_str = dir.to_string_lossy();
        let entry = fixture_golden_manifest_entry(&dir_str).unwrap();
        assert_eq!(entry.rows, 2, "should count 2 data rows, not the comment");
        assert_ne!(entry.checksum, 0, "checksum should be non-zero");
        assert_eq!(entry.name, "fixture_golden");
        assert_eq!(entry.file, "fixture_golden.csv");
        assert_eq!(entry.role, "fixture_golden");
        let _ = std::fs::remove_dir_all(&dir);
    }

    // corpus_csv_manifest_entry tests — asteroid slices, no kernel required.

    #[test]
    fn corpus_csv_entry_missing_file_errors_with_actionable_message() {
        // A tempdir with no asteroid CSV must yield an actionable Err naming
        // the missing file and noting it is not backend-generated.
        let dir = std::env::temp_dir().join(format!(
            "pleiades_test_missing_ar_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .subsec_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let dir_str = dir.to_string_lossy();
        let err = corpus_csv_manifest_entry(
            &dir_str,
            "asteroid_reference",
            "asteroid_reference.csv",
            "asteroid_reference",
        )
        .unwrap_err();
        assert!(
            err.contains("asteroid_reference"),
            "error should name the slice: {err}"
        );
        assert!(
            err.contains("not backend-generated"),
            "error should say not backend-generated: {err}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn corpus_csv_entry_asteroid_slices_return_correct_rows_checksums_and_names() {
        // A tempdir with both asteroid CSVs; verify row counts, checksums, and
        // SliceEntry fields for both asteroid_reference and asteroid_constrained.
        let dir = std::env::temp_dir().join(format!(
            "pleiades_test_asteroid_slices_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .subsec_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();

        // asteroid_reference: 3 data rows + 2 comment lines
        let ar_path = dir.join("asteroid_reference.csv");
        std::fs::write(
            &ar_path,
            "# comment one\n# comment two\n2451545.0,Ceres,1.0,2.0,3.0\n\
             2451546.0,Vesta,4.0,5.0,6.0\n2451547.0,Pallas,7.0,8.0,9.0\n",
        )
        .unwrap();

        // asteroid_constrained: 1 data row + 1 comment line
        let ac_path = dir.join("asteroid_constrained.csv");
        std::fs::write(
            &ac_path,
            "# header\n2451548.0,Hygiea,10.0,11.0,12.0\n",
        )
        .unwrap();

        let dir_str = dir.to_string_lossy();

        let ar_entry = corpus_csv_manifest_entry(
            &dir_str,
            "asteroid_reference",
            "asteroid_reference.csv",
            "asteroid_reference",
        )
        .unwrap();
        assert_eq!(ar_entry.rows, 3, "should count 3 data rows");
        assert_ne!(ar_entry.checksum, 0, "checksum should be non-zero");
        assert_eq!(ar_entry.name, "asteroid_reference");
        assert_eq!(ar_entry.file, "asteroid_reference.csv");
        assert_eq!(ar_entry.role, "asteroid_reference");

        let ac_entry = corpus_csv_manifest_entry(
            &dir_str,
            "asteroid_constrained",
            "asteroid_constrained.csv",
            "asteroid_constrained",
        )
        .unwrap();
        assert_eq!(ac_entry.rows, 1, "should count 1 data row");
        assert_ne!(ac_entry.checksum, 0, "checksum should be non-zero");
        assert_eq!(ac_entry.name, "asteroid_constrained");
        assert_eq!(ac_entry.file, "asteroid_constrained.csv");
        assert_eq!(ac_entry.role, "asteroid_constrained");

        // Checksums must differ between the two files.
        assert_ne!(
            ar_entry.checksum, ac_entry.checksum,
            "distinct files must have distinct checksums"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }
}
