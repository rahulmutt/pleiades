# FU-9 slice: `pleiades-ayanamsa` mutant triage (design)

**Status:** done · **Opened:** 2026-10-16 · **Issue:** #162 (slice 3 of the
remaining queue) · **Crate:** `pleiades-ayanamsa`

## Goal and scope

Drive the surviving mutants in `pleiades-ayanamsa` to zero or a documented
equivalent, using tests that check an independent reference: a value worked out
by hand from the source the code cites, a Swiss Ephemeris reference row, or a
crafted input that reaches one branch. The crate joins `[tasks.mutants]` when
the slice lands.

Non-goals: no production change, no parity-gate, corpus or tolerance change in
`pleiades-validate`, no `#[mutants::skip]`. The only non-test edits are the CI
wiring (`mise.toml`, `mutants.yml`) and the roadmap row.

## Measured baseline and result

```bash
cargo mutants -p pleiades-ayanamsa --test-tool nextest --test-workspace=false --baseline run
```

| When | Mutants | Missed | Caught | Unviable |
|------|---------|--------|--------|----------|
| 2026-07-25 roadmap | 305 | 85 | 196 | 24 |
| 2026-10-16 before (feb59f62) | 359 | 85 | 221 | 53 |
| 2026-10-16 after | 359 | 2 | 304 | 53 |

The two survivors are the documented equivalents below.

## Why the mutants survived

- **Published zero points were never pinned.** The catalog and `se_anchors.rs`
  hold expressions such as `360.0 - 341.33904` and negative literals such as
  `-0.79167046`. Existing tests checked drift and ordering, not the value, so
  `-`→`+`/`/` and `delete -` changed the zero point unseen. The catalog also
  carries two copies of each descriptor (`RELEASE_AYANAMSAS` and
  `BUILT_IN_AYANAMSAS`); `descriptor()` only reaches the second, so the first
  copy was untested.
- **The cubic coefficients were tested for `Some`/`None` only.** The true-star
  and galactic tests asserted the fit window and that a value exists, never the
  value; the arms for True Citra, Revati, Pushya and Mula fell through to the
  Sheoran fit without notice.
- **`general_precession_longitude_arcsec` was only exercised through a rate
  and a nonlinearity check**, which are insensitive to the cubic term.
- **One-branch guards and `Display` impls.** `AyanamsaDescriptor::validate`
  was tested with inputs that tripped a different clause first; the error and
  summary `Display` impls were only compared with themselves.

## Kills

