# Production Reference Corpus Definition — Design

**Date:** 2026-06-15
**Status:** Approved design, pending implementation plan
**Phase:** 1 — Production reference backend and corpus (active frontier)

## Summary

Turn the ad-hoc `generate-spk-corpus` output into a **defined, reproducible,
validated production reference corpus** that meets Phase 1's exit criteria and
can feed the Phase 2 compressed-artifact generator and hold-out comparisons.

The corpus is a **bounded, stratified, checked-in sample** of the SPK reference
truth, plus a **gated verify-from-kernel** path that reproduces it. The Phase 2
fitter samples the `SpkBackend` densely at generation time directly — the
checked-in corpus is the kernel-free regression + hold-out evidence, not the
fitter's dense input.

The hard SPK infrastructure (pure-Rust DAF/segment reader, `SpkBackend` runtime
+ generation engine, `generate_corpus_csv`/`CorpusRequest`,
`generate-spk-corpus` CLI) already exists and is merged; this design defines the
corpus those pieces produce and the validation that gates it.

## Goals

- Define the corpus precisely: epoch grid/cadence across 1600-2600 CE, body +
  channel set, and reference / fitting / hold-out / boundary / fixture-exactness
  / provenance-only stratification.
- Make it **reproducible from a public input**: a documented DE kernel plus a
  generation command regenerates and verifies the corpus.
- Make validation **fail-closed** on coverage, schema, and provenance/drift, in
  normal kernel-free CI.
- Keep the existing CSV schema and all downstream report/backend-matrix/bundle
  machinery unchanged — only breadth, structure, and provenance improve.

## Non-goals (held to existing deferrals)

- No apparent-place / light-time / aberration; output stays **mean geometric**
  (Phase 4).
- No topocentric / observer reduction (Phase 4).
- No UTC/UT1/Delta-T conversion; epochs are TDB, matching the SPK backend and
  the time/observer policy.
- No velocity/motion channels; position only (motion output is deferred Phase 4,
  so storing it would precede its consumer).
- No lunar points (mean/true node, apsides): these are analytically derived by
  `pleiades-elp`, not present in SPK kernels, so they are outside an SPK-sourced
  corpus.
- No de441 / pre-1550 floor; the de440 coverage window and its known gap remain
  as already recorded.
- No bundling of kernels into published crates; kernels stay user-supplied at
  runtime and contributor-fetched at generation time.

## Scope

- **Bodies:** Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus,
  Neptune, Pluto, and selected asteroids (asteroid kernel).
- **Channels:** geocentric ecliptic position as Cartesian `x/y/z` km
  (reconstructs longitude/latitude/distance), mean geometric, TDB.
- **Claim status:** Pluto and selected asteroids are carried but tagged
  constrained/approximate and excluded from release-grade tolerance evidence,
  consistent with the existing `release_grade_corpus`.

## Decisions

| Decision | Choice | Rationale |
| --- | --- | --- |
| Storage model | Checked-in bounded sample + gated verify-from-kernel | Keeps git size sane and CI kernel-free while preserving full reproducibility under a gated job. |
| Epoch strategy | Stratified by purpose, body-speed-scaled | A hold-out/regression corpus is most useful when it deliberately stresses boundaries and high-curvature bodies, not uniform sampling. |
| Body/channel scope | All release-claimed bodies, position channels | Matches current backend output exactly; no second-consumer (motion) data stored ahead of need. |
| Validation gate | Full completeness matrix + drift, fail-closed in CI | Satisfies the Phase 1 exit criterion and matches the repo's fail-closed posture. |
| Source of truth | One `corpus-spec.toml` drives both generation and validation | Prevents drift between "what must exist" and "what was generated." |
| Schema | Reuse existing `epoch_jd,body,x_km,y_km,z_km` CSV | Downstream report/backend-matrix/bundle machinery stays untouched; rows round-trip through `parse_snapshot_entries`. |

## Architecture

### Single source of truth: `corpus-spec.toml`

A checked-in config drives **both** generation and validation so the completeness
matrix and the generated data cannot diverge. It declares:

- the **epoch grid** (slice definitions below) and the **per-body cadence**
  (max-gap) table;
- the **release-claimed body × channel × frame** completeness matrix;
- **per-body-class tolerances** for cross-checks;
- kernel identity + pinned SHA-256(s), frame, time scale, and obliquity constant.

### Stratified slices

Each slice is a separate file tagged with its role, preserving the existing
reference / fitting / hold-out / boundary / fixture-exactness / provenance-only
separation:

