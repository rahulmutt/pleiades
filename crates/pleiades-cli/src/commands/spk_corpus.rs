//! `generate-spk-corpus` command: sample a DE kernel into the corpus CSV.

use pleiades_core::CelestialBody;
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
/// - `boundary.csv`      — guard epochs just inside/outside the target range
/// - `interior.csv`      — per-body cadence backbone across 1600-2600 CE
/// - `fast_clusters.csv` — fine-cadence windows for fast bodies
/// - `holdout.csv`       — deterministic pseudo-random hold-out epochs
/// - `manifest.txt`      — provenance manifest for all four slices
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
    let manifest = build_manifest(&generated);
    std::fs::write(format!("{out_dir}/manifest.txt"), manifest.render())
        .map_err(|e| format!("write manifest: {e}"))?;
    Ok(format!(
        "wrote {} slices + manifest to {out_dir}",
        generated.len()
    ))
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
}
