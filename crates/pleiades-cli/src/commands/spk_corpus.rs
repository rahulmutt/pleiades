//! `generate-spk-corpus` command: sample a DE kernel into the corpus CSV.

use pleiades_core::CelestialBody;
use pleiades_jpl::{generate_corpus_csv, CorpusRequest, SpkBackend};

/// Parses args of the form: `generate-spk-corpus <kernel.bsp> <jd1> [jd2 ...]`
/// and prints the corpus CSV. Bodies default to the Sun-through-Pluto set.
pub fn render_spk_corpus(args: &[&str]) -> Result<String, String> {
    let kernel = args
        .first()
        .ok_or("generate-spk-corpus requires a kernel path")?;
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
}
