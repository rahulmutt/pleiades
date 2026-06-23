# House-System Numeric Gate (Phase 5, Sub-cycle A) — Design

Status: **final — 2026-06-23**. Core design approved by the user. The two
network-blocked provisioning open items were resolved against a live Nix/devenv
environment (devenv 2.1.2): the SE Rust binding options and Astrolog's nixpkgs
status are now confirmed (see "Provisioning findings" and the resolved open
items). Ready to hand to writing-plans.

## Provisioning findings (2026-06-23, verified against live Nix)

- **SE Rust binding.** Two viable crates exist on crates.io: high-level
  `swisseph` 0.1.1 (depends on `libswisseph-sys`, `links = libswisseph`) and
  lower-level `libswe-sys` 0.2.7 (`links = libswe`). Both pull a `links`/`-sys`
  package into `Cargo.lock`, which **confirms Constraint C1**: the SE harness
  must live outside the published-workspace lockfile. Neither crate declares a
  license in its index metadata, and Swiss Ephemeris itself is dual-licensed
  AGPL-3.0 / commercial — a manual license check is required before adoption
  (verification-only, non-shipping use mitigates but does not eliminate this).
- **Astrolog.** Packaged in nixpkgs as `astrolog` 7.70 (GPL-2.0-or-later), but
  the **stock binary crashes on every invocation** in the current gcc-15 /
  glibc-2.42 environment — even `-v`: default build aborts with a fortify
  *buffer overflow*; with fortify disabled it aborts with a *stack smashing*
  detection; with all hardening disabled it segfaults silently. Astrolog 7.70
  has genuine out-of-bounds writes that modern hardening catches. A headless
  cusp-emitter therefore requires a **patched derivation** (hardening overrides
  and/or source patches), and even then must be verified to emit cusps. "Add
  `pkgs.astrolog` to `devenv.nix`" is **not** turn-key.

These findings drive the resolved decisions below: SE remains the sole canonical
gate, and the Astrolog cross-check is **best-effort / optional** so the gate
never depends on a fragile second engine.

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
| Reference source | **Swiss Ephemeris is the sole canonical gate.** **Astrolog** is a **best-effort / optional** independent cross-check (records agreement when a working astrolog is available; the gate never depends on it). |
| Disagreement handling | Astrolog **flags** disagreements (recorded, investigated per-system); it does **not** auto-fail the gate. When no working astrolog is provisioned, the corpus records cross-check = **"not run"** and the gate still passes on SE alone. |
| High-latitude failure modes | **Strict by default** (structured `InvalidLatitude` error beyond a documented bound); **SE-compat fallback opt-in** behind a request flag |
| SE provisioning | Use a **Rust Swiss Ephemeris binding** (`swisseph` 0.1.1 high-level, preferred, or `libswe-sys` 0.2.7) for verification only — **never** a dependency of any shipping crate (see Constraint C1). License check required before adoption. |
| Astrolog provisioning | Provide via **`devenv.nix`** (Nix). Stock `pkgs.astrolog` 7.70 crashes under modern hardening, so a **patched derivation** is required (verified to emit cusps). Because the cross-check is best-effort, a failed/absent astrolog never blocks the gate. |

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
checksums **and** source-engine provenance: Swiss Ephemeris version (always) and,
when a working astrolog was used, its version + pinned git SHA — exactly as the
JPL manifest records the de440 kernel SHA. If astrolog was not run, the manifest
records the cross-check as `not-run`.

- **Swiss Ephemeris values are the canonical reference** stored in the corpus.
- A **best-effort cross-check record** documents Astrolog agreement per system:
  `agree` (within cross-tolerance), `flagged` (measured delta + note), or
  `not-run` (no working astrolog was provisioned at generation time). Per the
  disagreement decision, Astrolog flags but does not gate, and `not-run` is a
  valid, non-failing state.
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
  in an isolated verification harness — see Constraint C1. Preferred crate is
  high-level `swisseph` 0.1.1 (exposes house computation; pulls `libswisseph-sys`
  with `links = libswisseph`); `libswe-sys` 0.2.7 (`links = libswe`) is the
  lower-level fallback. Either confirms the C1 isolation requirement. Produces SE
  cusps via `swe_houses`-equivalent calls.
