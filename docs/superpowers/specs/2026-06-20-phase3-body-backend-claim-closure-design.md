# Phase 3 — Body & Backend Claim Closure

Status: design approved (2026-06-20)
Phase: 3 (Body/Backend Claim Closure) — the active frontier per `PLAN.md`
Predecessors: Phase 1 (production reference corpus) and Phase 2 (release-grade
compressed ephemeris, SP3) — complete and merged
Branch base: `main`
Scope: Phase 3 only. Phases 4 (request-mode semantics) and 5 (compatibility &
release gates) get their own spec → plan → implement cycles afterward.

## Goal

Ensure every public body/backend claim is **source-backed, artifact-backed,
constrained, approximate, or unsupported, with no ambiguous middle state**, and
that every release-facing surface (backend matrices, compatibility profiles,
release summaries, CLI output, rustdoc) agrees structurally — enforced by a gate,
not by hand-maintained prose.

Exit criteria (from `plan/stages/03-body-and-backend-claims.md`):

- Backend matrices, compatibility profiles, release summaries, CLI output, and
  rustdoc agree on body/backend support and limitations.
- No release-facing surface advertises unsupported or approximate bodies as
  production-grade.

## Posture decision (resolved during brainstorming)

**Raise-where-feasible, expressed per-backend.** The first release makes the
strongest *honest* claims it can by promoting bodies through the backend that
already has validating evidence — not by writing new astronomy. Concretely:

1. **Scope: Phase 3 only.** Couples no unmade Phase 4/5 decisions.
2. **Claim model: per-backend, not global.** A body is release-grade *for a
   given backend* when that backend has the evidence. The same body can hold
   different tiers via different backends (Pluto is release-grade via
   packaged-data and approximate via VSOP87). This replaces today's single global
   "major bodies = Sun–Neptune" verdict.
3. **Algorithmic backends stay honestly labeled.** No new theory is written.
   VSOP87's simplified-element Pluto stays approximate; `pleiades-elp`'s compact
   Meeus Moon stays constrained. The packaged-data and JPL/SPK backends carry the
   release-grade claims because they already validate against the corpus.
4. **Tier-A asteroids are release-grade via JPL/SPK only.** Ceres/Pallas/Juno/
   Vesta have release-quality evidence only through the `ReferenceData` JPL/SPK
   backend (reproducible from the pinned `sb441-n16` kernel). They are *not* added
   to the offline packaged artifact (preserves the hard ≤ 12 MB size budget). The
   JPL/SPK backend is documented as corpus/kernel-dependent reference data.

### Validation bar for `ReleaseGrade`

`ReleaseGrade` for a body via a backend means: validation evidence passes the
existing **SP3 per-body-class accuracy ceilings** (`crates/pleiades-data/src/
thresholds.rs`) against the de440/sb441 corpus over the 1900–2100 window, using
the existing comparison harness (`crates/pleiades-validate/src/comparison/`).
Tier-A asteroids additionally require evidence traced to `sb441-n16`. No new
accuracy machinery is introduced; Phase 3 *consumes* the SP3 contract.

## Current baseline (what exists today)

- `BackendMetadata` (`crates/pleiades-backend/src/metadata.rs:122`) carries a flat
  `body_coverage: Vec<CelestialBody>` and a **single backend-wide**
  `accuracy: AccuracyClass`. There is no per-body accuracy or claim status — which
  is precisely why claims had to live in a separate global layer and why an
  algorithmic backend cannot say "majors good, Pluto approximate."
- `release_body_claims.rs` (`crates/pleiades-backend/src/`) is **global and
  string-based**: hand-written summary prose built from hardcoded `*_bodies()`
  lists, validated by `.contains(EXACT_PHRASE)` (lines 248–264). This is the
  brittle layer this design replaces.
- Canonical posture constants live in `policy/mod.rs` and are rendered to users by
  `pleiades-validate/src/render/summary/release.rs` and CLI commands in
  `pleiades-cli/src/cli.rs`; committed sidecars are checksummed by
  `release/bundle_verify.rs`.
