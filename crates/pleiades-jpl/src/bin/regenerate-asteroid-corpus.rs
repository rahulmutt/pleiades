//! Maintainer tool: regenerate the Tier A `asteroid_reference` corpus slice from
//! de440 + sb441-n16, and report the manifest line to record for it.
//!
//! Tier A (pinned-kernel) bodies only — the centaurs/TNOs (Tier B) are sourced
//! separately via Horizons (see `regenerate-asteroid-constrained`). This binary
//! is kernel-gated: it reads the two kernels from the environment and writes the
//! committed slice file, so a clean checkout stays kernel-free.
//!
//! Usage:
//!   PLEIADES_DE_KERNEL=/path/de440.bsp \
//!   PLEIADES_AST_KERNEL=/path/sb441-n16.bsp \
//!     cargo run -p pleiades-jpl --bin regenerate-asteroid-corpus
//!
//! It writes `crates/pleiades-jpl/data/corpus/asteroid_reference.csv` (relative
//! to the workspace root) and prints the `slice asteroid_reference …` manifest
//! line (rows + checksum) to paste into `data/corpus/manifest.txt`.

use pleiades_backend::EphemerisBackend;
use pleiades_jpl::spk::asteroid_roster::{asteroid_core_roster, AsteroidTier};
use pleiades_jpl::spk::corpus_manifest::corpus_checksum64;
use pleiades_jpl::spk::corpus_spec::SliceRole;
use pleiades_jpl::{generate_slice, SpkBackend};

const OUT_PATH: &str = "crates/pleiades-jpl/data/corpus/asteroid_reference.csv";

fn main() -> Result<(), String> {
    let de = std::env::var("PLEIADES_DE_KERNEL")
        .map_err(|_| "set PLEIADES_DE_KERNEL to the de440.bsp path".to_string())?;
    let ast = std::env::var("PLEIADES_AST_KERNEL")
        .map_err(|_| "set PLEIADES_AST_KERNEL to the sb441-n16.bsp path".to_string())?;

    let backend = SpkBackend::builder()
        .add_kernel(&de)
        .map_err(|e| e.message)?
        .add_kernel(&ast)
        .map_err(|e| e.message)?
        .build();

    // Report Tier A coverage so missing kernel ids are visible, not silent.
    for entry in asteroid_core_roster()
        .iter()
        .filter(|e| e.tier == AsteroidTier::PinnedKernel)
    {
        eprintln!(
            "Tier A {:<28} supported_by_kernel={}",
            format!("{}", entry.body),
            backend.supports_body(entry.body.clone())
        );
    }

    let slice = generate_slice(&backend, SliceRole::AsteroidReference)?;
    let rows = slice
        .csv
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .count();
    if rows == 0 {
        return Err(
            "asteroid_reference slice is empty — no Tier A bodies resolved \
                    from the kernels; check NAIF ids / kernel coverage"
                .to_string(),
        );
    }
    let checksum = corpus_checksum64(&slice.csv);

    std::fs::write(OUT_PATH, &slice.csv).map_err(|e| format!("write {OUT_PATH}: {e}"))?;

    eprintln!("\nwrote {OUT_PATH}: {rows} rows, checksum={checksum}");
    println!(
        "slice asteroid_reference file=asteroid_reference.csv role=asteroid_reference rows={rows} checksum={checksum}"
    );
    Ok(())
}