- **Astrolog (best-effort cross-check):** provided via **`devenv.nix`**.
  `astrolog` 7.70 is in nixpkgs (GPL-2.0-or-later) but the stock build crashes on
  every invocation under modern hardening (verified: fortify buffer-overflow →
  stack-smashing → segfault), so `devenv.nix` must supply a **patched
  derivation** (e.g. `hardeningDisable` plus any source fixes) verified to emit
  machine-readable cusps for all 11 systems. Version + pinned revision recorded
  in the corpus manifest. Because the cross-check is best-effort, if a working
  astrolog cannot be built the generator proceeds SE-only and marks the
  cross-check `not-run`.
- Generation is a maintainer-only, regeneration-time step. Like the de440 kernel
  (`PLEIADES_DE_KERNEL`) and the `horizons-fetch` feature, the engines are **not**
  runtime or shipping-crate dependencies; the committed corpus is the source of
  truth.

## Data flow

```
fixtures
  └─(offline, engines present)─> generate SE cusps [+ Astrolog cusps if available]
        ├─ cross-check SE vs Astrolog when present (flag exceptions, never gate;
        │   mark cross-check `not-run` if no working astrolog)
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
- **Generation fails** only on SE-side problems. When Astrolog is present and
  disagrees beyond cross-tolerance on a non-exempted system, generation records a
  flagged exception (forces a documented exception decision) but does not fail on
  the cross-check alone. When no working astrolog is provisioned, generation
  proceeds SE-only with cross-check `not-run` — never a failure.

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

- **C2 — SE binding license (verify before adoption).** Neither `swisseph` nor
  `libswe-sys` declares a license in its crates.io index metadata, and Swiss
  Ephemeris itself is dual-licensed AGPL-3.0 / commercial. Because the binding is
  verification-only and never shipped or distributed, AGPL obligations are
  unlikely to attach — but the chosen crate's actual license (and SE's data-file
  license) must be confirmed and recorded before the harness is committed.

## Testing

- Per-system unit tests vs. a few inline goldens.
- Integration test: the `validate-houses` gate over the committed corpus.
- Strict-rejection path tests (beyond-bound latitudes).
- SE-fallback path tests.
- Manifest checksum-drift test.

## Resolved open items (2026-06-23)

1. **SE Rust binding — RESOLVED to options + a license action.** `swisseph`
   0.1.1 (high-level, preferred) and `libswe-sys` 0.2.7 both exist; both pull a
   `links`/`-sys` package and so confirm C1. Remaining action is the **license
   verification in C2** and confirming the chosen crate's SE data-file
   loading/bundling, then isolating it per C1.
2. **Astrolog in nixpkgs — RESOLVED with a caveat.** `astrolog` 7.70 is packaged
   (GPL-2.0-or-later) but crashes on every invocation under modern hardening, so
   `devenv.nix` must supply a **patched derivation** verified to emit
   machine-readable cusps for all 11 systems. Because the cross-check is now
   best-effort, this is no longer a blocker for the gate.

## Open items (confirm during planning/implementation)

1. **Patched-astrolog derivation.** Produce a working `devenv.nix` astrolog
   (hardening overrides and/or source patches), and confirm its CLI emits
   machine-readable cusps for all 11 systems. If infeasible, ship the corpus
   SE-only with cross-check `not-run` (the design already permits this).
2. **SE high-latitude fallback semantics.** Verify what Swiss Ephemeris actually
   substitutes above the polar circle (Porphyry assumed) before encoding the
   SE-compat path.
3. **Corpus location.** `pleiades-houses/data/` vs. `pleiades-validate/` — pick
   to match where the gate and fixtures most naturally live.
4. **Per-family arcsecond ceilings.** Concrete numeric ceilings per formula
   family, set from observed SE-vs-pleiades residuals.
