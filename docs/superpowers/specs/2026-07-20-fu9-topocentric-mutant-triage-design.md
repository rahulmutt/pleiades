# FU-9 fifth slice — `topocentric.rs` mutant triage

**Date:** 2026-07-20
**Status:** design approved, pre-implementation
**Origin:** [FU-9](../../follow-ups.md) — cargo-mutants surviving-mutant triage backlog
**Baseline:** [`notes/2026-07-18-mutants-baseline.md`](notes/2026-07-18-mutants-baseline.md)
**Predecessor slices:**
[`2026-07-19-fu9-nutation-mutant-triage-design.md`](2026-07-19-fu9-nutation-mutant-triage-design.md),
[`2026-07-19-fu9-apparent-mutant-triage-design.md`](2026-07-19-fu9-apparent-mutant-triage-design.md),
[`2026-07-20-fu9-refraction-mutant-triage-design.md`](2026-07-20-fu9-refraction-mutant-triage-design.md),
[`2026-07-20-fu9-aberration-mutant-triage-design.md`](2026-07-20-fu9-aberration-mutant-triage-design.md)
**Target file:** `crates/pleiades-apparent/src/topocentric.rs` (27 survivors)

---

## 1. Context

FU-9 is the report-only mutation-testing backlog. Four slices are complete:
`nutation.rs` (45 → 1 documented equivalent), `apparent.rs` (10 → 0),
`refraction.rs` (37 → 3 documented equivalents), and `aberration.rs` (28 → 0).
They established the reusable method — **regenerate → classify → address →
verify** — and an independence discipline: every expected value must come from
an independent reference, never from the code's own output.

This is the fifth slice, against `topocentric.rs`, the top remaining file.

`topocentric.rs` is a **single-pass geometric pipeline**: one public function,
`topocentric_position`, converts a geocentric apparent ecliptic position to
equatorial, subtracts the observer's WGS84 rectangular vector (diurnal
parallax, Meeus ch. 11 + ch. 40), applies the classical diurnal-aberration
terms (`0.3192″ · ρcosφ′`), converts back to ecliptic, and records provenance
deltas. All 27 survivors are inside this one function; the helper geometry it
calls (`parallax.rs`) has no survivors.

Its survivor profile is closest to `refraction.rs`: the mutants are reachable
through the public API and survive because the existing four tests assert only
**sign-free magnitudes and loose bounds** — no exact value, no provenance
field, no wrap crossing, and a test observer whose geometry makes one factor
literally unmutatable (§6). This slice is therefore **tests-only**: no refactor
is needed, unlike `apparent.rs` and `aberration.rs`, because no survivor's
effect is below what an exact-literal assertion through the public API can
constrain (the smallest anticipated divergence is ~0.02″, more than three
orders of magnitude above the assertion tolerance).

## 2. Goal & scope

**Goal:** drive `topocentric.rs` from 27 surviving mutants to 0, or to a small
set of *documented equivalent* mutants, using tests that express intent against
independent references.

**In scope:**

- a relocated, expanded white-box test suite at `src/topocentric/tests.rs`;
- FU-9 progress notes in `docs/follow-ups.md`.

**Out of scope / non-goals:**

- any source change to `topocentric.rs` beyond relocating the inline
  `#[cfg(test)] mod tests` (tests-only slice);
- any change to the two guard error payloads — splitting the shared
  `stage: "topocentric"` string into two distinct stages would kill the two
  guard mutants (§5.1) but is a behavior change smuggled in to satisfy a
  mutation score, exactly what the report-only posture forbids;
- any parity-gate change (`validate-*`); the mutants tier stays **report-only**;
- `#[mutants::skip]` suppression of numeric mutants.

## 3. Baseline

Regenerated with the method's authoritative per-file command:

```
cargo mutants -p pleiades-apparent --test-tool nextest \
  --test-workspace=false --file crates/pleiades-apparent/src/topocentric.rs
```

Result: **82 mutants tested, 27 missed, 54 caught, 1 unviable.** This exactly
confirms the whole-workspace baseline figure of 27 for this file — the first
slice where the two invocations agree with no reconciliation note needed.

