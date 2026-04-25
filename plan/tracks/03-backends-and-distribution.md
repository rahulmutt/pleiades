# Track 3 — Backends and Distribution

## Role

Guide the remaining backend, compression, and packaged-data work so Pleiades can provide interchangeable pure-Rust ephemeris sources and offline artifacts.

## Standards

- Each backend family stays in its own crate.
- Backend metadata must state source material, time range, body coverage, frames, topocentric/apparent/mean support, determinism, offline status, and accuracy class.
- Unsupported bodies, frames, time scales, missing data, and out-of-range requests must produce structured errors.
- Composite routing must remain transparent through metadata and result backend identifiers.
- Packaged artifacts must state stored, derived, and unsupported outputs.

## Remaining backend goals

- Implement production `pleiades-vsop87` planetary calculations.
- Implement production `pleiades-elp` lunar calculations.
- Upgrade `pleiades-jpl` to a source-backed reference reader/interpolator.
- Build deterministic compressed artifacts for 1500-2500 CE.
- Benchmark lookup latency, batch throughput, artifact size, and corpus heap footprint estimates.

## Distribution constraints

- No mandatory C/C++ dependencies or native runtime libraries.
- Large data should be feature-gated or packaged deliberately.
- Generated artifacts must be reproducible from documented public inputs.