| Survivors | Site | Test (file) | Reference |
|-----------|------|-------------|-----------|
| 25 | `catalog.rs` zero points: 16 `-`→`+`/`/` on `360.0 - x` and `23.25 - x` (Usha/Shashi, Yukteshwar, Djwhal Khul, Bhasin, Lahiri ICRC; release and built-in tables), 9 `delete -` (SS Revati, Hipparchus, Babylonian Huber, SS mean Sun, Aryabhata mean Sun, Valens; both tables) | `descriptor_zero_points_match_published_values_in_both_catalog_tables` (`tests/zero_points.rs`) | `sweph.h` `ayanamsa[]` rows 4, 6, 7, 8, 25, 22, 24, 42, 46 evaluated by hand (`360 - 341.33904 = 18.66096`); Hipparchus `-28/3`; Huber `-434/3600` (no `sweph.h` source, see below) |
| 22 | `se_anchors.rs`: 10 `-`→`+`/`/`, 12 `delete -` | `se_anchor_table_matches_sweph_h_rows` | All 29 rows transcribed from SE 2.10.03 `sweph.h` (`libswisseph-sys-0.1.2`), `SE_SIDM`, `t0`, `ayan_t0`, `t0_is_UT` |
| (both) | Descriptor vs anchor drift | `pinned_descriptors_agree_with_se_anchors_except_documented_divergences` | The two tables must agree for the 9 pinned non-divergent modes |
| 4 | `galactic.rs` `delete -` on the `c3` of IAU 1958, True, Mula and Mula/Wilhelm | `galactic_cubics_reproduce_swiss_ephemeris_rows` (`tests/numeric.rs`) | `ayanamsa-corpus/ayanamsa.csv` rows at JD 2451545 and 2488070 |
| 3 | `truestar.rs` `eval` (`+`→`-` at col 31, `+`→`-` and `+`→`*` at col 51) | `true_star_cubics_reproduce_swiss_ephemeris_rows` and the galactic test (every cubic goes through `eval`) | Same corpus rows |
| 4 | `truestar.rs` deleted arms Citra, Revati, Pushya, Mula | `true_star_cubics_reproduce_swiss_ephemeris_rows` | Each mode must return its own SE value; the fall-through is the Sheoran fit, 0.64 (Mula) to 5.19 (Revati) degrees away at J2000 |
| 6 | `general_precession_longitude_arcsec` | `general_precession_matches_the_p03_polynomial` | IAU 2006 `pA(T)` (Capitaine et al. 2003) hand-summed at T = 1, -2, 3, 10 |
| 4 | `offset_from_components` | `linear_offset_adds_the_precession_rate_per_julian_century` | `offset + centuries * 1.3969712777...` at +2, -1, +0.5 centuries |
| 2 | `validate_ayanamsa_catalog_entries` `+=`→`*=` | `entry_validation_counts_every_canonical_name_and_alias` | A crafted slice of two descriptors with 3 + 2 labels; `built_in_validation_summary_counts_every_label` adds the whole-catalog count |
| 2 | `format_ayanamsa_label_list` → `String::new()`, `"xyzzy"` | `format_label_list_renders_none_or_a_comma_separated_list` | The documented `none` / comma-joined text |
| 3 | `AyanamsaDescriptor::validate`: `\|\|`→`&&` at 77, 103; `delete !` at 103 | `validate_rejects_a_canonical_name_with_surrounding_whitespace`, `validate_rejects_notes_with_surrounding_whitespace` | A name or note that only the padding clause rejects |
| 1 | `validate` `\|\|`→`&&` at 117 | `validate_rejects_an_alias_identical_to_the_canonical_name`, `validate_allows_one_case_variant_of_the_canonical_name_but_not_two` | An identical alias; two case variants |
| 1 | `has_sidereal_metadata` `&&`→`\|\|` | `has_sidereal_metadata_needs_both_epoch_and_offset` | Epoch only, offset only |
| 1 | `AyanamsaMetadataCoverage::is_complete` → `true` | `metadata_coverage_is_complete_only_without_missing_entries` | One entry in `without_sidereal_metadata` |
| 5 | `Display` impls (provenance summary and its error, coverage error, catalog error, validation summary) | `provenance_summary_display_joins_examples_with_semicolons`, `provenance_validation_error_display_names_the_field`, `metadata_coverage_error_display_is_exact_for_every_variant`, `catalog_error_display_is_exact_for_every_variant`, `validation_summary_display_reports_an_error_result` | Message text written out |

Descriptors that are not pinned to `sweph.h` by the zero-point table (for
example DeLuce, whose catalog epoch is J2000, and Hipparchus and Babylonian
(Huber), which keep their own historical values) are outside the
descriptor-versus-anchor comparison on purpose: the catalog does not claim to
mirror every `aya_init` row. The Huber zero point (`1721171.5`, `-0.12055...`) has
no `sweph.h` source; its test value is the descriptor's own documented number
written as a literal (`-0 deg 07' 14"`), so it pins the sign, not the
provenance.

## Per-mutant displacement rows

### Zero points (value of the offset in degrees; test tolerance 1e-9)

| Site | `-`→`+` | `-`→`/` | `delete -` |
|------|---------|---------|------------|
| Usha/Shashi `360 - 341.33904` = 18.66096 | 682.68 | -17.606 | - |
| Djwhal Khul `360 - 333.0369024` = 26.9630976 | 666.07 | -25.882 | - |
| Yukteshwar `360 - 338.917778` = 21.082222 | 677.84 | -20.020 | - |
| Bhasin `360 - 338.634444` = 21.365556 | 677.27 | -20.302 | - |
| Lahiri ICRC `23.25 - 0.00464207` = 23.24535793 | 0.009284 | 4985.3 | - |
| SS Revati -0.79167046 | - | - | 1.58334 |
| SS mean Sun -0.21463395 | - | - | 0.42927 |
| Aryabhata mean Sun -0.23763238 | - | - | 0.47526 |
| Valens -2.9422 | - | - | 5.8844 |
| Hipparchus -9.333... | - | - | 18.667 |
| Huber -0.12055... | - | - | 0.24111 |

