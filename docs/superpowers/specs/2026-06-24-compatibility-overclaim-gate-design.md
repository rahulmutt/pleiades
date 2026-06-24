# Compatibility Overclaim Gate — Design

Date: 2026-06-24
Phase: 5 (Compatibility and Release Gates)
Status: design approved; ready for implementation plan

## Problem

Phase 5's compatibility-audit pair is complete: `validate-houses` (12 baseline
systems) and `validate-ayanamsa` (6 release-claimed modes) both pass their
numeric residual gates. But nothing connects an entry's **claim level** to its
**numeric evidence**. The "release-grade numeric" tier is implicit and scattered
across three independent assertions:

1. the gate's covered set (`ayanamsa_mode_class() == Some`; the four
   corpus-validated house formula families),
2. `thresholds.rs`'s "corpus-validated vs generous placeholder" distinction, and
3. README/CLI prose ("12 house systems pass", "6 ayanamsa modes pass").

Because these are independent, adding a house system or ayanamsa to the
release-claimed set — or marking one release-grade — passes every existing gate
**without** a corpus row or passing residual backing it. This is the overclaim
hole Phase 5 is meant to close, plus a release-gate-hardening gap: the numeric
gates currently run only under `cargo test`, so the standalone `release-smoke` /
`release-gate` CLI path never executes them.

This design closes both as one item: a single source of truth for the claim
tier, a bidirectional overclaim audit, and wiring the full numeric-gate set into
the in-process release path.

## Scope

In scope:

- A per-entry claim tier on the house and ayanamsa catalog descriptors (the new
  single source of truth).
- A `compat-claims-audit` that proves claim tier ↔ numeric evidence ↔ profile ↔
  prose all agree.
- Wiring the numeric gates and the overclaim audit into
  `validate_release_smoke_at` (and therefore `release-gate`).

Out of scope:

- Tightening the generous, non-corpus-validated house family ceilings (separate
  future work; those families stay `DescriptorOnly`).
- Gating the remaining ~53 ungated ayanamsa variants or the remaining house
  systems beyond the baseline 12 (Phase 6 catalog-completion work).
- Any broadening of public release claims.

## Tier model

A new shared enum lives in `pleiades-types` (alongside the existing typed
vocabulary) so both catalog crates can use it without depending on each other:

```rust
pub enum CompatibilityClaimTier {
    /// Numeric compatibility is asserted: the entry is exercised by the SE
    /// numeric gate against a *corpus-validated* ceiling and passes it.
    ReleaseGradeNumeric,
    /// Catalogued with metadata/aliases only. No numeric compatibility claim.
    DescriptorOnly,
}
```

**Two tiers, no ambiguous middle state** — matching the principle Phase 3
enforced for body/backend claims.

**Evidence is per-entry, not per-family/class.** "Has numeric evidence" means
the entry itself appears in the SE corpus *and* its measured residual passes the
ceiling — not merely that it belongs to a family/class some other entry
validated. This distinction matters: the house ceiling is keyed by formula
family, but only some members of a corpus-validated family have actual corpus
rows. The `ReleaseGradeNumeric` set is therefore exactly:

- **House systems (12)** — the distinct `system_code`s present in
  `crates/pleiades-validate/data/houses-corpus/cusps.csv`: `Placidus`, `Koch`,
  `Porphyry`, `Regiomontanus`, `Campanus`, `Equal`, `WholeSign`, `Alcabitius`,
  `Meridian`, `Axial`, `Topocentric`, `Morinus`. Family-mates with no corpus row
  (e.g. `Vehlow`, `EqualMidheaven`, `EqualAries`, `Sripati`, `Carter`) are
  `DescriptorOnly` even though their family carries a corpus-validated ceiling.
- **Ayanamsas (6)** — the modes present in
  `crates/pleiades-validate/data/ayanamsa-corpus/ayanamsa.csv`: `Lahiri`,
  `Raman`, `Krishnamurti`, `FaganBradley`, `TrueChitra`, `TrueCitra`.

A house family carrying a generous, explicitly **NOT corpus-validated** ceiling
(`GreatCircle`, `SolarArc`, `Sector`, `Custom`, `Unknown`) is `DescriptorOnly`
for the same reason: holding a placeholder ceiling is not evidence.

### Descriptor change (breaking — accepted)

Add `pub claim_tier: CompatibilityClaimTier` to both `HouseSystemDescriptor`
(`pleiades-houses`) and `AyanamsaDescriptor` (`pleiades-ayanamsa`). The field is
non-`Option`, so every catalog entry must set it explicitly. This is a breaking
change to the published `0.2.x` catalog crates and is accepted.

After this change, README counts, compatibility-profile slices, and gate
coverage are all **derived from and verified against** `claim_tier` rather than
being independent assertions.

## The overclaim audit

New module `crates/pleiades-validate/src/claims/compat.rs`, mirroring the body
`claims-audit` structure: a `CompatClaimAuditError` enum, one function per check,
and an aggregate entry point returning `Result<(), Vec<CompatClaimAuditError>>`.

### Check A — Tier ↔ evidence agreement (bidirectional)

For every descriptor marked `ReleaseGradeNumeric`:

