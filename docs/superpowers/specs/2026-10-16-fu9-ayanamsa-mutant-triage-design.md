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
| 25 | `catalog.rs` zero points: 16 `-`→`+`/`/` on `360.0 - x` and `23.25 - x` (Usha/Shashi, Yukteshwar, Djwhal Khul, Bhasin, Lahiri ICRC; release and built-in tables), 9 `delete -` (SS Revati, Hipparchus, Babylonian Huber, SS mean Sun, Aryabhata mean Sun, Valens; both tables) | `descriptor_zero_points_match_published_values_in_both_catalog_tables` (`tests/zero_points.rs`) | `sweph.h` `ayanamsa[]` rows 4, 6, 7, 8, 25, 22, 24, 42, 46 evaluated by hand (`360 - 341.33904 = 18.66096`); Hipparchus `-28/3`. The two Babylonian (Huber) `delete -` kills (`catalog.rs:266`, `:795`) are **not source-backed**: a self-referential regression pin, see below |
| 22 | `se_anchors.rs`: 10 `-`→`+`/`/`, 12 `delete -` | `se_anchor_table_matches_sweph_h_rows` | All 29 rows transcribed from SE 2.10.03 `sweph.h` (`libswisseph-sys-0.1.2`), `SE_SIDM`, `t0`, `ayan_t0`, `t0_is_UT` |
| 4 | `galactic.rs` `delete -` on the `c3` of IAU 1958, True, Mula and Mula/Wilhelm | `galactic_cubics_reproduce_swiss_ephemeris_rows` (`tests/numeric.rs`) | `ayanamsa-corpus/ayanamsa.csv` rows at JD 2451545 and 2488070 |
| 3 | `truestar.rs` `eval` (`+`→`-` at col 31, `+`→`-` and `+`→`*` at col 51) | `true_star_cubics_reproduce_swiss_ephemeris_rows` and the galactic test (every cubic goes through `eval`) | Same corpus rows |
| 4 | `truestar.rs` deleted arms Citra, Revati, Pushya, Mula | `true_star_cubics_reproduce_swiss_ephemeris_rows` | Each mode must return its own SE value; the fall-through is the Sheoran fit, 0.64 (Mula) to 5.19 (Revati) degrees away at J2000 |
| 6 | `general_precession_longitude_arcsec` | `general_precession_matches_the_p03_polynomial` | IAU 2006 `pA(T)` (Capitaine et al. 2003) hand-summed at T = 1, -2, 3, 10 |
| 4 | `offset_from_components` | `linear_offset_adds_the_precession_rate_per_julian_century` | `offset + centuries * 1.3969712777...` at +2, -1, +0.5 centuries; the rate is the IAU 1976 precession 5029.0966 arcsec per Julian century (Lieske et al. 1977) over 3600 |
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
mirror every `aya_init` row.

`pinned_descriptors_agree_with_se_anchors_except_documented_divergences` is a
drift guard, not a kill: it keeps the descriptor and anchor tables from
diverging for the 9 pinned non-divergent modes, and no mutant depends on it.

**Self-referential exception: Babylonian (Huber).** The catalog's Huber zero
point (`1721171.5`, `-0.12055...`) has no independent source. The catalog notes
cite no number, no spec or plan names one, a pickaxe search of the history
finds the value only in the commits that introduced it, and `sweph.h` row 12 is
`{1684532.5, -4.46667}` (`sweph.h` also says the Huber mode "had a wrong initial
value until 14 Sept. 2018"). The test literal (`-434/3600`, written
`-0 deg 07' 14"`) is the descriptor's own value converted back, so the two
`delete -` kills at `catalog.rs:266` and `:795` are a regression pin: they catch
a sign slip and say nothing about whether the value is right. The test row is
labelled the same way. (Huber is a fitted-offset mode, so `sidereal_offset`
does not read this zero point; it is catalog metadata only.)

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

The 22 `se_anchors.rs` kills (10 operator, 12 `delete -`) have their own rows.
`delete -` displacement is `2|v|`; the operator rows use the same `a - b`
constants as the catalog (source line in `se_anchors.rs` in brackets).

