# Broaden Selected-Asteroid Source Coverage via Small-Body Kernel — Design

- **Status:** Approved design, ready for implementation planning
- **Date:** 2026-06-17
- **Phase:** PLAN.md Phase 1 closeout (production reference backend and corpus)
- **Crate:** `pleiades-jpl` (+ provenance docs; thin reference-summary/report surfaces)

## Goal

Close the last remaining Phase 1 implementation item:

> Adopt a small-body asteroid SPK kernel to broaden selected-asteroid source
> coverage (record its provenance in `docs/spk-kernel-sourcing.md` when added).
> — `plan/stages/01-production-reference-corpus.md:36`, `PLAN.md:49`

Today the major bodies are sourced reproducibly from the pinned de440 kernel,
but asteroid rows in the corpus are **fixture-sourced** (`data/selected_asteroid_*.rs`
plus hand-checked CSV slices) — not reproducible from a kernel, and limited to a
handful of bodies. This work makes a broad, astrologically-relevant asteroid set
available as committed reference data, with the well-determined main-belt core
reproducible from a pinned kernel and the wider esoteric set committed as
clearly-separated constrained/provenance evidence.

## Scope decisions (from brainstorming)

| Decision | Choice |
| --- | --- |
| Body class | **Physical minor planets only.** No calculated points, no fictional bodies. |
| Default asteroid window | **1900–2100 CE** (distinct from the 1600–2600 major-body window). Fails closed outside it. |
| Beyond the window / uncommitted bodies | User supplies data through the existing `ingest` / `horizons-fetch` tools — library provides the recipe, not the committed data. |
| Sourcing model | **Two tiers:** Tier A pinned main-belt kernel (reproducible-from-kernel); Tier B Horizons-generated constrained/provenance slices. |
| Pinned kernel (Tier A) | **`sb441-n16.bsp`** — DE441-consistent (agrees with de440 over the overlap), full-range, SHA-pinnable, contains the classical 4 plus other massive main-belt bodies. |
| Roster model | **Curated core (~35 bodies) committed + unbounded long tail on demand** via `Custom` ids + the ingest tools. |
| Accuracy/claims posture | Entire asteroid class is **constrained**, advertised over 1900–2100 — alongside the existing "Pluto stays constrained" posture. |

## Non-goals

- **Calculated points** — Black Moon Lilith (lunar apogee), Part of Fortune,
  Vertex, etc. These are deferred to a downstream higher-level astrology crate
  that computes them from this library's outputs. This design's only obligation
  is to **verify** the library already exposes enough primitives (Moon/node
  geometry, body positions, chart angles) for that crate to compute them; it
  does not implement them.
- **Fictional / hypothetical bodies** — Uranian/Hamburg-school planets (Cupido,
  Hades, Zeus, Kronos, Apollon, Admetos, Vulkanus, Poseidon), Vulcan,
  Selena/White Moon, Transpluto. Not physical, no ephemeris, not derivable;
  excluded entirely.
- **Exhaustive asteroid coverage** — committing a slice per numbered minor
  planet (the Swiss-Ephemeris approach, 1,000,000+ bodies) is explicitly out.
  The committed corpus is a curated core; arbitrary bodies are reachable on
  demand (see Tier B / on-demand tail).
- **Promoting asteroids to release-grade claims** — the class stays constrained.
- **Asteroid-specific apparent-place / topocentric / sidereal output** — the
  existing backend boundary (mean, geometric, geocentric, tropical) is unchanged.

## Background: what already exists

The groundwork makes this a focused addition, not new infrastructure:

- The pure-Rust SPK reader already maps the classical asteroids to NAIF ids
  (`chain.rs:26` — Ceres→`2000001` … Vesta→`2000004`) and parses arbitrary
  `asteroid:NNN-Name` `Custom` ids into candidate ids via `parse_custom_naif`
  (`chain.rs:39`, trying both `2_000_000+n` and `20_000_000+n` schemas).
