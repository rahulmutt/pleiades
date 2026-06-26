# Ayanamsa Fabricated Observational-Babylonian Mode Removal (Phase 6, slice 4)

Date: 2026-06-26
Status: design approved, pending implementation plan

## Goal

Remove the six **fabricated** observational-Babylonian ayanamsa enum variants —
`BabylonianTrueGeoc`, `BabylonianTrueTopc`, `BabylonianTrueObs`,
`BabylonianHouse`, `BabylonianHouseObs`, `BabylonianSissy` — from the public API
and catalog, and correct the false provenance they ship. They have no Swiss
Ephemeris code, no entry in the committed spec target catalog, no external
reference engine, and no computation path. This closes the ayanamsa half of
Phase 6 by aligning the code with the spec's committed scope instead of carrying
dead, mislabeled surface as if it were deferred work.

This is **not** a promotion slice. The cubic-fit promotion path is exhausted:
all 48 smoothly-fittable, SE-coded modes are already release-grade (slices 1–3).
This slice removes surface rather than adding evidence.

## Background / why these six are different

After slice 3, the catalog held **59 catalogued ayanamsas**: 48 release-grade
(SE numeric gate) and **11 deferred** (`DescriptorOnly`). The 11 deferred modes
are not homogeneous. They split into:

- **Fabricated / out-of-scope (these six):** `BabylonianTrueGeoc`,
  `BabylonianTrueTopc`, `BabylonianTrueObs`, `BabylonianHouse`,
  `BabylonianHouseObs`, `BabylonianSissy`.
- **Real-but-ungateable (the remaining five — kept, see "Out of scope"):**
  `Udayagiri`, `PvrPushyaPaksha`, `Sheoran`, `DhruvaGalacticCenterMula`, legacy
  `GalacticEquator`.

The six are fabricated in a precise, verifiable sense:

1. **No SE code.** The pinned `libswisseph` header (`swephexp.h`) defines
   `SE_SIDM_*` constants only through `46` (`LAHIRI_ICRC`), plus `255`
   (user-defined). None of `BABYL_TRUE_GEOC / _TOPC / _OBS / _HOUSE /
   _HOUSE_OBS / _SISSY` exist, and no `SE_SIDBIT_*` flag combination produces
   them (the SIDBITs select projection plane / precession handling, not
   geocentric/topocentric/observational/house variants).
2. **Not in the committed target catalog.** `spec/compatibility-catalog.md`
   binds the ayanamsa target to "the full Swiss Ephemeris sidereal-mode catalog,
   referenced by the `SE_SIDM_*` constants as the authoritative enumeration."
   These six are absent from that list.
3. **No external reference engine.** Swiss Ephemeris (header + official docs +
   pyswisseph binding) does not expose them; a web survey of astrology engines
   found no software that defines `True Geoc / True Topc / True Obs / House /
   House Obs / Sissy` as ayanamsha modes. There is genuine *scholarship* on the
   observationally-defined Babylonian zodiac (Mercier 1977 on η Piscium
   culmination; Koch's research), but that is literature, not a computational
   authority emitting reference values. Implementing from it would be original
   research with self-certified output — an overclaim under this project's
   evidence-promotes-the-claim discipline, and outside the committed SE-set
   scope.
4. **They ship false provenance.** Each descriptor currently asserts text such
   as *"Babylonian sidereal mode labeled BABYL_TRUE_GEOC in Swiss Ephemeris."*
   Swiss Ephemeris has no such label. This is a shipped inaccuracy regardless of
   any other decision.

The five real-but-ungateable modes are categorically different: `Sheoran`
("Vedic"/Sheoran) and `DhruvaGalacticCenterMula` (Dhruva/Wilhelm) **are** in the
spec target catalog; `Udayagiri` and `PvrPushyaPaksha` are real published
ayanamsas that merely lack an SE_SIDM code; legacy `GalacticEquator` is an alias
of an already-promoted mode. They remain catalogued as honest known gaps.

## Scope

**In scope — remove these six variants and all their surface:**
`BabylonianTrueGeoc`, `BabylonianTrueTopc`, `BabylonianTrueObs`,
`BabylonianHouse`, `BabylonianHouseObs`, `BabylonianSissy`.

**Out of scope — explicitly kept:**

- The five real-but-ungateable deferred modes (`Udayagiri`, `PvrPushyaPaksha`,
  `Sheoran`, `DhruvaGalacticCenterMula`, legacy `GalacticEquator`) stay
  catalogued as `DescriptorOnly` known gaps. Their descriptors are audited for
  the same false-provenance defect and corrected if present, but the modes are
  **not** removed.
- The 48 already-gated modes are not re-touched.
- No new ayanamsa computation, no change to the SE numeric gate, holdout grid,
  precession model, or corpus.

## Design

### 1. Remove the variants (blast radius: 4 code files)

- **`crates/pleiades-types/src/ayanamsa.rs`** — delete the six enum variants and
  their doc comments. The enum is `#[non_exhaustive]`, so external `match`
  exhaustiveness is unaffected; serde derives are optional/feature-gated and the
  variants never produced real values, so no meaningful persisted data breaks.
- **`crates/pleiades-ayanamsa/src/catalog.rs`** — delete the six descriptors
  (definitions + aliases) from `RELEASE_AYANAMSAS`; delete them from
  `BUILT_IN_AYANAMSAS` and change the array length `[AyanamsaDescriptor; 59]` →
  `[AyanamsaDescriptor; 53]`. `BASELINE_AYANAMSAS` is unaffected (none are
  baseline).
- **`crates/pleiades-ayanamsa/src/catalog/tests.rs`** — update the completeness
  invariant `built_in.len() == baseline_ayanamsas().len() +
  release_ayanamsas().len()` (it stays valid by construction once both lists
  shrink) and drop per-mode descriptor/alias/epoch assertions referencing the
  six.
- **`crates/pleiades-ayanamsa/src/tests.rs`** — drop the six from completeness,
  summary-line, and `resolve_ayanamsa` alias tests.

`thresholds.rs` needs **no** change: it never asserted a mode class for these six
(only the three anchorless modes are asserted `None`). `lookup.rs` needs **no**
change: it does not name the six (they fell through its default path).

### 2. API / semver handling

The workspace is `0.2.0` (pre-1.0). Under semver, a 0.x breaking change bumps the
minor version → **`0.3.0`**. This is a deliberate, recorded breaking change:
removal of six public enum variants. Rationale for **hard delete over
`#[deprecated]`**: deprecation would keep the fabricated surface and its false
descriptors alive in the public API and in serde output; the whole point is to
remove mislabeled, non-functional surface. The cost — re-adding them if SE ever
standardizes these exact names upstream — is cheap and speculative.

### 3. Claims-surface alignment

Every public surface that counts or describes the catalog updates consistently;
`compat-claims-audit` must re-pass bidirectionally:

- **`README.md`** (the catalogued-count line): "59 catalogued ayanamsas … the
  remaining 11 are catalogued with metadata only" → **"53 catalogued ayanamsas …
  the remaining 5 are catalogued with metadata only."** (48 release-grade is
  unchanged.)
