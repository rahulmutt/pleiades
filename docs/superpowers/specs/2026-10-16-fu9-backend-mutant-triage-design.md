# FU-9 slice: `pleiades-backend` mutant triage (design)

**Status:** done · **Opened:** 2026-10-16 · **Issue:** #162 (slice 1 of the
remaining queue) · **Crate:** `pleiades-backend`

## Goal and scope

Drive the surviving mutants in `pleiades-backend` to zero or a documented
equivalent, using tests that check an independent reference: a hand-worked
expected value, a documented rule, or a crafted input that reaches one branch.
The crate joins `[tasks.mutants]` when the slice lands.

Non-goals: no runtime behavior change, no parity-gate change, and the mutants
tier stays report-only. The only non-test source edits are rustdoc that states
two rules the tests check (below) and the `#[cfg(test)]` wiring of two new
test files.

## Measured baseline and result

```bash
cargo mutants -p pleiades-backend --test-tool nextest --test-workspace=false --baseline run
```

| When | Mutants | Missed | Caught | Unviable |
|------|---------|--------|--------|----------|
| 2026-07-25 roadmap | 263 | 70 | 145 | 48 |
| 2026-10-16 before (b664daeb) | 269 | 69 | 147 | 53 |
| 2026-10-16 after | 269 | 1 | 215 | 53 |

The one survivor is the documented equivalent below.

## Why the mutants survived

- **The combinators were tested only on their happy path.** Every stub had the
  same default capabilities, frames, time scales and data sources, so the
  `&&`/`&=` merging, `combine_sources` and `intersect_strings` produced the
  same metadata whatever the operator. `CompositeBackend::supports_body` and
  `RoutingBackend::supports_body` were never called directly (their `position`
  asks each provider, not itself), and no stub failed with a non-retryable
  error.
- **`test_backend::LinearSunMoon` is tested downstream only.** It is the
  analytic Sun/Moon fixture behind the `test-backend` feature, used by the
  events and eclipse crates' tests, which `--test-workspace=false` does not
  run.
- **Single-tier fixtures.** Every claim-tier filter saw claims at one tier
  only, so `==` and `!=` returned the same set.
- **One-branch guards.** `FrameTreatmentSummary` validation, `validate_non_blank`,
  the capability and apparentness guards and the zodiac rejection message
  each lacked the input that reaches the other side.

## Kills

| Survivors | Site | Test (file) | Reference |
|-----------|------|-------------|-----------|
| 8 `&&`→`\|\|` | `CompositeBackend::metadata` flags, `deterministic`, `offline` | `composite_metadata_keeps_only_what_both_providers_offer` (`traits_tests.rs`) | An all-true and an all-false provider; every merged flag must be false |
| 7 `&=`→`\|=` | `RoutingBackend::metadata` | `routing_metadata_keeps_only_what_every_provider_offers` | Same, with the all-true provider first, since the fold starts from it |
| 5 | `combine_sources` (3 replacements, `delete !`, `==`→`!=`) | both metadata tests | `[alpha, shared] + [shared, beta]` = `[alpha, shared, beta]` by hand |
| 1 | `intersect_strings` → `vec![]` | both metadata tests | Frames `[Ecliptic, Equatorial] ∩ [Equatorial]`, scales `[TT, TDB] ∩ [TDB]` |
| 6 | Both `supports_body` (`true`, `false`, `\|\|`→`&&`) | `combinators_support_a_body_that_any_provider_supports` | Sun and Moon served, Mars not |
| 3 | `RoutingBackend::backends`, `is_empty` | `routing_backend_reports_its_provider_chain` | Two providers in order; an empty router |
| 2 | Composite guard → `true`, `&&`→`\|\|` (`traits.rs:103`) | `composite_returns_a_non_retryable_primary_error`, `composite_does_not_retry_on_a_secondary_without_the_body` | Documented fallback rule; the stub's `position` answers `Ok` for any body, so only the support check stops the retry |
| 2 | Routing guard → `true`; `should_fallback_to_secondary` → `true` | `routing_stops_at_a_non_retryable_error` (and the composite test) | `OutOfRangeInstant`, `NumericalFailure` are not retried |
| 13 | `LinearSunMoon` | `test_backend_tests.rs` | Closed form `ref + offset + rate·dt`, worked by hand (table below) |
| 9 | `FrameTreatmentSummary` and its error's `Display` | `policy/frame_tests.rs` | Compact, blank, padded, `\n` and `\r`-only lines; the variants' messages |
| 5 | `release_grade_bodies`, `claims_by_tier`, `merge_body_claims` `>`→`>=` | `metadata_tests.rs` | One claim per tier; two equal-tier claims with different evidence keep the first |
| 3 | `ReleasePosture::claims_for_tier`, `summary_line` | `release_posture_tests.rs` | Four tiers over two backends; the summary written out |
| 1 | `capabilities.rs:61` `&&`→`\|\|` | `one_value_mode_is_enough` | Apparent-only and mean-only are valid |
| 1 | `validation.rs:8` `\|\|`→`&&` | `validate_non_blank_rejects_empty_and_padded_text` | `""` equals its own trim, so only the blank check rejects it |
| 3 | `current.rs:63`, `current.rs:246`, `request.rs:324` | `request_tests.rs` | An apparent request to an apparent backend; a sidereal-only backend's message; a Mars request against Sun-only metadata |