| Anchor (`SE_SIDM`) | `-`→`+` | `-`→`/` | `delete -` |
|--------------------|---------|---------|------------|
| Usha/Shashi (4) [48] | 682.68 | -17.606 | - |
| Djwhal Khul (6) [58] | 666.07 | -25.882 | - |
| Yukteshwar (7) [68] | 677.84 | -20.020 | - |
| Bhasin (8) [78] | 677.27 | -20.302 | - |
| Lahiri ICRC (46) [318] | 0.0092841 | 4985.3 | - |
| Kugler 1 (9) -5.66667 | - | - | 11.33334 |
| Kugler 2 (10) -4.26667 | - | - | 8.53334 |
| Kugler 3 (11) -3.41667 | - | - | 6.83334 |
| Huber (12) -4.46667 | - | - | 8.93334 |
| Eta Piscium (13) -5.079167 | - | - | 10.15833 |
| Aldebaran (14) -4.44138598 | - | - | 8.88277 |
| Hipparchus (15) -9.33333 | - | - | 18.66666 |
| SS mean Sun (22) -0.21463395 | - | - | 0.42927 |
| Aryabhata mean Sun (24) -0.23763238 | - | - | 0.47526 |
| SS Revati (25) -0.79167046 | - | - | 1.58334 |
| Britton (38) -3.2 | - | - | 6.40000 |
| Valens (42) -2.9422 | - | - | 5.88440 |

These are 5 expression anchors x 2 mutants = 10 and 12 negative anchors. They
are transcribed from `sweph.h`, so the Huber caveat above does not apply to
them. The true minimum over the anchor table is **0.42927** (SS mean Sun
`delete -`); over the catalog and anchor tables together it is **0.009284 degrees** (Lahiri ICRC `-`→`+`, because the subtracted nutation term
is only 0.00464207), seven orders above the 1e-9 tolerance. Every row is checked
at both catalog tables for the descriptors and once for the anchor.

### `general_precession_longitude_arcsec` (arcseconds; tolerance 1e-9)

Each mutant has its own row. Column labels are the source columns of the
operator in `5028.796195 * t + 1.1054348 * t * t + 0.000_079_64 * t * t * t`.

| Mutant | T = 1 | T = -2 | T = 3 | T = 10 |
|--------|-------|--------|-------|--------|
| 11:21 `+`→`-` | -2.21087 | -8.84348 | -19.8978 | -221.087 |
| 11:33 `*`→`+` (inner `*`: `(c2+t)*t`) | 1.0 | -2.63261 | 2.36739 | 0.510868 |
| 11:41 `+`→`-` | -1.5928e-4 | 1.27424e-3 | -4.30056e-3 | -0.15928 |
| 11:56 `*`→`+` (`((c3+t)*t)*t`) | 1.0 | -7.99904 | 26.9986 | 999.928 |
| 11:60 `*`→`+` | 1.0 | 4.00096 | 8.99857 | 99.9283 |
| 11:64 `*`→`+` | 1.0 | -1.99904 | 2.99857 | 9.92832 |

The true minimum is **1.5928e-4 arcsec** (11:41, T = 1; that is `2 * 7.964e-5`),
five orders above the 1e-9 tolerance, which sits two orders above f64 rounding
at the largest value (5e4 arcsec, about 7e-12). Over the other five mutants the
minimum is **0.510868** (11:33, T = 10).

### `eval` and the galactic `c3` flips (degrees; tolerance 2e-8)

`eval` is `((c0 + c1 t) + c2 t t) + c3 t t t`, so the mutants are:
58:31 `+`→`-` (the second `+`: the `c2` term negated), 58:51 `+`→`-` (the `c3`
term negated) and 58:51 `+`→`*` (`(c0 + c1 t + c2 t^2) * (c3 t^3)`). The galactic
`delete -` mutants (28:5, 34:5, 40:5, 52:5) negate one `c3` and move the value
by `2|c3| T^3`, the same as 58:51 `+`→`-` for that mode.

Two tables, one row per mode. First the displacement (mutant minus original);
then the killing margin `|mutant - SE|`, which is what the tolerance has to stay
below, because the tests compare against the SE row. T = 0 is JD 2451545 and
T = 1 is JD 2488070. At T = 0 the 31 and 51 `-` mutants move the value by
exactly 0 (every term but `c0` vanishes), so that row only pins `c0`; the 51
`+`→`*` mutant is displaced by about `-c0` at T = 0.

| Mode | 31 `-` disp (T=1) | 51 `-` disp (T=1) | 51 `*` disp (T=0 / T=1) |
|------|------------------|------------------|--------------------------|
| True Chitra | -6.147e-4 | -7.088e-8 | -23.84 / -25.24 |
| True Citra | -6.147e-4 | -7.088e-8 | -23.84 / -25.24 |
| True Revati | -6.136e-4 | -5.606e-8 | -20.04 / -21.44 |
| True Pushya | -6.135e-4 | -5.750e-8 | -22.72 / -24.12 |
| True Mula | -6.409e-4 | -6.122e-8 | -24.59 / -25.98 |
| True Sheoran | -6.135e-4 | -5.750e-8 | -25.23 / -26.63 |
| Galactic Center | -6.248e-4 | -7.076e-8 | -26.85 / -28.25 |
| Galactic Center Rgilbrand | -6.248e-4 | -7.076e-8 | -22.47 / -23.87 |
| Galactic Center Cochrane | -6.248e-4 | -7.076e-8 | -356.9 / -358.2 |
| Galactic Equator IAU 1958 | -5.974e-4 | +1.112e-7 | -30.02 / -31.43 |
| Galactic Equator True | -5.973e-4 | +1.111e-7 | -30.08 / -31.48 |
| Galactic Equator Mula | -5.973e-4 | +1.111e-7 | -23.41 / -24.81 |
| Galactic Center Mardyks | -6.142e-4 | -7.403e-9 | -30.02 / -31.41 |
| Galactic Center Mula/Wilhelm | -9.742e-4 | +1.402e-5 | -20.05 / -21.50 |
| Galactic Equator Fiorenza | -6.142e-4 | -7.409e-9 | -25.00 / -26.40 |

