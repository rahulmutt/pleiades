# Pure-Rust SPK Reference Backend — Design

**Date:** 2026-06-13
**Status:** Approved design, pending implementation plan
**Phase:** 1 — Production reference backend and corpus (active frontier)

## Summary

Add a pure-Rust reader for JPL DE binary **SPK kernels** (`.bsp`) to
`pleiades-jpl`, exposed from a single engine in two roles:

1. a **runtime backend** that answers ephemeris requests from a user-supplied
   kernel set, and
2. the **generation engine** that samples a kernel to emit the broad,
   provenance-stamped reference/hold-out/boundary corpus that unblocks Phase 1.

This replaces the hand-curated checked-in Horizons CSV fixtures as the *source*
of reference truth with a reproducible-from-public-input pipeline, while keeping
those CSVs as trusted cross-check golden values.

## Goals

- Make the reference corpus **reproducible from a public input**: anyone with a
  documented DE kernel plus the generation command reproduces the corpus.
- Provide a high-confidence reference backend for validating the VSOP87/ELP
  backends and feeding the compressed-artifact fitter.
- Stay pure Rust with no mandatory FFI; preserve `#![forbid(unsafe_code)]`.
- Keep all public claims truthful to the loaded kernel's actual coverage.

## Non-goals (held to existing deferrals)

- No apparent-place / light-time / aberration reduction (Phase 4). Output stays
  **mean geometric**, consistent with all current first-party backends; apparent
  requests remain structured rejections.
- No topocentric / observer reduction (Phase 4).
- No UTC/UT1/Delta-T conversion inside the backend — TT/TDB only, matching the
  existing time/observer policy. SPK is natively TDB, so this is a clean fit.
- No de441 / full 1500 CE floor in this slice — recorded as an explicit known
  gap (see Coverage).
- No bundling of kernels into published crates — kernels stay user-supplied at
  runtime and contributor-fetched at generation time.

## Decisions

| Decision | Choice | Rationale |
| --- | --- | --- |
| Reference source | Pure-Rust SPK (DE binary) reader | Fully self-contained, no gen-time network, satisfies the `pleiades-jpl` "may parse official/public data files in Rust" mandate. |
| Reader role | Runtime backend **and** generation engine | One decoder serves both high-accuracy runtime queries and corpus generation. |
| Body/segment scope | Planetary + asteroid kernels | Covers all release-claimed bodies (majors + Pluto + Moon + selected asteroids) in one slice. Requires SPK types 2, 3, 1, 21. |
| Kernel target | de440, dynamic coverage detection | ~114 MB fetch (vs ~3 GB de441). Backend advertises the kernel's real window; claim narrowed to ~1550–2650 with the 1500 floor logged as a known gap. |
| Test strategy | Synthetic fixtures + existing-CSV cross-check | Deterministic decoder unit tests in CI without a large file; real-number confidence via the trusted Horizons CSVs; opt-in gated full-kernel integration test. |
| Output semantics | Mean geometric, geocentric ecliptic, TT/TDB | Consistent with existing backends and with what the corpus stores. |

## Architecture

### Module layout — new tree under `pleiades-jpl/src/spk/`

- `spk/daf.rs` — DAF container parsing: file record, endianness marker
  (`LTL-IEEE` / `BIG-IEEE`), comment area, summary/name record traversal →
  list of segment descriptors.
- `spk/segment/mod.rs` with `type2.rs`, `type3.rs`, `type1.rs`, `type21.rs` —
  per-type decoders. Type 2 (Chebyshev position) and Type 3 (position +
  velocity) cover DE planetary kernels; Type 1 (modified difference arrays) and
  Type 21 (extended modified difference arrays) cover small-body kernels.
- `spk/pool.rs` — kernel pool: load N kernels, merge segment descriptors into a
  `(target, center) → segment` routing table, detect actual coverage windows
  dynamically.