The anchor table's 29 rows carry the same operators on the same constants
(`delete -` on 12 negative anchors, 5 expression anchors) and the same or larger
displacements (for example Kugler 1, -5.66667: 11.333). The true minimum is
**0.009284 degrees** (Lahiri ICRC `-`→`+`, because the subtracted nutation term
is only 0.00464207), seven orders above the 1e-9 tolerance. Every row is checked
at both catalog tables for the descriptors and once for the anchor.

### `general_precession_longitude_arcsec` (arcseconds; tolerance 1e-9)

Each mutant has its own row. Column labels are the source columns of the
operator in `5028.796195 * t + 1.1054348 * t * t + 0.000_079_64 * t * t * t`.

| Mutant | T = 1 | T = -2 | T = 3 | T = 10 |
|--------|-------|--------|-------|--------|
| 11:21 `+`→`-` | -2.21087 | -8.84348 | -19.8978 | -221.087 |
| 11:33 `*`→`+` (`(c2+t)*t*t`) | 1.0 | -8.0 | 27.0 | 1000.0 |
| 11:41 `+`→`-` | -1.5928e-4 | 1.27424e-3 | -4.30056e-3 | -0.15928 |
| 11:56 `*`→`+` | 1.0 | -7.99904 | 26.99857 | 999.928 |
| 11:60 `*`→`+` | 1.0 | 4.00096 | 8.99857 | 99.9283 |
| 11:64 `*`→`+` | 1.0 | -1.99904 | 2.99857 | 9.92832 |

The true minimum is **1.5928e-4 arcsec** (11:41, T = 1; that is `2 * 7.964e-5`),
five orders above the 1e-9 tolerance, which sits an order above f64 rounding at
the largest value (5e4 arcsec, about 7e-12). The cubic-term mutants need only
T = 1, but the three values at |T| > 1 also separate `*`→`+` mutants that happen
to coincide at T = 1.

### `eval` and the galactic `c3` flips (degrees; tolerance 2e-8)

Rows are at JD 2488070 (T = 1); at JD 2451545 (T = 0) every mutant
displacement is exactly 0, so that row pins only the constant term. The fit
residual against the SE row, measured at the same two epochs, is at most 6.7e-9
(Wilhelm) and about 1e-9 for the rest, so the tolerance sits above the residual
and below every row marked as killing.

| Mode | 58:31 `+`→`-` | 58:51 `+`→`-` | 58:51 `+`→`*` | galactic `delete -` on `c3` |
|------|---------------|---------------|---------------|------------------------------|
| True Chitra / Citra | -6.15e-4 | -7.09e-8 | -3.07e-4 | - |
| True Revati | -6.14e-4 | -5.61e-8 | -3.07e-4 | - |
| True Pushya | -6.13e-4 | -5.75e-8 | -3.07e-4 | - |
| True Mula | -6.41e-4 | -6.12e-8 | -3.20e-4 | - |
| True Sheoran | -6.13e-4 | -5.75e-8 | -3.07e-4 | - |
| Galactic Center / Rgilbrand / Cochrane | -6.25e-4 | -7.08e-8 | -3.12e-4 | - |
| Galactic Equator IAU 1958 | -5.97e-4 | +1.11e-7 | -2.99e-4 | +1.11e-7 (28:5) |
| Galactic Equator True | -5.97e-4 | +1.11e-7 | -2.99e-4 | +1.11e-7 (34:5) |
| Galactic Equator Mula | -5.97e-4 | +1.11e-7 | -2.99e-4 | +1.11e-7 (40:5) |
| Galactic Center Mardyks | -6.14e-4 | -7.4e-9 | -3.07e-4 | - |
| Galactic Center Mula/Wilhelm | -9.74e-4 | +1.40e-5 | -4.80e-4 | +1.40e-5 (52:5) |
| Galactic Equator Fiorenza | -6.14e-4 | -7.4e-9 | -3.07e-4 | - |