- `KernelPool` (`spk/pool.rs`) and `SpkBackendBuilder` (`spk/backend.rs:31`)
  already load **multiple** kernels and query them as one set. A de440 +
  asteroid-kernel pairing composes today with **no plumbing change** — de440
  supplies Earth/Sun, the asteroid kernel supplies the target body.
- `generate_slice` / `generate_corpus_csv` (`spk/generate.rs`) already samples a
  backend into the corpus CSV schema with provenance headers.
- `corpus_spec.rs` defines `SliceRole`, `KERNEL_LABEL`, `KERNEL_SHA256`, and the
  cadence/epoch helpers; the gated `corpus_regen` test reproduces each
  major-body slice from de440 within ~1 km.
- `validate-corpus` is a live fail-closed gate over the committed corpus.
- `ingest` + `horizons-fetch` (landed 2026-06-17) already read arbitrary
  Horizons/CSV inputs into the corpus types — the mechanism Tier B and the
  on-demand tail reuse.
- `production_generation.rs` already reports per-body-class coverage with a
  distinct selected-asteroid subset (`asteroid_row_count`, `asteroid_windows`).

## Architecture

### Two-tier asteroid sourcing

```
Tier A — pinned main-belt kernel (reproducible-from-kernel)
  source:  sb441-n16.bsp  (SHA-pinned, NOT committed; SHA-256 recorded)
  bodies:  classical 4 + the massive main-belt members of n16 that are
           astrologically used (Hygiea, Psyche, Iris, ...)
  gate:    corpus_regen reproduces the Tier-A asteroid slice from the pinned
           kernel within the existing ~1 km tolerance (env-gated)

Tier B — Horizons-generated constrained/provenance slices (committed)
  source:  Horizons per-object SPKs over 1900-2100 (not byte-stable -> not
           SHA-pinned); solution epoch + generation date recorded
  bodies:  centaurs (Chiron, Pholus, Nessus, Chariklo, Asbolus),
           personal/"goddess" asteroids (Eros, Sappho, Amor, Lilith-1181,
           Astraea, Hebe, Flora, Metis, Fortuna, Hidalgo, Icarus, Toro,
           Apollo, ...), and TNOs/dwarfs (Eris, Sedna, Haumea, Makemake,
           Quaoar, Orcus, Ixion, Varuna, Gonggong)
  gate:    schema / checksum / non-finite / provenance + fixture-golden
           cross-check (NOT regen) — kept separate in data and reports

On-demand tail (not committed)
  any other numbered minor planet via CelestialBody::Custom + ingest/
  horizons-fetch tooling, over any range the user supplies
```

The two tiers are kept **separate in data and reports** (Phase 1 rule:
fitting/reference, hold-out, boundary-overlay, fixture-exactness, and
provenance-only evidence stay distinct).

### Curated core roster (~35 bodies)

Categories below are the design intent. **Exact designations are verified
against the MPC numbered-asteroid catalog / Horizons during implementation —
no designation numbers are hard-coded from memory into the corpus.**

- **Classical 4:** Ceres, Pallas, Juno, Vesta *(named `CelestialBody` variants)*
- **Centaurs:** Chiron, Pholus, Nessus, Chariklo, Asbolus
- **Personal / "goddess" asteroids:** Eros, Psyche, Sappho, Amor, Lilith (1181),
  Hygiea, Astraea, Hebe, Flora, Metis, Iris, Fortuna, Hidalgo, Icarus, Toro,
  Apollo
- **TNOs / dwarf planets:** Eris, Sedna, Haumea, Makemake, Quaoar, Orcus, Ixion,
  Varuna, Gonggong

### Body taxonomy

- Keep `CelestialBody::{Ceres, Pallas, Juno, Vesta}` as named variants.
- **All other curated bodies use the existing `Custom`/`catalog:designation`
  mechanism** (`asteroid:2060-Chiron`, `tno:136199-Eris`, …) so the enum does
  not balloon. The NAIF-id chain already resolves these.