- The packaged artifact (`crates/pleiades-data/src/lib.rs:172`) holds exactly 11
  bodies — Sun, Moon, Mercury–Pluto, and `asteroid:433-Eros` — all sub-arcsec and
  corpus-validated (SP3). The JPL/SPK backend (`crates/pleiades-jpl/src/backend.rs`,
  `spk/backend.rs`) serves Tier-A asteroids from `sb441-n16` plus a constrained
  Tier-B set. `pleiades-elp` serves Moon + lunar points; VSOP87 serves Sun +
  Mercury–Pluto.

## Target claim assignments

Each backend's `metadata()` declares per-body claims per this table. The audit
gate proves each assignment against evidence.

| Backend | ReleaseGrade | Constrained | Approximate | Unsupported |
|---|---|---|---|---|
| packaged-data | Sun, Moon, Mercury–Pluto, Eros | — | — | — |
| JPL/SPK | Ceres, Pallas, Juno, Vesta | Apophis, Tier-B (centaurs/TNOs) | — | — |
| ELP | — | Moon, Mean/True Node, Mean Apogee/Perigee | — | True Apogee/Perigee |
| VSOP87 | — | Sun, Mercury–Neptune | Pluto | — |

Pluto and Moon each appear under different tiers via different backends — the
per-backend model resolving the global contradiction, rather than one global
verdict.

## Architecture

### Section 1 — Claim data model

New module `crates/pleiades-backend/src/claims.rs`, mirroring the existing
`identity.rs::AccuracyClass` style (Display, serde, stable labels):

```rust
enum BodyClaimTier { ReleaseGrade, Constrained, Approximate, Unsupported }

enum ClaimEvidence {
    ArtifactValidated,                 // packaged-data, corpus-checked at build
    CorpusValidated { source: String },// e.g. sb441-n16, de440
    AlgorithmicModel,                  // VSOP87 / compact ELP
    None,
}

struct BodyClaim {
    body: CelestialBody,
    tier: BodyClaimTier,
    accuracy: AccuracyClass,           // per-body now, not backend-wide
    evidence: ClaimEvidence,
}
```

**Representational decision:** `BackendMetadata.body_coverage: Vec<CelestialBody>`
is **replaced** by `body_claims: Vec<BodyClaim>` as the single source of truth.
Coverage becomes derived views, never a second stored list (which would
re-introduce drift):

- `supported_bodies()` → bodies with tier ≠ `Unsupported`. Preserves preflight
  semantics exactly.
- `claim_for(&body) -> Option<&BodyClaim>`, `release_grade_bodies()`,
  `claims_by_tier()`.

The backend-wide `accuracy: AccuracyClass` field stays as a headline label; the
precise truth is per-body inside each `BodyClaim`.