## 4. Survivor classification & treatment

Line numbers refer to `topocentric.rs` at the baseline commit.

| Group | Location | Count | Root cause |
| --- | --- | --- | --- |
| P — parallax subtraction signs | L53–55 (`- → +` ×3) | 3 | only `hypot` magnitude asserted — sign-free |
| H — hour angle + diurnal-aberration term | L67 (×2), L69 (×4), L71 (×2), L72, L73 | 10 | only a `< 0.36″` bound asserted; test observer degenerate (§6) |
| D — topocentric distance output | L78 (`/ → %`, `/ → *`) | 2 | output `distance_au` never asserted |
| W — provenance Δlon wrap + Δlat + scaling | L83–86 (×8), L88, L104 | 10 | no test crosses 0°/360°; provenance lon/lat fields never asserted |
| G — non-finite guards | L57, L95 (`\|\| → &&` ×2) | 2 | equivalent-mutant candidates (§5.1) |
| **Total** | | **27** | |

**Groups P, H, D and the two non-wrap W mutants (L88, L104) — 17 in all — are
killed by one exact-literal test** at a crafted discriminating geometry (§6):
the primary test asserts the output ecliptic longitude, latitude, and
`distance_au` **and all four provenance fields** against independently
computed literals. Kill margins: a parallax
sign flip moves the output by up to ~2°; the distance `/ → *` swap by a factor
of ~5.5×10⁸; the smallest divergence in the group (the `/ → %` on
`cos δ` at L69) is ~0.02″ — all far above the 1e-9° / 1e-6″ assertion
tolerances.

**The four wrap-branch `==`/compound mutants** (L83 `> → ==`, L84 `-= → +=`,
`-= → /=`, L85 `< → ==`, L86 `+= → -=`, `+= → *=`) require the wrap branches
to actually fire, which needs the topocentric longitude to cross the 0°/360°
boundary; two dedicated wrap-crossing tests provide that (§6). Under a fired
branch the mutants diverge grossly (an unwrapped Δlon is ~±359.9° ≈
1.3 × 10⁶″ in provenance).

The remaining two `> → >=` / `< → <=` forms and both guard mutants are the
equivalent-mutant candidates, next.

## 5. Equivalent-mutant analysis

### 5.1 The L95 guard `|| → &&` mutant — equivalent

**L95 (final guard, `!lon.is_finite() || !lat.is_finite()`).** The
nutation-precedent shape: distinguishing `||` from `&&` needs **exactly one**
of the two operands non-finite, but `to_ecliptic` mixes RA and Dec into both
outputs, so any non-finite reaching L95 poisons longitude and latitude
together. The only exactly-one route would be an infinite `aberr_ra` from
`cos(dec_topo) == 0.0` — which requires `tz / topo_distance == 1.0` in exact
f64, unreachable from any physical input. This guard's mutant remains a
documented equivalent, returning the byte-identical error value,
`ApparentPlaceError::NonFiniteCorrection { stage: "topocentric" }`, on every
reachable input.

**Correction — L57 is not equivalent; it is killed.** The original analysis
here (early guard, `!topo_distance.is_finite() || topo_distance <= 0.0`)
considered only NaN inputs and the `<= 0.0` disjunct in isolation, and
concluded both routes collapse into the same L95 error. It missed a third
route: a **finite** but astronomically large `distance_au` (e.g. `1e301` AU)
makes `d = distance_au * AU_IN_EARTH_RADII` finite but pushes
`tx*tx + ty*ty + tz*tz` past `f64::MAX`, so `topo_distance` overflows to
**`+inf`**. Under the `||`→`&&` mutant, `!topo_distance.is_finite()` is true
but `topo_distance <= 0.0` is false (`inf <= 0.0` is false), so the `&&`
guard does *not* fire. Execution falls through with `topo_distance = inf`;
`tz / inf` and `tx / inf`-style divisions all evaluate to `0.0`, which is
finite, so `dec_topo`, both aberration terms, and the ecliptic reconversion
all stay finite and the L95 guard never fires either. The unmutated function
returns `Err(NonFiniteCorrection { stage: "topocentric" })` at L57; the
mutant returns `Ok(TopocentricPosition { distance_au: Some(inf), .. })` —
observably different. Per this spec's own rule (§8), a mutant found killable
during implementation is killed, not documented: it is killed by
`overflowing_distance_fails_closed` in `topocentric/tests.rs`.

