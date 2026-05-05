# Track 3 — Backends and Distribution

## Role

Guide remaining backend, compression, and packaged-data work so Pleiades provides interchangeable pure-Rust ephemeris sources and offline artifacts with truthful capability claims.

## Standards

- Keep each backend family in its own first-party crate.
- Keep backend metadata current for source material, time range, body coverage, frames, time scales, topocentric/apparent/mean support, determinism, offline status, and accuracy class.
- Return structured errors for unsupported bodies, frames, time scales, missing data, out-of-range dates, apparent requests, and observer-bearing geocentric-only requests.
- Keep composite routing transparent through metadata, capability summaries, and result backend identifiers.
- Make packaged artifacts state stored, derived, and unsupported outputs and enforce those claims at decode/lookup time.

## Remaining backend goals

- Resolve Pluto's release posture: source-backed within thresholds or explicitly approximate/excluded from release-grade claims.
- Expand JPL/reference data from fixture evidence into a production-suitable validation and artifact-generation corpus.
- Decide whether the lunar backend remains a compact validated baseline or grows into a fuller ELP-style coefficient implementation for the first release.
- Build deterministic compressed artifacts for 1500-2500 CE with measured error inside published thresholds.
- Benchmark lookup latency, batch throughput, artifact decode cost, artifact size, and full-chart use.

## Distribution constraints

- No mandatory C/C++ dependencies, build scripts, or native runtime libraries.
- Large data should be feature-gated or packaged deliberately.
- Generated artifacts must be reproducible from documented public inputs and versioned parameters.
- Prototype artifacts must stay labeled as prototype until they meet production thresholds.
