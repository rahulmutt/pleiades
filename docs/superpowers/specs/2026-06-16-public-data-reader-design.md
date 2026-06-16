# Broad Public-Data Reader for External JPL-Style Inputs — Design

- **Status:** Approved design, ready for implementation planning
- **Date:** 2026-06-16
- **Phase:** PLAN.md Phase 1 closeout (production reference backend and corpus)
- **Crate:** `pleiades-jpl` (+ thin `pleiades-validate` CLI surface)

## Goal

Close the remaining Phase 1 implementation item: a reader that ingests
**arbitrary external JPL-style data products** — beyond the pinned de440 kernel
and the checked-in canonical fixtures — and normalizes them into the existing
`SnapshotCorpus` typed structure so they can feed corpus generation, validation,
and comparison.

This unblocks corpus regeneration/extension from public inputs (not just the
pinned kernel and committed fixtures) and gives downstream consumers a way to
load their own reference data.

## Scope decisions (from brainstorming)

| Decision | Choice |
| --- | --- |
| Input categories | Multiple, via a detection layer: Horizons vector-table text, Horizons API JSON, and generic JPL-style CSV. |
| Consumer | Both: library entry points are the core deliverable; a thin `pleiades-validate` subcommand exercises the maintainer workflow. |
| Network | Live Horizons fetch **in scope**, but quarantined behind a default-off feature and a trait seam. |
| Fetch isolation | One crate; `HorizonsSource` trait seam; default-off `horizons-fetch` Cargo feature. |
| Normalization posture | Fail-closed on contradiction and unsupported modes; a caller-supplied `ExpectedProfile` fills **only** genuine silences, recorded as provenance. |
| Internal organization | Shared intermediate representation (IR) + one normalizer (front-ends only tokenize). |

## Non-goals

- Apparent-place, topocentric, observer-table, or native-sidereal ingestion —
  these are rejected as `Unsupported`, consistent with the backend boundary.
- UTC/UT1 acceptance at the ingestion boundary — only TT/TDB, matching the
  existing time-scale policy.
- Wiring live fetch into the default CLI binary in this phase (library + opt-in
  test only; a `--fetch` flag is a clearly-scoped follow-up).
- Asteroid SPK kernel adoption (separate Phase 1 item, tracked independently).

## Architecture

New module tree inside `pleiades-jpl`, reusing `SnapshotCorpus`,
`SnapshotManifest`, `SnapshotEntry`, and the existing validation/comparison
consumers:

```
crates/pleiades-jpl/src/ingest/
  mod.rs            # public API: read_public_corpus(...), detect_format(...), ExpectedProfile, types
  ir.rs             # RawCorpus, RawManifest, RawEphemerisRecord — the format-neutral IR
  detect.rs         # sniff bytes -> InputFormat (vector-table | horizons-json | generic-csv)
  normalize.rs      # the one fail-closed normalizer: IR -> SnapshotCorpus + IngestProvenance
  format/
    vector_table.rs # Horizons $$SOE/$$EOE text front-end -> IR
    horizons_json.rs# Horizons API JSON front-end -> IR (delegates row parsing to vector_table)
    generic_csv.rs  # tolerant CSV front-end (col aliasing/reorder/units) -> IR
  fetch.rs          # #[cfg(feature="horizons-fetch")] HorizonsSource trait + HTTP impl
  error.rs          # IngestError taxonomy
```

One-directional data flow; network strictly at the edge:

```
bytes ──detect()──> InputFormat ──front-end──> RawCorpus
                                                   │
                               ExpectedProfile ───►│ normalize() (fail-closed)
                                                   ▼
                                    SnapshotCorpus + IngestProvenance
```

The `fetch` module is the only thing the `horizons-fetch` feature compiles; it
produces `bytes`, then re-enters the same offline path. Nothing downstream of
`detect()` knows whether bytes came from disk or the wire.

## Intermediate representation (`ir.rs`)

The IR captures *what the source said* without interpreting it. Everything is
`Option`/string at this layer — front-ends never default or convert.