| Mode | 31 `-` margin | 51 `-` margin (= `c3` flip) | 51 `*` margin |
|------|---------------|-----------------------------|---------------|
| True Chitra | 6.147e-4 | 6.941e-8 | 25.24 |
| True Citra | 6.147e-4 | 6.941e-8 | 25.24 |
| True Revati | 6.136e-4 | 5.530e-8 | 21.44 |
| True Pushya | 6.135e-4 | 5.644e-8 | 24.12 |
| True Mula | 6.409e-4 | 6.022e-8 | 25.98 |
| True Sheoran | 6.135e-4 | 5.569e-8 | 26.63 |
| Galactic Center | 6.248e-4 | 6.952e-8 | 28.25 |
| Galactic Center Rgilbrand | 6.248e-4 | 6.952e-8 | 23.87 |
| Galactic Center Cochrane | 6.248e-4 | 6.954e-8 | 358.2 |
| Galactic Equator IAU 1958 | 5.974e-4 | 1.121e-7 | 31.43 |
| Galactic Equator True | 5.973e-4 | 1.124e-7 | 31.48 |
| Galactic Equator Mula | 5.973e-4 | 1.127e-7 | 24.81 |
| Galactic Center Mardyks | 6.142e-4 | 5.924e-9 (does not kill) | 31.41 |
| Galactic Center Mula/Wilhelm | 9.742e-4 | 1.402e-5 | 21.50 |
| Galactic Equator Fiorenza | 6.142e-4 | 6.178e-9 (does not kill) | 26.40 |

Best killing margin per mutant (the largest over modes and epochs, since one
failing assertion kills):

| Mutant | Best margin | From | Weakest mode that still kills |
|--------|-------------|------|-------------------------------|
| 58:31 `+`→`-` | 9.742e-4 | Mula/Wilhelm | 5.973e-4 (Equator True / Mula) |
| 58:51 `+`→`-` | 1.402e-5 | Mula/Wilhelm | 5.530e-8 (True Revati); Mardyks and Fiorenza do not kill |
| 58:51 `+`→`*` | 358.2 | Cochrane | 21.44 (True Revati, T = 1) |
| 28:5 `delete -` (IAU 1958 `c3`) | 1.121e-7 | IAU 1958 | the only mode that carries it |
| 34:5 `delete -` (Equator True `c3`) | 1.124e-7 | Equator True | same |
| 40:5 `delete -` (Equator Mula `c3`) | 1.127e-7 | Equator Mula | same |
| 52:5 `delete -` (Mula/Wilhelm `c3`) | 1.402e-5 | Mula/Wilhelm | same |

**Binding constraint and floor.** The tolerance must sit above the worst fit
residual at the two epochs and below the smallest margin of a mutant that has
only one killer. The floor is Mula/Wilhelm's residual of 6.73e-9 at T = 1 (at
T = 0 the worst residual is Wilhelm's 3.6e-9). The ceiling is the three galactic
Equator `c3` `delete -` flips, each killable only through its own mode:
1.121e-7, 1.124e-7 and 1.127e-7. 2e-8 is 3x above the floor and 5.6x below the
ceiling. The true minimum over the mutants is therefore **1.121e-7** (28:5, IAU
1958). The 58:51 `+`→`-` mutant is killed through many modes (best 1.4e-5) and
every True-star and Center mode except Mardyks clears 5.5e-8, so that test is
not leaning on one fit.

Why SE rows with a 2e-8 degree (7.2e-5 arcsec) tolerance rather than the gate's
1.0 arcsec ceiling: the ceiling is a release criterion sized so the fits pass
with margin (measured worst case 0.057 arcsec), so it cannot see a `c3` sign
flip of about 1e-7 degrees. The committed fits reproduce the T = 0 and T = 1
rows to 6.7e-9 degrees, so the tighter bound is measured, not assumed, and the
corpus file stays unchanged. **Both JDs are nodes of the fit grid** (T = 0 and
the window edge T = 1); a refit with a different grid or window must re-measure
these residuals before the tolerance is trusted.

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
