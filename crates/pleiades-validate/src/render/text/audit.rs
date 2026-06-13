//! Workspace/native dependency audit summaries, benchmark backend, and checksums.

use crate::*;

/// Renders a compact workspace audit summary used by the CLI and release bundle.
pub fn render_workspace_audit_summary() -> Result<String, std::io::Error> {
    let report = workspace_audit_report()?;
    Ok(render_workspace_audit_summary_text(&report))
}

/// Renders the compact native-dependency audit summary used by release bundling.
///
/// This stays explicit even though it currently shares the same underlying report,
/// so release-bundle bookkeeping can keep the native-dependency path separate.
pub fn render_native_dependency_audit_summary() -> Result<String, std::io::Error> {
    render_workspace_audit_summary()
}

/// Benchmarks a backend against a validation corpus.
pub fn benchmark_backend(
    backend: &dyn EphemerisBackend,
    corpus: &ValidationCorpus,
    rounds: usize,
) -> Result<BenchmarkReport, EphemerisError> {
    let single_start = StdInstant::now();
    for _ in 0..rounds {
        for request in &corpus.requests {
            std::hint::black_box(backend.position(request)?);
        }
    }
    let elapsed = single_start.elapsed();

    let batch_start = StdInstant::now();
    for _ in 0..rounds {
        std::hint::black_box(backend.positions(&corpus.requests)?);
    }
    let batch_elapsed = batch_start.elapsed();

    let report = BenchmarkReport {
        backend: backend.metadata(),
        corpus_name: corpus.name.clone(),
        apparentness: corpus.apparentness,
        rounds,
        sample_count: corpus.requests.len(),
        elapsed,
        batch_elapsed,
        estimated_corpus_heap_bytes: corpus.estimated_heap_bytes(),
    };
    report.validate()?;
    Ok(report)
}

/// Computes a deterministic 64-bit checksum for bundle text.
pub(crate) fn checksum64(text: &str) -> u64 {
    checksum64_bytes(text.as_bytes())
}

/// Computes a deterministic 64-bit checksum for arbitrary bytes.
pub(crate) fn checksum64_bytes(bytes: &[u8]) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0000_0001_0000_01b3;

    let mut hash = FNV_OFFSET_BASIS;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}