```rust
pub struct RawCorpus {
    pub declared: RawManifest,         // strings lifted verbatim from source headers
    pub records: Vec<RawEphemerisRecord>,
}

pub struct RawManifest {
    pub source_label: Option<String>,  // e.g. "JPL Horizons API v1.2"
    pub center: Option<String>,        // e.g. "500@0" / "Solar System Barycenter"
    pub frame: Option<String>,         // e.g. "Ecliptic of J2000.0" / "ICRF"
    pub time_scale: Option<String>,    // e.g. "TDB" / "CT"
    pub units: Option<String>,         // e.g. "KM-S" / "AU-D"
    pub columns: Vec<String>,          // raw column order as seen
}

pub struct RawEphemerisRecord {
    pub body_label: String,            // raw target name/id as seen
    pub epoch_jd: f64,                 // numeric JD as seen (scale still a string in declared)
    pub pos: [f64; 3],                 // raw x,y,z in source units (no conversion yet)
    pub vel: Option<[f64; 3]>,         // captured if present; dropped on normalize (entries are position-only)
}
```

## Normalizer (`normalize.rs`)

The single fail-closed gate.

```rust
pub fn normalize(raw: RawCorpus, expected: &ExpectedProfile)
    -> Result<(SnapshotCorpus, IngestProvenance), IngestError>;
```

Ordered checks, each producing a structured error on failure:

1. **Frame** — map `declared.frame` → supported frame enum. Unknown/unsupported
   (apparent/topocentric markers) → `Unsupported`. Silent → take
   `expected.frame`, record as `Asserted`.
2. **Time scale** — `declared.time_scale` → `TimeScale`. UTC/UT1 markers →
   `Unsupported` (boundary accepts only TT/TDB). Silent → `expected`, recorded
   `Asserted`.
3. **Units** — `KM` vs `AU`, `S` vs `D`. Drives the AU→km conversion factor
   (`AU_IN_KM = 149_597_870.7`). Silent → `expected`.
4. **Center/origin** — must resolve to a supported center; mismatch with
   `expected.center` → `Contradiction`.
5. **Per record** — map `body_label` → `CelestialBody` via the body taxonomy +
   alias table; non-finite/NaN/inf → `MalformedRow`; convert `pos` to km; build
   `Instant { julian_day, scale }`; assemble `SnapshotEntry`.
6. **Contradiction rule** — whenever `declared.X` is present *and* `expected.X`
   is present and they disagree → hard `Contradiction` (never silently prefer
   one).

`ExpectedProfile` is all-optional; the maintainer asserts only what is needed.
`IngestProvenance` records, per attribute, whether the value was `Read` (from
source) or `Asserted` (filled from expected), so the resulting corpus/manifest
carries honest provenance into the existing reports.

```rust
pub struct ExpectedProfile {
    pub frame: Option<CoordinateFrame>,    // existing type, reused
    pub time_scale: Option<TimeScale>,     // existing type, reused
    pub units: Option<Units>,              // new small enum: Km | Au
    pub center: Option<Center>,            // new small enum of supported centers (e.g. SSB)
}
```

Type note: `CoordinateFrame` and `TimeScale` are existing workspace types and
are reused as-is. `Units` and `Center` are new, small ingestion-local enums.
Throughout this doc, bare `Frame` in prose refers to `CoordinateFrame`.

## Detector (`detect.rs`)

Cheap, deterministic byte-sniffing; no full parse.

```rust
pub enum InputFormat { HorizonsVectorTable, HorizonsApiJson, GenericCsv }

pub fn detect_format(bytes: &[u8]) -> Result<InputFormat, IngestError>;
```

Sniff order (first match wins):

1. Leading non-whitespace `{` **and** contains `"signature"`/`"result"` keys →
   `HorizonsApiJson`.
2. Contains `$$SOE` … `$$EOE` markers → `HorizonsVectorTable`.
3. Parses as delimited rows with a recognizable header → `GenericCsv`.
4. None → `UnrecognizedFormat` (lists what it looked for).

Callers can bypass detection and name the format explicitly (for terse CSV that
could be ambiguous).

## Format front-ends (`format/*.rs`)

Each is *only* a tokenizer: bytes → `RawCorpus`. No semantic decisions, no
defaulting, no unit conversion.

- **`vector_table.rs`** — parses the Horizons text block: header fields
  (`Center body`, `Reference frame`, `Output units`, target name) into
  `RawManifest`, then the `$$SOE`/`$$EOE` rows (JD, calendar tag,
  X/Y/Z[, VX/VY/VZ]).
