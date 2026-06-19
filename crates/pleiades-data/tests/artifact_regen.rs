//! Gated: regenerates the packaged compressed artifact from the real de440
//! kernel and asserts byte-identity against the committed fixture. Skipped
//! unless PLEIADES_DE_KERNEL points at de440.bsp.
//!
//! Also contains a non-gated size/perf measurement test that prints the
//! SP1 draft baseline numbers (artifact byte size, decode latency, lookup
//! latency) without enforcing any threshold (SP1 measures; SP3 budgets).

#[test]
fn sp1_draft_size_perf_baseline() {
    use pleiades_backend::{CelestialBody, Instant, JulianDay, TimeScale};
    use pleiades_compression::CompressedArtifact;
    use std::time::Instant as StdInstant;

    let bytes = pleiades_data::packaged_artifact_bytes();
    let artifact_size = bytes.len();

    // Decode benchmark: 3 rounds, average.
    let decode_rounds = 3usize;
    let start = StdInstant::now();
    for _ in 0..decode_rounds {
        let _ = CompressedArtifact::decode(bytes).expect("decode should succeed");
    }
    let decode_elapsed = start.elapsed();
    let decode_ms = decode_elapsed.as_secs_f64() * 1_000.0 / decode_rounds as f64;

    // Lookup benchmark: 100 rounds of Sun at J2000, average.
    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let lookup_rounds = 100usize;
    let start = StdInstant::now();
    for _ in 0..lookup_rounds {
        let _ = pleiades_data::packaged_lookup(&CelestialBody::Sun, instant)
            .expect("Sun lookup should succeed");
    }
    let lookup_elapsed = start.elapsed();
    let lookup_us = lookup_elapsed.as_secs_f64() * 1_000_000.0 / lookup_rounds as f64;

    eprintln!(
        "SP1 draft size/perf baseline (unoptimized, informational only):\n  artifact_size={} bytes\n  decode={:.1} ms (avg over {} rounds)\n  lookup={:.1} µs (avg over {} lookups, Sun@J2000, TT)",
        artifact_size, decode_ms, decode_rounds, lookup_us, lookup_rounds
    );

    // No threshold assertions — SP1 measures, SP3 budgets.
    assert!(artifact_size > 0, "artifact must be non-empty");
    assert!(decode_ms > 0.0, "decode time must be positive");
    assert!(lookup_us > 0.0, "lookup time must be positive");
}

#[test]
fn regenerated_artifact_matches_committed() {
    let Ok(kernel) = std::env::var("PLEIADES_DE_KERNEL") else {
        eprintln!("skipping: set PLEIADES_DE_KERNEL to run");
        return;
    };

    let regenerated = pleiades_data::regenerate_packaged_artifact_from_kernel(&kernel)
        .expect("artifact regeneration from de440 kernel should succeed");

    let regenerated_bytes = regenerated
        .encode()
        .expect("re-encoded artifact should encode cleanly");

    let committed_bytes = pleiades_data::packaged_artifact_bytes();

    assert_eq!(
        regenerated_bytes.len(),
        committed_bytes.len(),
        "re-encoded artifact byte length ({}) differs from committed ({}) — not byte-identical",
        regenerated_bytes.len(),
        committed_bytes.len(),
    );

    assert_eq!(
        regenerated_bytes,
        committed_bytes,
        "re-encoded artifact is not byte-identical to the committed fixture (lengths match but content differs)",
    );

    eprintln!(
        "artifact_regen: PASS — byte-identical ({} bytes)",
        committed_bytes.len()
    );
}