The true minimum over the rows that carry a mutant's kill is **5.61e-8**
(58:51 `+`→`-`, True Revati) and **1.11e-7** for the three smallest galactic
`delete -` mutants, against a 2e-8 tolerance. Mardyks and Fiorenza have a `c3`
near 3.7e-9, so their 58:51 `+`→`-` row (7.4e-9) is below the tolerance and does
not by itself kill that mutant; the other thirteen modes do. This is why the
tolerance is 2e-8 and not tighter.

Why SE rows with a tolerance of 2e-8 degrees (7.2e-5 arcsec) rather than the
gate's 1.0 arcsec ceiling: the gate's ceiling is a release criterion, sized
so that the cubic fits pass with margin (the measured maxima are 0.057 arcsec
at worst). A sign flip on a cubic coefficient moves the value by about 1e-7
degrees at the window edge, so the ceiling cannot see it. The committed fits
reproduce the T = 0 and T = 1 rows to 7e-9 degrees, so the tighter bound is
measured, not assumed, and the corpus file stays unchanged.

### `offset_from_components` (degrees; tolerance 1e-12)

Expected values: `5 + c * 1.3969712777777778` for `c` = 2, -1, 0.5 centuries.

| Mutant | c = 2 | c = -1 | c = 0.5 |
|--------|-------|--------|---------|
| 347:64 `/`→`%` | -2.79394 | +1.39697 | +25511 |
| 347:64 `/`→`*` | +3.727e9 | -1.864e9 | +9.318e8 |
| 349:26 `+`→`-` | -5.58789 | +2.79394 | -1.39697 |
| 349:38 `*`→`/` | -1.36229 | +0.68114 | -0.34057 |

The true minimum is **0.34057** (349:38 at c = 0.5).

### Counts, validation guards and `Display`

These mutants change a boolean, a count or text, not a magnitude, so they are
caught by an equality failure. `+=`→`*=` on the label counters (lines 202 and
218) returns 3 and 2 for the crafted five-label slice, and 2 and 1 for the
three-label one, instead of 5 and 3.

## Documented equivalents

- **`lookup.rs:106` `provenance_summary` → `Default::default()`.**
  `provenance_summary()` is `AyanamsaProvenanceSummary::new()`. The type's
  `Default` implementation (`model.rs`, `impl Default for
  AyanamsaProvenanceSummary`) is `Self::new()`. The mutant therefore calls the
  same function through a different name; no input distinguishes the two, and
  there is no way to change one without the other short of editing `Default`
  itself. Left visible, not `#[mutants::skip]`-suppressed.
- **`lookup.rs:245` `validate_ayanamsa_catalog` → `Ok(())`.** The function takes
  no input and validates the compiled-in `built_in_ayanamsas()` table. That
  table is valid (`validate_ayanamsa_catalog().is_ok()` and the validation
  summary test pin that), so the correct result is `Ok(())` too. The mutant
  differs from the original only when the shipped table is invalid, and a test
  cannot swap the table. The failure paths it forwards are exercised on crafted
  slices through `validate_ayanamsa_catalog_entries` (duplicate labels,
  non-round-tripping labels, partial metadata, label counts), which is the
  function the wrapper delegates to. A mutant that returned `Err` on a valid
  catalog is killed by the existing `is_ok` assertions.

## Notes for follow-up

- The catalog's duplicate descriptor tables (`RELEASE_AYANAMSAS` inside
  `BUILT_IN_AYANAMSAS`) mean every constant is typed twice; the tests check both
  copies. Collapsing them would remove the duplication but is a production
  refactor, out of scope for a triage slice.
- The consistency test covers 9 non-divergent modes. Widening it to the other
  anchor-backed descriptors would need the catalog's different epochs
  (for example DeLuce at J2000) to be reconciled with `sweph.h` first, which is
  a catalog decision.