- **`PLAN.md`** — shrink the deferred-set narrative (Phase 6 ayanamsa note,
  Phase 5 note, Status line) from 11 deferred to the genuine **5**, and record
  slice 4 as done (2026-06-26): the six fabricated observational-Babylonian
  variants removed; catalogued count 59 → 53.
- **`spec/compatibility-catalog.md`** — add a one-line note that six speculative
  observational-Babylonian variants, never part of the committed SE-set scope,
  were removed so the code matches the spec.
- **Compatibility profile / claims audit (`crates/pleiades-validate`,
  `claims/` + `compatibility/`)** — update any enumerated ayanamsa listing and
  re-green `compat-claims-audit`. The six are lowest-tier `DescriptorOnly`, so
  no tier↔evidence↔profile↔prose disagreement is introduced; the only change is
  a shorter catalog.
- **Compatibility-profile version bump** — fold in the bump deferred by slice 3
  §7 ("until the full ayanamsa family lands"). The family has now landed in the
  operative sense: everything promotable is promoted, and the catalog is pruned
  to its committed scope. Bump the profile version wherever it is defined and
  asserted (pinned during planning).

### 4. Error handling / fail-closed behavior

No runtime error paths change. The six never had a computation path; after
removal they are simply not constructible. Existing fail-closed behavior of the
SE numeric gate, the corpus checksum/manifest guards, and the overclaim audit is
untouched. Removing catalog entries cannot broaden any claim.

### 5. Testing

- Catalog completeness test passes with `BUILT_IN_AYANAMSAS` length 53 and the
  updated baseline+release sum.
- `resolve_ayanamsa` no longer resolves the six removed aliases (assert they
  return `None`, replacing the prior positive assertions).
- A test (or the existing deferred-set guard) asserts the **five** remaining
  `DescriptorOnly` modes are still present and still `DescriptorOnly` — guarding
  against accidentally removing a real, spec'd mode.
- `compat-claims-audit`, `release-smoke`, and `release-gate` re-run green.
- Workspace builds with no `match` arms left referencing the removed variants.

## Acceptance criteria

- The six variants are absent from `Ayanamsa`, `RELEASE_AYANAMSAS`, and
  `BUILT_IN_AYANAMSAS` (length 53); the workspace builds and all tests pass.
- No descriptor in the catalog claims a Swiss Ephemeris label that SE does not
  define (verified for the surviving deferred modes too).
- `README.md`, `PLAN.md`, `spec/compatibility-catalog.md`, and the compatibility
  profile all reflect 53 catalogued / 48 release-grade / 5 deferred, and the
  profile version is bumped.
- The workspace version is `0.3.0` with the breaking change recorded.
- `compat-claims-audit` passes bidirectionally; `release-gate` is green.
- The five real-but-ungateable deferred modes remain catalogued and
  `DescriptorOnly`.

## Open questions

None blocking. The exact location of the compatibility-profile version constant
and any profile-side ayanamsa enumeration are pinned during implementation
planning (they live under `crates/pleiades-validate`). The scope decision —
remove the six fabricated modes, keep the five real ones — is settled.