- Extend the curated "reference asteroid" registry and `is_reference_asteroid`
  (`backend.rs:2274`) to recognize the committed core, tagging each entry by
  **tier** (pinned vs constrained) and **class** (main-belt / centaur / TNO) so
  reports can keep the evidence classes separate.

### Corpus size / cadence

Asteroids and TNOs move slowly; sample at a **speed-appropriate, coarse
cadence** (slower than the fast-cluster major-body cadence; sparser still for
TNOs) so the committed asteroid corpus stays bounded and does not dwarf the
existing ~25,659-row major-body corpus. The exact cadence is set during
implementation against a target total-row budget.

## Validation & reproducibility gates

- **Tier A:** extend `corpus_regen` with a second env var (e.g.
  `PLEIADES_AST_KERNEL` pointing at `sb441-n16.bsp`) that reproduces the Tier-A
  asteroid slice within the existing tolerance. Clean checkout stays kernel-free
  and skips (early return), exactly like the de440 path today.
- **Tier B:** `validate-corpus` fails closed on the constrained slices the same
  way it does for existing slices — missing bodies/epochs/channels/roles, schema
  drift, checksum/source-revision drift, malformed/non-finite rows — but Tier B
  is held to **provenance + fixture-golden cross-check**, not kernel regen.
- Coverage advertising: the asteroid window (1900–2100) is advertised from the
  data/segment descriptors and reports; requests outside it fail closed (no
  silent extrapolation), consistent with the existing coverage gate.

## Provenance documentation

Fill in the **"Asteroid kernel"** section of `docs/spk-kernel-sourcing.md`
(currently a placeholder):

- **Tier A pinned kernel:** filename (`sb441-n16.bsp`), source URL (JPL NAIF /
  SSD), license (public domain, U.S. Government work), SHA-256, advertised
  coverage, and the `PLEIADES_AST_KERNEL` usage + `corpus_regen` recipe.
- **Tier B Horizons generation:** per-object designations, solution epoch,
  generation date, and the exact Horizons command/recipe used — so the
  constrained slices are auditable even though they are not byte-pinned.

## Definition of done

- `sb441-n16.bsp` provenance + SHA-256 recorded in `docs/spk-kernel-sourcing.md`;
  the kernel itself remains uncommitted (kernel-free clean checkout).
- The curated core (~35 bodies) is committed: Tier A as reproducible-from-kernel
  slices, Tier B as constrained/provenance slices, kept separate in data and
  reports.
- `corpus_regen` reproduces the Tier-A asteroid slice from the pinned kernel
  within tolerance (env-gated); clean checkout still skips and passes.
- `validate-corpus` covers the new slices and fails closed on the documented
  drift conditions.
- Reports/summaries surface asteroids as a **constrained** class advertised over
  1900–2100, separate from release-grade major-body claims.
- Any numbered minor planet remains reachable on demand via `Custom` ids + the
  ingest/horizons tooling.
- A short note confirms the library exposes the primitives a downstream crate
  needs for calculated points (Black Moon Lilith etc.) — or records precisely
  what is missing, if anything.
- PLAN.md and `plan/stages/01-production-reference-corpus.md` updated: the
  asteroid-kernel item is removed from "remaining work" per the plan-maintenance
  rule (remove when implemented, don't annotate).

## Risks & mitigations

- **Tier B byte-instability** — Horizons SPKs are not reproducible byte-for-byte.
  *Mitigation:* Tier B is provenance/golden-validated, not regen-gated; solution
  epoch + generation date are recorded so drift is auditable.
- **Designation errors** — guessing IAU numbers risks committing wrong bodies.
  *Mitigation:* all designations verified against MPC/Horizons at implementation
  time; none are taken from memory.
- **Corpus bloat** — ~35 bodies over 200 years could balloon the corpus.
  *Mitigation:* coarse, speed-appropriate cadence against a row budget.
- **`sb441-n16` membership assumptions** — the exact n16 body list and its
  advertised coverage are verified against the actual downloaded file before its
  SHA is pinned (consistent with "record provenance when added").