- `spk/chain.rs` — body→geocenter resolution: walk target→SSB and Earth→SSB
  chains, difference them, apply ICRF (J2000 equatorial) → ecliptic rotation
  using the same obliquity constant the existing backend already uses so the
  SPK and fixture paths agree on frame. (Frame bias ICRF↔dynamical-J2000 is at
  the milliarcsecond level; documented and applied or explicitly noted.)
- `spk/backend.rs` — the backend trait impl plus a capability matrix derived
  from the loaded kernels.

This mirrors the crate's existing modularization style (small focused
submodules, as in `reference_summary/…`).

### Runtime backend API

A builder that takes one or more kernel paths:

```rust
let backend = SpkBackend::builder()
    .add_kernel("de440.bsp")?          // planetary
    .add_kernel("ast343de440.bsp")?    // selected asteroids
    .build()?;
```

`build()` reads each kernel's segment descriptors, validates the endianness and
segment types it understands, and constructs the routing table. The capability
matrix — supported bodies, **actual** date window per body, frames, mean/
geometric handling, error class, and "requires external kernel file" — is
computed from what was actually loaded; nothing is hardcoded. Unsupported
segment types or missing body chains surface as structured errors, never silent
gaps.

File access is `std::fs` seek-based: only the records a query needs are read, so
a 114 MB kernel does not have to be resident in memory and no mmap/`unsafe` is
required.

## Data flow — corpus generation (Phase 1 unblock)

The SPK backend becomes the source the corpus generator samples:

1. Load the kernel pool (de440 + an asteroid kernel), fetched by **documented
   URL + SHA-256**, never committed.
2. Sample each release-claimed body at the corpus's defined epochs/cadence,
   preserving the existing **reference / fitting / hold-out / boundary /
   fixture-exactness / provenance-only** separation.
3. Emit the *same* CSV/snapshot schema the crate already parses, so all
   downstream report, backend-matrix, and bundle machinery is unchanged — only
   the corpus's breadth and provenance improve.
4. Stamp provenance: kernel identity (de440), segment source revision, SHA-256,
   generation command, frame, time scale, and obliquity constant.

## Testing

- **Synthetic SPK fixtures** committed to the repo: a tiny hand-crafted DAF
  holding one small Type 2 segment and one Type 21 segment with known
  coefficients → deterministic unit tests for `daf.rs` and each segment decoder
  (endianness, record math, Chebyshev / MDA evaluation), running in CI with no
  large file.
- **Numerical cross-check**: the SPK backend's geocentric ecliptic output is
  compared against the existing trusted Horizons CSV fixtures at their epochs,
  within documented tolerances, tying the decoders to real-world-correct values.
- **Gated full-kernel integration test**: opt-in (env var / kernel-present
  check), skipped in normal CI, runs the reader against the real de440 for
  end-to-end confidence locally and in a dedicated job.
- **TDD order**: DAF parse → segment decoders (synthetic fixtures) →
  chaining/frame reduction (CSV cross-check) → backend/capability.

## Coverage and release-gate wiring

- The backend's dynamically-detected coverage flows into the capability matrix
  and the release/compatibility profiles.
- The **narrowed claim** (~1550–2650 from de440) is recorded truthfully; the
  **1500 CE floor is logged as an explicit known gap** to revisit with de441.
- A release gate fails if a profile claims a body/date the loaded-kernel
  evidence does not actually support — satisfying the Phase 1 checklist items on
  validated coverage and drift detection.

## Phase-1 checklist items advanced

- Production source strategy implemented and pure-Rust compatible.
- Public source provenance, schema, frame, time scale, source revision,
  generation command, and checksums recorded.
- Reference / fitting / hold-out / boundary / fixture-exactness / provenance-only
  evidence kept separable (schema unchanged).
- Corpus validation fails on body/epoch/channel/frame/checksum/source-revision
  drift.

## Open items for the implementation plan

- Exact obliquity / frame-bias treatment and the constant shared with the
  existing backend path.
- Precise asteroid kernel choice and which selected asteroids it must cover to
  match current claims.
- Tolerance values for the CSV cross-check per body class.
- Generation command surface (which CLI / `pleiades-validate` entry point owns
  it) and how the fetched-kernel checksum is pinned.
