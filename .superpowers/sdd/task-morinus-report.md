# Morinus House System Fix Report

## Root Cause

In `crates/pleiades-houses/src/systems/mod.rs` (line ~282), `HouseSystem::Morinus` was routed into the same `equatorial_projection_houses` function as `Meridian` and `Axial`. That function uses:

```rust
ecliptic_longitude_from_ra(ra, obliquity) // atan2(sin(ra), cos(ra)*cos(eps))
```

Swiss Ephemeris's Morinus system ('M') is a *distinct* system from Meridian/Axial ('X'). Routing Morinus through the Meridian formula produced errors of ~4.9° (17,532 arcseconds) — the systematic difference between the two projection formulas at ~23.4° obliquity.

## Morinus Convention Derived from SE Corpus

The SE Morinus system divides the celestial equator into twelve equal 30° arcs starting at **RA = RAMC + 90°** (the IC meridian direction projected onto the equator), then projects each arc endpoint onto the ecliptic using the **full spherical rotation from equatorial to ecliptic coordinates for dec=0**:

```
ecliptic longitude = atan2(sin(RA) * cos(eps), cos(RA))
```

This is algebraically equivalent to `atan2(sin(RA), cos(RA) / cos(eps))`, which is what the implementation uses.

**Key difference from Meridian/Axial:**
- Meridian formula: `atan2(sin(RA), cos(RA) * cos(eps))` — inverse ecliptic-to-equatorial projection
- Morinus formula: `atan2(sin(RA) * cos(eps), cos(RA))` — forward equatorial-to-ecliptic projection

The offset is **not** a magic constant — it is the principled equatorial coordinate rotation. The `+90°` starting offset places house 1 at the IC meridian direction on the equator (consistent with the SE 'M' system origin convention), confirmed identically across all 5 corpus fixtures.

Morinus is **latitude-independent** because only RAMC and obliquity enter the formula (no geographic latitude).

## Fix

**Split Morinus out of the shared arm** and add a dedicated `morinus_houses` function:

```rust
// In calculate_houses match arm:
HouseSystem::Meridian | HouseSystem::Axial => {
    equatorial_projection_houses(request.instant, &request.observer, obliquity).into()
}
HouseSystem::Morinus => {
    morinus_houses(request.instant, &request.observer, obliquity).into()
}
```

```rust
fn morinus_houses(instant: Instant, observer: &ObserverLocation, obliquity: Angle) -> [Longitude; 12] {
    let st = local_sidereal_time(instant, observer.longitude).degrees();
    let eps = obliquity.degrees().to_radians();
    let cos_eps = eps.cos();
    core::array::from_fn(|index| {
        let ra = (st + 90.0 + (index as f64) * 30.0).to_radians();
        Longitude::from_degrees(ra.sin().atan2(ra.cos() / cos_eps).to_degrees())
    })
}
```

## BEFORE / AFTER Residual Table (vs SE corpus)

| Fixture        | BEFORE max (arcsec) | AFTER max (arcsec) |
|----------------|--------------------:|-------------------:|
| c0_lat00       |            17,532.2 |              14.31 |
| c1_lat40       |            17,532.2 |              14.31 |
| c2_lat55       |            17,532.2 |              14.31 |
| c3_lat66       |            17,532.2 |              14.31 |
| c4_lat40_e2    |            17,545.8 |               4.52 |

All post-fix residuals are well within the 120 arcsecond tolerance. The ~14 arcsecond residual at J2000.0 fixtures is the known floor from mean vs. apparent sidereal time (same as all other correctly-implemented systems).

## Meridian and Axial Unchanged

Meridian and Axial continue to use `equatorial_projection_houses` unchanged. Their residuals against SE remain identical to pre-fix:

| Fixture        | Meridian max (arcsec) |
|----------------|----------------------:|
| c0_lat00       |                 14.31 |
| c1_lat40       |                 14.31 |
| c2_lat55       |                 14.31 |
| c3_lat66       |                 14.31 |
| c4_lat40_e2    |                  4.52 |

## Tests Added / Modified

1. **Renamed** `meridian_axial_and_morinus_share_the_documented_equatorial_projection_layout` → `meridian_and_axial_share_the_documented_equatorial_projection_layout` (removed Morinus from the shared-layout assertion).

2. **Added** `morinus_is_distinct_from_meridian_and_produces_12_cusps` — verifies Morinus produces 12 cusps and is not identical to Meridian at non-zero obliquity.

3. **Added** `morinus_cusps_match_swiss_ephemeris_corpus_within_120_arcsec` — SE-anchored test using c1_lat40 fixture, 120 arcsec tolerance (actual max ~14 arcsec).

4. **Added** `morinus_cusps_are_latitude_invariant` — asserts bit-identical cusps at lat 0°, 40°, 55°, 66° for the same JD/lon, confirming the latitude-independence property.

All 64 pleiades-houses tests pass. `cargo test -p pleiades-validate --no-run` builds cleanly. Zero warnings.