### 5.2 The wrap comparison forms (L83 `> → >=`, L85 `< → <=`) — equivalent

These differ from the original only when the raw Δlon equals **exactly**
`±180.0`. The parallax + diurnal-aberration shift is bounded well under 2°
(Moon horizontal parallax ≈ 0.95°), so the raw difference of the two wrapped
longitudes lies in (−2°, 2°) ∪ ±(358°, 360°); ±180.0 exactly is unreachable.
This is an *unreachability* argument (like `nutation.rs`'s equivalent), not
`refraction.rs`'s identical-expression argument — at Δlon = 180.0 the branches
*would* differ, but no input produces that value. The `≪ 2°` bound holds for
any `distance_au` at or beyond the observer's geocentric radius (~1 Earth
radius, the physical regime); a nonphysical sub-Earth-radius distance could in
principle produce a much larger raw Δlon, but no such input pins the value to
exactly ±180.0 either, so the practical-equivalence verdict is unaffected.

**Expected residual: 3 documented equivalents** (L95, L83 `>=`, L85
`<=`), left visible and documented in `docs/follow-ups.md`, not
`#[mutants::skip]`-suppressed. If implementation finds any of them killable
after all, it is killed instead of documented — as happened with L57 (§5.1).

## 6. Reference strategy (independence discipline)

**Independent recomposition, computed outside the crate.** Expected literals
come from a one-off scratchpad script that reimplements the published pipeline
— Meeus ch. 11 observer terms (`ρsinφ′`, `ρcosφ′` on WGS84), the ch. 40
rectangular parallax subtraction, the classical diurnal-aberration terms
(`Δα = 0.3192″ ρcosφ′ cos H / cos δ`, `Δδ = 0.3192″ ρcosφ′ sin H sin δ`), and
the standard ecliptic↔equatorial rotations — **without calling the crate**.
The script's f64 results are pinned as literals with the derivation documented
in test comments. Assertion tolerances: `1e-9°` on coordinates, `1e-6″` on
provenance fields; the closest killable mutant diverges by ~0.02″ ≈ 5×10⁻⁶ °,
comfortably separated. During implementation, each killable mutant is
confirmed to diverge under the chosen geometry before the literals are pinned
(the per-file cargo-mutants re-run is the authoritative check).

**Primary discriminating geometry** (proposed; the script fixes the final
values and verifies the checklist):

```
observer = Palomar (φ = +33.356111°, elevation 1706 m)   → ρcosφ′ ≈ 0.836339
body     = λ = 100°, β = +5°, Δ = 0.00257 AU (Moon-scale)
ε        = 23.44°,  LAST = 70°
```

Properties, each load-bearing:

- **`ρcosφ′ ≈ 0.836 ≠ 1`** — kills the `* → /` mutants on
  `rho_cos_phi_prime` (L69:35, L71:35). Palomar is Meeus ch. 11's own worked
  example and is already pinned in `parallax.rs`'s tests.
- **`dec_topo ≈ 28° ≠ 0`** — `cos δ ≠ 1` separates `/ cos δ` from both
  `% cos δ` and `* cos δ` (L69), and `sin δ ≠ 0` keeps the Dec term alive
  (L71).
- **`H ≈ −31° (→ 329°)`** — both `cos H` and `sin H` non-zero, so no
  aberration term vanishes; the raw `LAST − α` is negative, so `rem_euclid`
  genuinely wraps (no mutant targets it, but the branch is then exercised
  rather than a no-op).
- **Moon-scale distance** — parallax ~1°, diurnal term ~0.2″: every mutated
  term moves the output far above tolerance.

