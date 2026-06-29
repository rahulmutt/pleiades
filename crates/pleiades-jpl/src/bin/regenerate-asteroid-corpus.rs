//! Maintainer tool: regenerate the Tier A `asteroid_reference` corpus slice from
//! de440 + sb441-n373s, report the manifest line to record for it, and filter
//! promoted Tier-A bodies out of the committed `asteroid_constrained.csv`.
//!
//! Tier A (pinned-kernel) bodies only — the centaurs/TNOs (Tier B) are sourced
//! separately via Horizons (see `regenerate-asteroid-constrained`). This binary
//! is kernel-gated: it reads the two kernels from the environment and writes the
//! committed slice file, so a clean checkout stays kernel-free.
//!
//! Usage:
//!   PLEIADES_DE_KERNEL=/path/de440.bsp \
//!   PLEIADES_AST_KERNEL=/path/sb441-n373s.bsp \
//!     cargo run -p pleiades-jpl --bin regenerate-asteroid-corpus
//!
//! It writes `crates/pleiades-jpl/data/corpus/asteroid_reference.csv` (relative
//! to the workspace root) and prints the `slice asteroid_reference …` and
//! `slice asteroid_constrained …` manifest lines (rows + checksum) to paste into
//! `data/corpus/manifest.txt`.

use pleiades_backend::EphemerisBackend;
use pleiades_jpl::spk::asteroid_roster::{asteroid_core_roster, tier_b_bodies, AsteroidTier};
use pleiades_jpl::spk::corpus_manifest::corpus_checksum64;
use pleiades_jpl::spk::corpus_spec::SliceRole;
use pleiades_jpl::{generate_slice, SpkBackend};

const OUT_PATH: &str = "crates/pleiades-jpl/data/corpus/asteroid_reference.csv";
const CONSTRAINED_PATH: &str = "crates/pleiades-jpl/data/corpus/asteroid_constrained.csv";

fn main() -> Result<(), String> {
    let de = std::env::var("PLEIADES_DE_KERNEL")
        .map_err(|_| "set PLEIADES_DE_KERNEL to the de440.bsp path".to_string())?;
    let ast = std::env::var("PLEIADES_AST_KERNEL")
        .map_err(|_| "set PLEIADES_AST_KERNEL to the sb441-n373s.bsp path".to_string())?;

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

    // Drop now-Tier-A bodies from the committed constrained slice without
    // re-fetching Horizons: keep header lines and rows whose body id is still
    // Tier-B, so remaining rows stay byte-identical.
    let tier_b: std::collections::HashSet<String> =
        tier_b_bodies().iter().map(|b| format!("{b}")).collect();
    let existing = std::fs::read_to_string(CONSTRAINED_PATH)
        .map_err(|e| format!("read {CONSTRAINED_PATH}: {e}"))?;
    let mut out = String::new();
    let mut kept_rows = 0usize;
    for line in existing.lines() {
        if line.starts_with('#') || line.is_empty() {
            out.push_str(line);
            out.push('\n');
            continue;
        }
        let body = line.split(',').nth(1).unwrap_or_default();
        if tier_b.contains(body) {
            out.push_str(line);
            out.push('\n');
            kept_rows += 1;
        }
    }
    std::fs::write(CONSTRAINED_PATH, &out)
        .map_err(|e| format!("write {CONSTRAINED_PATH}: {e}"))?;
    let c_checksum = corpus_checksum64(&out);
    eprintln!("\nfiltered {CONSTRAINED_PATH}: {kept_rows} rows, checksum={c_checksum}");
    println!(
        "slice asteroid_constrained file=asteroid_constrained.csv role=asteroid_constrained rows={kept_rows} checksum={c_checksum}"
    );
    Ok(())
}