1. it must be in the gate's **validated-entry set** (the distinct systems/modes
   actually present in the corpus, exposed by the gate report — see below), and
2. the numeric gate must pass overall (it already enforces residual ≤ ceiling
   per row), reusing the existing gate *report* rather than recomputing
   residuals.

For every descriptor marked `DescriptorOnly`: it must **not** be in the gate's
validated-entry set.

To make the validated-entry set available, `HouseCorpusReport` and
`AyanamsaCorpusReport` each gain an accessor returning the distinct typed
entries they validated (`validated_systems() -> Vec<HouseSystem>`,
`validated_modes() -> Vec<Ayanamsa>`). `Vec` + `PartialEq` membership is used
(not `BTreeSet`/`HashSet`) because `Ayanamsa` derives only `PartialEq` and
`HouseSystem` lacks `Ord`; this avoids adding derives to the published types.
The reports already track `systems_checked` / `modes_checked` counts; the new
accessors expose the membership Check A compares against.

Bidirectionality makes the `ReleaseGradeNumeric` descriptor set and the
corpus-validated gate set provably equal, so both overclaim (claim without
evidence) and silent under-claim drift (evidence exists but isn't reflected) fail
closed. This is the catalog analog of `audit_release_grade_accuracy`.

### Check B — Profile ↔ tier agreement

The compatibility profile gains a derived "release-grade-numeric" view computed
*from* `claim_tier`. `verify-compatibility-profile` asserts that view is
internally consistent (every `ReleaseGradeNumeric` entry is a shipped built-in,
not custom/descriptor-only territory) and that its membership/count matches Check
A's descriptor-derived set. The profile becomes a derived-and-verified surface
rather than an independent assertion.

### Check C — Surface / prose drift

Mirror `check_claim_drift`'s token approach: assert README and the relevant CLI
summaries carry counts/labels matching the descriptor-derived sets ("12 house
systems", "6 ayanamsa modes"), so prose cannot drift from the gated reality.

### Error variants

Each variant names the catalog, the entry, and the violated invariant, e.g.:

- `ReleaseGradeWithoutCorpusEvidence { catalog, entry }`
- `ReleaseGradeAboveCeiling { catalog, entry }`
- `DescriptorOnlyHasEvidence { catalog, entry }`
- `ProfileCountMismatch { catalog, profile, descriptors }`
- `SurfaceDisagrees { surface }`

## Release-gate wiring

### New CLI subcommands (`render/cli.rs`)

- `compat-claims-audit` (+ `compat-claims-audit-summary` alias) → runs the
  aggregate overclaim audit (Checks A–C); fails closed with the full violation
  list.

### Numeric gates into the in-process release path

Today the gates (`validate-houses`, `validate-ayanamsa`, `validate-apparent`,
`validate-topocentric`, `validate-corpus`) run only under `cargo test`, so the
standalone `release-smoke` / `release-gate` CLI never executes them. Add a
`run_all_numeric_gates()` helper and call it from `validate_release_smoke_at`, so
the smoke/gate path becomes:

```
workspace_audit  ->  run_all_numeric_gates  ->  compat overclaim audit
  ->  verify_compatibility_profile  ->  artifact_report
  ->  bundle + verify_bundle  ->  check_claim_drift
```

Check A reuses the house/ayanamsa gate reports produced by
`run_all_numeric_gates` (no second residual computation).

### `mise.toml`

`release-smoke` now transitively covers the gates, so `release-gate`'s dependency
list needs no new entries. Add per-gate convenience tasks (`gate-houses`,
`gate-ayanamsa`, `gate-apparent`, `gate-topocentric`, `gate-corpus`) for running
them individually.

## Testing (TDD)

- **Per-check unit tests** with synthetic inputs:
  - a `ReleaseGradeNumeric` entry absent from the gate set →
    `ReleaseGradeWithoutCorpusEvidence`;
  - a `DescriptorOnly` entry that *is* corpus-validated →
    `DescriptorOnlyHasEvidence`;
  - a mismatched profile count → `ProfileCountMismatch`;
  - a wrong README token → `SurfaceDisagrees`.
- **Non-empty guard** (mirrors `packaged_data_expected_body_set_is_non_empty`):
  assert the `ReleaseGradeNumeric` set is non-empty, so the audit cannot pass by
  checking nothing.
- **Real-catalog pass**: the audit passes against the actual catalogs.
- **Count assertions**: exactly 12 `ReleaseGradeNumeric` house systems and 6
  ayanamsas.

## Exit criteria

- `HouseSystemDescriptor` and `AyanamsaDescriptor` carry an explicit
  `claim_tier`; every entry sets it.
- `compat-claims-audit` exists, fails closed on any A/B/C violation, and passes
  on the current catalogs.
- `validate_release_smoke_at` runs the full numeric-gate set and the overclaim
  audit, so `release-gate` exercises them without relying on `cargo test`.
- README/profile/CLI surfaces are verified against the descriptor-derived sets;
  prose drift fails the gate.
- No public release claim is broadened.

## Plan-doc updates on completion

- Mark Phase 5 "release-gate hardening and compatibility-profile overclaim
  checks" done in `PLAN.md`; update the Phase 5 progress note.
- Keep README's catalog/claim wording aligned with the descriptor-derived
  counts.