`Unsupported`-tier bodies *may* be listed (e.g. ELP's True Apogee/Perigee) to make
the "unsupported" claim explicit and renderable, but `supported_bodies()` excludes
them and preflight rejects requests for them.

### Section 2 — Components & file-level changes

- **A. New claim types** — `pleiades-backend/src/claims.rs`; accessors on
  `BackendMetadata`.
- **B. Metadata change** — `metadata.rs`: field swap; `validate_request` (line 213)
  uses `supported_bodies().contains(&req.body)`; `validate()` (300–340) gains
  no-duplicate-body and internal-consistency checks (`Unsupported` ⇒ not in
  `supported_bodies()`); `summary_line()` renders per-body tiers.
- **C. Per-backend population** — all four `metadata()` impls build `body_claims`
  per the table: `pleiades-data/src/backend.rs`, `pleiades-jpl/src/backend.rs` +
  `spk/backend.rs`, `pleiades-elp/src/backend.rs`, `pleiades-vsop87/src/backend.rs`.
  **All four convert in this one effort** — a half-migrated model would force
  shims in the aggregator.
- **D. Derived release posture** — `release_body_claims.rs` rewritten: delete the
  hardcoded `*_bodies()` lists and hand-written `release_body_claims_summary_text()`;
  introduce `ReleasePosture::from_backends(&[&BackendMetadata])` aggregating over a
  new **canonical first-party backend set** (explicit registry of the four release
  backends → deterministic aggregation). Summary text is rendered from the
  aggregate. `policy/mod.rs` posture constants become derived values, not frozen
  prose.
- **E. Render/CLI/rustdoc** — `pleiades-validate/src/render/summary/release.rs`
  (189–251), CLI commands in `pleiades-cli/src/cli.rs`
  (`release-body-claims-summary`, `pluto-fallback-summary`, `backend-matrix`,
  `release-summary`), and rustdoc examples all consume the derived posture.

### Section 3 — Audit + drift gate

New surface `crates/pleiades-validate/src/claims/` (`audit.rs`, `drift.rs`,
`mod.rs`), exposed via a `claims-audit` CLI command and wired into the release
gate + CI alongside the existing fail-closed gates (`validate-corpus`, size gate).

**1. Capability audit** — does each backend support what it claims, at the claimed
tier? For every `BodyClaim` across the canonical backend set:

- *Coverage check:* a representative `position()` call returns `Ok` for a supported
  body; an `Unsupported`-tier body is rejected by preflight.
- *Tier-vs-evidence check* (the honesty gate):
  - `ReleaseGrade` ⇒ evidence passes the SP3 per-body-class ceiling against
    de440/sb441 over 1900–2100, via the existing comparison harness. No passing
    evidence ⇒ hard fail. Tier-A asteroids additionally require
    `ClaimEvidence::CorpusValidated` traced to `sb441-n16`.
  - `Constrained` ⇒ evidence exists but is below release ceiling or
    corpus-dependent; `Approximate` ⇒ algorithmic, no release claim; `Unsupported`
    ⇒ preflight rejects. Each tier has a precise, testable evidence predicate.
- *Metadata self-consistency:* declared `nominal_range`, frames, time scales, and
  observer/apparent capability match what the backend actually honors (the plan's
  "audit capability metadata against actual support" item).

**2. Drift gate** — does every rendered surface match the derived truth? Re-derive
`ReleasePosture` from metadata and assert structural equality against each rendered
surface: release-body-claims summary, pluto-fallback summary, backend matrix,
compatibility profile, and the committed sidecars `bundle_verify.rs` checksums.
This **replaces** the brittle `.contains(EXACT_PHRASE)` checks (lines 248–264) with
model comparison — wording can evolve freely as long as it still derives from the
same claims.

**3. Structured errors** (existing-style typed enums):
`ClaimAuditError::{ DeclaredBodyNotComputable, ReleaseGradeMissingEvidence,
EvidenceBelowCeiling, TierEvidenceMismatch, MetadataActualMismatch }` and
`ClaimDriftError::{ SurfaceDisagreesWithPosture { surface }, StaleSidecar { path } }`.
Fail-closed.

**4. Fast/slow split** (per the repo's `#[ignore]` test-speed discipline): the
slow corpus-comparison portion of the audit is `#[ignore]`'d and run in
CI/release-gate; a fast structural audit (coverage + tier-consistency + drift, no
corpus math) runs in the default test pass.

### Section 4 — Migration, report stability & error handling

**Migration sequence (order matters):**

1. Land `BodyClaim`/`BodyClaimTier`/`ClaimEvidence` + accessors (additive).
2. Swap `body_coverage` → `body_claims` in `BackendMetadata`; update all four
   backends and the `body_coverage.contains(...)` call sites (preflight, routing,
   batch validation in `policy/current.rs`). Workspace compiles green.
3. Rewrite `release_body_claims.rs` to derive `ReleasePosture`; make `policy/mod.rs`
   constants derived.
4. Point render/CLI/rustdoc surfaces at the derived posture.
5. **Regenerate committed sidecars** from the new renderers and re-checksum them in
   `bundle_verify.rs` *in the same commit*, so the bundle gate flips red→green
   atomically and never lands broken.
6. Add the audit + drift gate; wire into release-gate/CI.

**Report-text stability:** the promotions change the claims, so summary text *will*
change (Pluto/Moon/Eros release-grade via packaged-data; Tier-A via JPL/SPK). That
is intended and visible. The commitment: the change is driven by the derived model,
sidecar regeneration is part of the migration commit, and the drift gate keeps them
in sync afterward. Old exact-phrase consts (`LUNAR_VALIDATION_PHRASE`, etc.) are
deleted, not updated.

**README / PLAN alignment** (the repo's maintenance rule): README "current limits"
and `PLAN.md` Phase 3 status are updated in the same change — Pluto reframed from
globally "approximate/fallback-backed" to "approximate via VSOP87, release-grade
via packaged-data."

**Error handling, end to end:**

- *Request preflight:* unchanged contract — `Unsupported`-tier or absent body ⇒
  `EphemerisError::UnsupportedBody` before computation. `Constrained`/`Approximate`
  bodies still compute (claim tier is advisory metadata, never a request gate).
- *Audit/drift:* fail-closed structured errors, surfaced by the release gate.
- *API break:* replacing the public `body_coverage` field on `BackendMetadata` is
  a breaking change. Acceptable for the experimental `0.2.x` crates; replaced
  outright with no deprecated shim. Noted in the changelog with an appropriate
  version bump.

### Section 5 — Testing strategy

**Unit (fast, default pass):**

- `claims.rs`: construction, `Display`, serde round-trip, accessor logic
  (`supported_bodies()` excludes `Unsupported`, `claim_for`, `release_grade_bodies`).
- `BackendMetadata::validate()`: rejects duplicate-body claims and inconsistencies;
  `validate_request` preflight still rejects `Unsupported`/absent bodies and admits
  `Constrained`/`Approximate`.
- Per-backend `metadata()`: each of the four backends emits exactly the approved
  claim table (golden assertions).

**Derived posture + drift (fast):**

- `ReleasePosture::from_backends` aggregates deterministically; same backends →
  identical posture.
- Drift: rendered summary / pluto-fallback / backend-matrix / compatibility-profile
  each equal the re-derived posture; snapshot tests for human-readable summary text
  so intended wording changes are reviewed explicitly.
- Teeth check: tampering metadata (e.g. flip VSOP87 Pluto to `ReleaseGrade`) makes
  the drift/audit gate fail.

**Capability audit (slow, `#[ignore]`'d, CI/release-gate):**

- Coverage probe: every declared body computes; `Unsupported` rejected.
- Tier-vs-evidence: each `ReleaseGrade` body passes its SP3 body-class ceiling via
  the comparison harness; Tier-A traces to `sb441-n16`; downgrade tiers satisfy
  their weaker predicates.
- Metadata-vs-actual: nominal range / frames / time scales / observer-apparent
  capability match real behavior.

**Integration / gate:**

- `bundle_verify` passes against regenerated sidecars; `claims-audit` CLI exits
  non-zero on any seeded violation.
- `mise test` (fast) green without corpus math; `mise test-full` / CI runs the
  ignored audit.

## Exit-criteria mapping

- *"No surface advertises unsupported/approximate as production-grade"* → audit
  tier-vs-evidence check, fail-closed.
- *"Backend matrices, profiles, summaries, CLI, rustdoc agree"* → drift gate,
  structural equality.
- *"No ambiguous middle state"* → every covered body carries exactly one
  `BodyClaimTier`; `validate()` enforces totality.

## Out of scope (explicit)

- New astronomy: no source-backed VSOP87 Pluto, no full ELP coefficient lunar
  theory. Algorithmic backends stay honestly labeled.
- Adding Tier-A asteroids to the offline packaged artifact (preserves ≤ 12 MB).
- Phase 4 request-mode semantics (UTC/Delta-T, apparent, topocentric, native
  sidereal) and Phase 5 house/ayanamsa provenance audits — separate cycles.
- Apophis and Tier-B asteroids remain constrained for first release.
