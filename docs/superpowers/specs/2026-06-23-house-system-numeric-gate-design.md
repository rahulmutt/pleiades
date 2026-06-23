# House-System Numeric Gate (Phase 5, Sub-cycle A) — Design

Status: **draft — saved mid-brainstorm 2026-06-23**. Core design approved by the
user; provisioning details (SE Rust binding, Astrolog via Nix) captured from the
latest direction but carry open items to confirm at implementation time. Not yet
handed to writing-plans.

## Context

Phase 4's only remaining item — native sidereal backend output — is a deliberate
non-goal (capability flag exists, composite backends AND it, fail-closed policy
summary enforces "unsupported"). The genuine next frontier is **Phase 5:
Compatibility and Release Gates**.

The first Phase 5 item is the **house-system audit**. The 11 baseline house
systems are already implemented (`pleiades-houses/src/systems/mod.rs`), each with
a `HouseSystemDescriptor` (canonical name, aliases, formula-family tag,
`latitude_sensitive` flag) and a descriptor `validate()`. The compatibility
profile already *reports* counts, aliases, and latitude constraints. So this is
an **audit**, not a build: the systems exist; the work is producing evidence that
their release claims are trustworthy.

The audit was scoped as **two sub-cycles**:

- **Sub-cycle A (this spec):** numeric correctness gate + strict/SE-compat
  high-latitude behavior.
- **Sub-cycle B (later spec):** metadata/provenance descriptor audit (formula
  docs, aliases, source-label mappings, SE-interop notes).

## Decisions captured

| Topic | Decision |
| --- | --- |
| Deliverable for the full item | Both numeric gate + metadata audit (B deferred to its own cycle) |
| Reference source | Multiple engines cross-check: **Swiss Ephemeris canonical**, **Astrolog** independent cross-check |
| Disagreement handling | Astrolog **flags** disagreements (recorded, investigated per-system); it does **not** auto-fail the gate. SE is canonical reference. |
| High-latitude failure modes | **Strict by default** (structured `InvalidLatitude` error beyond a documented bound); **SE-compat fallback opt-in** behind a request flag |
| SE provisioning | Use a **Rust Swiss Ephemeris binding** for verification only — **never** a dependency of any shipping crate (see Constraint C1) |
| Astrolog provisioning | Provide via **`devenv.nix`** (Nix) for reproducible local builds |

## Scope & boundaries

**In:** numeric correctness gate for the **11 baseline house systems**
(P Placidus, K Koch, O Porphyry, R Regiomontanus, C Campanus, A/E Equal-from-Asc,
W Whole Sign, B Alcabitius, X Meridian/Axial, T Polich-Page/Topocentric,
M Morinus), plus the strict-default / SE-compat-opt-in high-latitude behavior.

**Out (explicitly):**

- The 12 target-only systems (D, N, V, U, Y, S, F, H, G, L, Q, I) — Phase 6.
- The metadata/provenance descriptor audit — Sub-cycle B.
- Any new public house systems or public-API redesign.

## Architecture

### 1. Reference corpus (mirrors the de440 JPL corpus)

New `crates/pleiades-houses/data/corpus/` (location TBD vs. `pleiades-validate/`;
see open items) with CSV fixture slices + a `manifest.txt` carrying per-slice
checksums **and** source-engine provenance: Swiss Ephemeris version and Astrolog
version + pinned git SHA — exactly as the JPL manifest records the de440 kernel
SHA.

- **Swiss Ephemeris values are the canonical reference** stored in the corpus.
- A **cross-check record** documents Astrolog agreement per system: agree-within-
  cross-tolerance, or a flagged exception carrying the measured delta and a note.
  Per the disagreement decision, Astrolog flags but does not gate.
- Each row: chart id, instant, observer (lat/lon/elev), house-system code,
  12 cusps + Ascendant + Midheaven.
- Reproducible from the engines when present; a clean checkout stays tool-free
  and only **validates** committed values (de440 precedent).

### 2. Fixtures

- Latitudes: `0°`, `~40°`, `~55°`, `~66.0°` (just inside the polar circle) for
  the numeric **in-band** path; `~70°`, `~80°` (above the polar circle) for the
  **strict-rejection** path.
- A couple of longitudes × 2 epochs (distinct sidereal times).
- Checked-in, checksum-pinned.

### 3. Strict-default latitude behavior

`calculate_houses` returns `HouseError { kind: InvalidLatitude }` for
latitude-sensitive systems (Placidus, Koch, Topocentric) beyond a **documented
latitude bound** added to the descriptor — never garbage cusps. The gate asserts
the rejection fires. `HouseErrorKind::InvalidLatitude` already exists.

### 4. SE-compat opt-in

A policy field on `HouseRequest` (e.g.
`high_latitude_policy: Strict | SwissEphemerisFallback`, default `Strict`). Under
`SwissEphemerisFallback`, beyond-bound systems reproduce Swiss Ephemeris's
documented fallback (verify: typically Porphyry substitution) instead of
erroring, with a provenance record of the substitution. The gate validates the
fallback output against the SE reference. Mirrors the codebase's established
"strict default, opt-in correction" pattern (apparent default, topocentric
opt-in).