| Slice file | Role | Contents |
| --- | --- | --- |
| `boundary.csv` | boundary | Guard epochs near 1600 & 2600 CE (just inside + just outside the target span), all bodies — catches edge interpolation error. |
| `interior.csv` | reference backbone | Coarse grid across the range, **cadence scaled to body speed** (outer planets sparse, inner bodies denser). |
| `fast_clusters.csv` | reference (high-curvature) | Short fine-cadence windows for Moon/Mercury/Venus (e.g. Moon daily over ~1 month at several anchor dates spread across the range). |
| `holdout.csv` | independent hold-out | Deterministic pseudo-random epochs (documented seed), **disjoint from any fitting epochs**, for unbiased artifact error estimation. |
| `fixture_golden.csv` | fixture-exactness | The existing trusted JPL Horizons values, kept as exact pinned regression anchors. |

Target footprint for the checked-in sample: low thousands of rows (~100 KB). The
dense corpus produced by the verify-from-kernel path may be far larger and is
regenerated on demand, never committed.

### Storage layout

```
crates/pleiades-jpl/data/corpus/
  boundary.csv  interior.csv  fast_clusters.csv  holdout.csv  fixture_golden.csv
  manifest.toml      # per-file content checksum, row count, role, generation command/args
  corpus-spec.toml   # the source-of-truth config above
```

Provenance is stamped in both the CSV `#` header lines and `manifest.toml`:
kernel identity (de440), **real pinned SHA-256** (computed from the kernel,
replacing today's `<run shasum…>` placeholder), segment source revision,
generation command + args, frame, time scale, obliquity constant, and the
asteroid kernel identity when used.

## Data flow — generation and verification

- Extend `pleiades-jpl` with a higher-level `generate_corpus` over the existing
  `CorpusRequest` / `generate_corpus_csv`, driven by `corpus-spec.toml`, that
  emits **all slices + manifest** in one run and computes the real kernel
  SHA-256.
- Promote the `generate-spk-corpus` CLI from "kernel + hand-typed Julian Days"
  to "kernel + spec file → full slice set + manifest."
- Add a **gated verify path** (active when `PLEIADES_DE_KERNEL` is present):
  regenerate each slice from the kernel using the same spec and compare to the
  checked-in values within a tight reproducibility tolerance; verify the kernel
  SHA-256 matches the manifest.

## Validation gate — kernel-free CI, fail-closed

A new `pleiades-validate` corpus module + CLI/release-gate wiring loads the
checked-in slices and enforces, as **hard failures**:

- **Completeness matrix:** every release-claimed body × required epoch-class
  (boundary-low, boundary-high, interior backbone spanning the range, hold-out)
  × channel (x/y/z) present; frame == ecliptic; apparentness == mean.
- **Schema conformance:** header, parseability, finite values.
- **Provenance / drift:** each file's content checksum matches the manifest;
  kernel SHA-256 is non-placeholder; source revision present.
- **Cross-check:** `fixture_golden` values within per-body-class tolerances.
- **Claim status:** Pluto and selected asteroids must be present but flagged
  constrained/approximate and excluded from release-grade tolerance evidence.

The gated kernel job additionally runs the regenerate-and-compare from the data
flow above. Any gap is a hard error.

## Testing (TDD order)

1. `corpus-spec.toml` parse + completeness-matrix derivation.
2. Validation gate fails on each gap class (missing body / epoch-class / channel
   / frame / checksum / placeholder SHA / schema drift) — table-driven.
3. Generation emits spec-conformant slices + manifest from a synthetic DAF (as
   the existing `generate.rs` tests do).
4. Round-trip through `parse_snapshot_entries`; cross-check tolerances against
   `fixture_golden`.
5. Gated full-kernel regenerate-and-verify (opt-in via `PLEIADES_DE_KERNEL`).

## Phase-1 exit-criteria mapping

- *A clean checkout can reproduce or verify the reference inputs from documented
  public sources* → generation + gated verify with a pinned SHA-256.
- *Corpus validation fails on missing bodies, epochs, channels, frames,
  apparentness policy, schema drift, or checksum/source-revision drift* →
  fail-closed completeness matrix.
- *Release-claimed body/channel/frame/date coverage is broad enough to feed the
  production artifact generator and hold-out comparisons* → stratified slices
  with a disjoint hold-out; the dense fitting input is drawn from `SpkBackend`
  directly at Phase 2 generation time.

## Open items for the implementation plan

- Concrete per-body cadence (max-gap) numbers and the resulting checked-in row
  count budget.
- Exact `corpus-spec.toml` schema and the hold-out RNG seed/algorithm.
- Per-body-class cross-check tolerance values.
- Asteroid kernel choice and which selected asteroids it must cover to match
  current claims (carries over from the SPK backend design's open item).
- Whether the gated verify lives in `generate-spk-corpus`, a new
  `verify-spk-corpus` surface, or `pleiades-validate`.