- **`horizons_json.rs`** — parses the API envelope; the ephemeris lives in the
  `result` text payload (same `$$SOE` block), so this front-end extracts
  header/result then **delegates row parsing to `vector_table.rs`**.
  JSON-specific metadata (signature/version) maps to `source_label`. Uses a
  minimal pure-Rust JSON read (a small hand-rolled extractor or a lightweight
  no-default-features dependency — finalized in the plan, kept off the
  offline-core's required deps).
- **`generic_csv.rs`** — builds on the existing `parse_snapshot_manifest` header
  convention plus a **column-alias table** (`x`/`X`/`x_km`/`PX` → position-x,
  etc.) so reordered/renamed columns resolve. Units/frame/time come from `#`
  header comments or are left silent for `ExpectedProfile` to fill.

Reuse point: the JSON path does not duplicate row logic — it funnels into the
vector-table row reader. There is one row tokenizer for the two Horizons shapes
plus the CSV tokenizer.

## Live-fetch seam (`fetch.rs`)

Gated behind a default-off `horizons-fetch` Cargo feature. With the feature off,
the crate has zero HTTP dependencies and `fetch.rs` compiles to nothing.

```rust
#[cfg(feature = "horizons-fetch")]
pub trait HorizonsSource {
    /// Returns the raw bytes of a Horizons product for the given query.
    fn fetch(&self, query: &HorizonsQuery) -> Result<Vec<u8>, IngestError>;
}

#[cfg(feature = "horizons-fetch")]
pub struct HorizonsQuery {
    pub command: String,        // target body id
    pub center: String,         // e.g. "500@0"
    pub start: f64, pub stop: f64, pub step: String,
    pub format: HorizonsWireFormat, // Json | Text
}
```

- **Real implementation** `HttpHorizonsSource` — wraps a small blocking HTTP
  client (`ureq` without TLS bloat, or `reqwest` blocking; final pick recorded
  in the plan), builds the `ssd.jpl.nasa.gov/api/horizons.api` URL from
  `HorizonsQuery`, returns bytes. It does **not** parse — bytes flow into
  `detect()` → front-end → `normalize()`, the identical offline path.
- **Provenance** — a fetch stamps `IngestProvenance` with the request URL and
  wire format, and records the live kernel SHA / `signature` reported by
  Horizons so the corpus can pin what it pulled (mirrors the existing
  kernel-SHA-pinning discipline).
- **Determinism guard** — the gated fetch path is excluded from the default test
  run; ingestion tests always run offline against saved fixture bytes injected
  through a test `HorizonsSource` impl. A separate, explicitly gated integration
  test (network, opt-in, like the existing `corpus_regen` kernel gate) can
  exercise the real endpoint.

## Public library API

Re-exported from the `pleiades-jpl` lib root, mirroring how `SpkBackend` /
`build_manifest` are surfaced.

```rust
// Offline core — always compiled, no network:
pub fn detect_format(bytes: &[u8]) -> Result<InputFormat, IngestError>;

pub fn read_public_corpus(
    bytes: &[u8],
    expected: &ExpectedProfile,            // all-optional asserts
) -> Result<PublicCorpus, IngestError>;    // auto-detects format

pub fn read_public_corpus_as(
    bytes: &[u8],
    format: InputFormat,                   // bypass detection
    expected: &ExpectedProfile,
) -> Result<PublicCorpus, IngestError>;

pub struct PublicCorpus {
    pub corpus: SnapshotCorpus,            // the existing typed structure
    pub provenance: IngestProvenance,      // Read vs Asserted per attribute, + source labels
}

// Path convenience (mirrors load_snapshot_corpus_from_paths):
pub fn read_public_corpus_from_path(path, expected) -> Result<PublicCorpus, IngestError>;

// Network — only under feature = "horizons-fetch":
pub fn fetch_public_corpus<S: HorizonsSource>(
    source: &S, query: &HorizonsQuery, expected: &ExpectedProfile,
) -> Result<PublicCorpus, IngestError>;
```

`PublicCorpus.corpus` plugs directly into the existing
validation/comparison/manifest builders — no consumer changes needed.

## CLI surface (`pleiades-validate`)

One new subcommand via the `render_cli` string dispatch, fail-closed like its
siblings:

```
pleiades-validate ingest-public --input <path> [--format vector-table|horizons-json|generic-csv]
                  [--expect-frame …] [--expect-time-scale …] [--expect-units …] [--expect-center …]
```

It reads the file, runs `read_public_corpus[_as]`, and renders a deterministic
summary: detected format, row/body/epoch counts, and the **provenance table**
(which attributes were Read vs Asserted), plus any structured error. Network is
intentionally **not** wired into the CLI in this phase — the maintainer fetch
workflow stays a library/opt-in-test concern, keeping the default binary
network-free.

## Error taxonomy (`error.rs`)

One structured `IngestError` enum, implementing `std::error::Error` + `Display`,
every variant naming exactly what failed and where.

```rust
pub enum IngestError {
    // --- detection ---
    UnrecognizedFormat { looked_for: Vec<&'static str> },

    // --- front-end tokenizing ---
    Malformed { format: InputFormat, line: usize, detail: String },
    MissingMarker { format: InputFormat, marker: &'static str },   // e.g. $$EOE absent
    ColumnUnresolved { column: String, format: InputFormat },      // generic-csv alias miss

    // --- normalization (the fail-closed gate) ---
    Unsupported { attribute: Attribute, value: String },           // apparent/topocentric/UTC/bad center
    Contradiction { attribute: Attribute, declared: String, expected: String },
    Undetermined { attribute: Attribute },                         // silent in source AND in ExpectedProfile
    UnknownBody { label: String },
    MalformedRow { epoch_jd: f64, detail: String },                // non-finite, etc.

    // --- fetch (feature = "horizons-fetch") ---
    Fetch { detail: String },                                      // network/HTTP/status
}

pub enum Attribute { Frame, TimeScale, Units, Center }
```

Two rules make the posture predictable:

- **`Undetermined` vs `Asserted`** — an attribute silent in the source is only
  an error if `ExpectedProfile` *also* leaves it silent. If asserted, it is
  `Read`-vs-`Asserted` provenance, not an error.
- **`Contradiction` is never auto-resolved** — declared ≠ expected always
  errors; the maintainer reconciles by fixing the input or the assertion.

This mirrors the existing `validate-corpus` gate's "fail closed on missing
bodies, epochs, channels, schema drift, malformed rows" language.

## Testing strategy

All deterministic and offline by default; network is the only opt-in.

**Unit tests, per layer** (the IR seam makes each layer testable in isolation):

- **Detector** — minimal byte snippets per format + ambiguous/garbage → asserts
  `InputFormat` or `UnrecognizedFormat`.
- **Front-ends** — small saved fixtures (a real-shaped Horizons vector block, an
  API JSON envelope, a reordered/aliased generic CSV) → assert exact `RawCorpus`
  (raw strings/numbers, no interpretation). JSON test asserts it delegates to
  the same row reader.
- **Normalizer** — the densest suite, driven by hand-built `RawCorpus` values
  (no parsing needed): one test per ordered check — frame
  map/unsupported/silent-asserted, time-scale TT/TDB-ok vs UTC-rejected, AU→km
  conversion exactness, center mismatch, contradiction-never-resolved, unknown
  body, non-finite row, and `Read`-vs-`Asserted` provenance correctness.

**Integration tests:**

- **End-to-end offline** — fixture bytes → `read_public_corpus` → assert
  resulting `SnapshotCorpus` equals an expected typed corpus, and that it feeds
  the existing manifest/comparison builders without error.
- **Round-trip anchor** — ingest a small slice whose values overlap the
  committed de440 corpus and assert agreement within the corpus's existing km
  tolerance (proves the AU→km/frame mapping matches the real pipeline).
- **Fetch (opt-in, gated)** — a test `HorizonsSource` returning saved bytes
  verifies the fetch→detect→normalize wiring under `--features horizons-fetch`;
  a separate `#[ignore]`/env-gated test (like `corpus_regen`) hits the live
  endpoint.

**Fixtures** — saved under `crates/pleiades-jpl/tests/fixtures/ingest/`, each a
real-shaped but small public sample, checked in as the deterministic source of
truth.

**Workspace gates** — the offline ingest tests run in the default `cargo test`;
the existing audit/fmt/clippy gates cover the new module; the `horizons-fetch`
feature gets a `--features` build check so it cannot bit-rot.

## Plan/documentation impact

On completion:

- Update `plan/stages/01-production-reference-corpus.md` to mark the
  arbitrary-external-input reader as **Met**, leaving only asteroid-kernel
  adoption open.
- Refresh `PLAN.md` "Important current limits" wording that calls out the
  not-yet-a-broad-public-reader gap.
- Note the new `horizons-fetch` feature and `ingest-public` subcommand where
  CLI/feature surfaces are documented.