**Rejected geometries (recorded so they are not re-proposed):**

- **Equator/sea-level observer** (`φ = 0`, `h = 0`) — the existing tests' choice,
  and the root cause of the H-group survivors: it makes `ρcosφ′ = 1.0`
  exactly, so `* rho_cos_phi_prime → / rho_cos_phi_prime` is **bit-identical**.
- **`β` (and `dec_topo`) ≈ 0** — makes `cos δ = 1` (collapsing `/ → %` and
  `/ → *` on L69 to near-identity) and `sin δ = 0` (annihilating the L71 Dec
  term entirely).

**Wrap-crossing geometries.** Two tests at Palomar, Moon-scale distance, low
latitude, with body longitude just above 0° and just below 360°
(`λ = 0.02°` / `λ = 359.98°`), LAST chosen by the script so the parallax
shift carries the topocentric longitude across the boundary — westward
(Δlon raw ≈ +359.9°, exercising the `> 180 → −360` branch) and eastward
(Δlon raw ≈ −359.9°, exercising the `< −180 → +360` branch). Each asserts the
exact provenance `parallax_longitude_arcsec` literal (small, correctly
signed). The near-zero `dec_topo` degeneracy is acceptable *here* because
these tests only carry the wrap-mutant kills; the primary geometry carries the
aberration-term kills.

**Fail-closed test.** NaN `local_sidereal_time_deg` and NaN `obliquity_deg`
each return `Err(NonFiniteCorrection { stage: "topocentric" })` — pinning the
fail-closed intent of the guards even though it cannot distinguish the two
`||` mutants (§5.1).

## 7. Test relocation & inventory

Per AGENTS.md, the inline `#[cfg(test)] mod tests` moves to
`crates/pleiades-apparent/src/topocentric/tests.rs`, matching
`apparent/tests.rs`, `nutation/tests.rs`, `refraction/tests.rs`, and
`aberration/tests.rs`. White-box unit tests with `use super::*`; not converted
to integration tests.

| Test | Carries |
| --- | --- |
| existing four (moon-parallax bound, distant-body bound, missing-distance error, aberration bound) | kept — real physical intent the new tests do not replace |
| `palomar_moon_matches_independent_meeus_pipeline` | groups P, H, D + L88, L104 (17 kills): exact literals on output lon/lat/distance and all four provenance fields |
| `parallax_displaces_toward_horizon` | intent documentation: the topocentric shift is directional, not just non-zero (redundant kill of group P) |
| `wrap_westward_across_zero` | L83 `> → ==`, L84 `-= → +=`, `-= → /=` |
| `wrap_eastward_across_zero` | L85 `< → ==`, L86 `+= → -=`, `+= → *=` |
| `non_finite_inputs_fail_closed` | guard intent (equivalents documented, not killed) |

## 8. Verification & acceptance criteria

1. `cargo mutants -p pleiades-apparent --test-tool nextest
   --test-workspace=false --file crates/pleiades-apparent/src/topocentric.rs`
   reports **3 missed**, each one of the §5 documented equivalents — or fewer,
   if implementation finds one killable.
2. `mise run ci` is green; `cargo fmt` / clippy clean.
3. No source change to `topocentric.rs` other than the test-module relocation.
4. No `validate-*` gate file is touched.
5. No `#[mutants::skip]` is added; equivalents stay visible and documented.

## 9. Deliverables

1. Commit 1 — `test(topocentric): FU-9 mutant triage — exact-literal Meeus
   recomposition, wrap-crossing, and fail-closed tests` (includes the
   relocation to `src/topocentric/tests.rs`).
2. Commit 2 — `docs(follow-ups): record FU-9 topocentric.rs triage
   (27 → 3 documented equivalents)`.

Branch `fu9-topocentric-mutant-triage`, PR flow as in prior slices.

## 10. Follow-on (out of scope here, tracked in FU-9)

Remaining slices in priority order: `sidereal.rs` (17), `precession.rs` (17),
`lighttime.rs` (5), then the `pleiades-time` and `pleiades-types` survivors.