### 5. The gate: `validate-houses` (alias `houses-gate`)

Fail-closed command in `pleiades-validate`, wired into the `release-gate`
aggregate, mirroring `validate-apparent` / `validate-topocentric`. Checks:

- corpus checksum / schema / provenance drift;
- completeness (all baseline systems × fixtures present);
- numeric residuals vs **per-formula-family arcsecond ceilings** — tight for
  space-division systems (Equal/Whole/Porphyry), looser for iterative ones
  (Placidus/Koch/Alcabitius/Topocentric), mid for equatorial-projection. Ceilings
  live in a `thresholds` module mirroring `pleiades-data/src/thresholds.rs`;
- strict-rejection assertions (beyond-bound → `InvalidLatitude`);
- SE-fallback assertions (opt-in path matches SE reference).

### 6. Reproduction tooling & engine provisioning

- **Swiss Ephemeris (canonical):** a Rust Swiss Ephemeris binding used **only**
  in an isolated verification harness — see Constraint C1. Produces SE cusps via
  `swe_houses`-equivalent calls.
- **Astrolog (cross-check):** provided via **`devenv.nix`**. Astrolog ships no
  Linux release binary (only Windows assets + `astXXcli.zip` source on
  `CruiserOne/Astrolog`), so Nix builds it reproducibly from a pinned version.
  Version + pinned revision recorded in the corpus manifest.
- Generation is a maintainer-only, regeneration-time step. Like the de440 kernel
  (`PLEIADES_DE_KERNEL`) and the `horizons-fetch` feature, the engines are **not**
  runtime or shipping-crate dependencies; the committed corpus is the source of
  truth.

## Data flow

```
fixtures
  └─(offline, engines present)─> generate SE cusps + Astrolog cusps
        ├─ cross-check SE vs Astrolog (flag exceptions, never gate)
        └─ write corpus CSV + manifest (checksums + engine versions/SHAs)
                └─> committed corpus (source of truth)
                        └─(runtime gate: validate-houses)─>
                              recompute pleiades cusps per fixture
                              ├─ compare to SE reference within per-family ceiling
                              ├─ assert strict rejection beyond latitude bound
                              └─ assert SE-fallback path matches SE reference
                                    └─> pass / fail (fail-closed)
```

## Error handling / fail-closed conditions

- **Gate fails** on: missing slice, checksum/schema/provenance drift, residual
  over ceiling, missing strict rejection, or SE-fallback mismatch.
- **Generation fails** if SE and Astrolog disagree beyond cross-tolerance on a
  non-exempted system (forces a documented exception decision).

## Constraints

- **C1 — Pure-Rust workspace audit (hard).** `workspace-audit`
  (`pleiades-validate/src/release/workspace_audit.rs`) fails closed on `links`
  assignments, `-sys` dependencies, `build.rs` scripts, and **lockfile packages
  ending in `-sys`** (only `cc`/`ring`/`windows-sys` are exempt phantoms). A Rust
  Swiss Ephemeris binding is FFI over libswe with a build script and will appear
  as a `-sys` package in `Cargo.lock`. Therefore the SE verification harness
  **must not** be a member of the published workspace and **must not** enter the
  workspace `Cargo.lock`. Options to resolve in the plan: a separate, excluded
  verification crate with its own isolated lockfile outside the workspace, or an
  out-of-band verification script invoked only during regeneration. The shipping
  crates and their lockfile stay pure-Rust.

## Testing

- Per-system unit tests vs. a few inline goldens.
- Integration test: the `validate-houses` gate over the committed corpus.
- Strict-rejection path tests (beyond-bound latitudes).
- SE-fallback path tests.
- Manifest checksum-drift test.

## Open items (confirm during planning/implementation)

1. **Exact SE Rust binding.** crates.io probes for `swisseph` / `libswe-sys`
   were inconclusive in the brainstorm environment (no network metadata
   returned). Confirm the specific crate, its license, that it exposes house
   computation, and how it bundles/loads SE data files — then isolate it per C1.
2. **Astrolog in nixpkgs.** Confirm `astrolog` is packaged (or pin a source
   build in `devenv.nix`) and capture the exact version/revision for the
   manifest. Confirm its CLI emits machine-readable cusps for all 11 systems.
3. **SE high-latitude fallback semantics.** Verify what Swiss Ephemeris actually
   substitutes above the polar circle (Porphyry assumed) before encoding the
   SE-compat path.
4. **Corpus location.** `pleiades-houses/data/` vs. `pleiades-validate/` — pick
   to match where the gate and fixtures most naturally live.
5. **Per-family arcsecond ceilings.** Concrete numeric ceilings per formula
   family, set from observed SE-vs-pleiades residuals.