### `LinearSunMoon` margins, per mutant

Longitude displacement in degrees at `dt = 10` days, against the tests' 1e-9°
tolerance. Each mutant has its own row; "Moon +30" is the white-box fixture
with a 30° Moon offset.

| Mutant | Sun | Moon (new) | Moon (+30) |
|--------|-----|------------|------------|
| 104:48 `-`→`+` (`dt`) | 75.949 | 175.464 | 175.464 |
| 104:48 `-`→`/` (`dt`) | 8.871 | 118.588 | 118.588 |
| 121:66 `+`→`-` (`ref ∓ offset`) | 0 | 0 | 60.000 |
| 121:66 `+`→`*` | 100.000 | 100.000 | 10.000 |
| 121:83 `+`→`-` | 19.713 | 96.472 | 96.472 |
| 121:83 `+`→`*` | 155.791 | 15.368 | 52.449 |
| 121:90 `*`→`+` | 1.129 | 108.588 | 108.588 |
| 121:90 `*`→`/` | 9.758 | 130.446 | 130.446 |

The true minimum over each mutant's killing assertions is 1.129° (121:90
`*`→`+`, Sun). 121:66 `+`→`-` is killed by the "+30" fixture alone: the public
constructors offer offsets of 0° and 180° only, and `ref − offset` equals
`ref + offset` modulo 360 for both, so through the public API that mutant is
equivalent. The test builds the fixture with private fields, which the
in-crate test module can reach. The two deleted-match-arm mutants (106, 107),
`delete !` (93) and the two `supports_body` replacements are caught by an
`Ok`/`Err` or `Some`/`None` change, not a displacement.

## Documented equivalent

- **`traits.rs` `retag_instant`, guard `instant.scale != scale` → `true`.** The
  guarded arm returns `Instant::new(instant.julian_day, scale)`. With the
  guard forced true it also runs when `instant.scale == scale`, and then
  rebuilds the instant from its own two fields. `Instant` is exactly
  `{ julian_day, scale }` (`pleiades-types/src/time.rs`), so the result equals
  the input field for field. No input distinguishes the two. Left visible,
  not `#[mutants::skip]`-suppressed.

## Rules made explicit

Two behaviors the tests pin had no written statement, so the tests would
otherwise check the code against itself. Both are now in the rustdoc, with
behavior unchanged:

- **Which errors a router retries** (`RoutingBackend`, referenced from
  `CompositeBackend`): the nine kinds about request shape or provider
  coverage are retried; `OutOfRangeInstant` and `NumericalFailure` are
  returned at once. This sits against `spec/backend-trait.md`'s "select a
  preferred backend based on date range": a chain whose first provider is
  out of range does not fall through to the next. That is existing behavior,
  and #202 depends on it: its gate requires every asteroid refusal from the
  default chain to reach the caller as `OutOfRangeInstant` naming
  `SpkBackend`. It is not changed in a triage slice; it is raised on #162
  instead.
- **`merge_body_claims` on an equal tier** keeps the claim from its first
  argument, as a router keeps its earlier provider.
