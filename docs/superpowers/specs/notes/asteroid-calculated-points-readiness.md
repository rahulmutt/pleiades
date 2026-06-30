# Downstream Readiness: Calculated Astrological Points

Calculated points (Black Moon Lilith / lunar apogee, Part of Fortune, Vertex, Arabic Parts, etc.)
are **out of scope** for this library and are deferred to a downstream astrology crate that
consumes the primitives exposed here.

## Primitive availability

| Primitive | Status | Location |
|---|---|---|
| **Lunar node variants** — `MeanNode`, `TrueNode` | EXISTS | `crates/pleiades-types/src/bodies.rs:69,71` (enum variants); listed as source-backed release bodies in `crates/pleiades-backend/src/release_body_claims.rs:9-10` |
| **Lunar apogee variants** — `MeanApogee` | EXISTS | `crates/pleiades-types/src/bodies.rs:73`; source-backed release body in `crates/pleiades-backend/src/release_body_claims.rs:11` |
| **Lunar apogee variants** — `TrueApogee` | EXISTS | Release-grade via `PackagedDataBackend` osculating path (`crates/pleiades-data/src/backend.rs`) + `crates/pleiades-apsides` (derived from packaged Moon state). Gated against Swiss Ephemeris `SE_OSCU_APOG` by `validate-lilith` (3177 samples, 1900–2100). Of-date frame: precession + nutation-in-longitude only. |
| **Lunar perigee variants** — `MeanPerigee` | EXISTS | `crates/pleiades-types/src/bodies.rs:77`; release body in `PackagedDataBackend` and constrained in `ElpBackend` |
| **Lunar perigee variants** — `TruePerigee` | EXISTS | Release-grade via `PackagedDataBackend` osculating path (`crates/pleiades-data/src/backend.rs`) + `crates/pleiades-apsides`. Perigee = opposite apse to `TrueApogee`. Gated by `validate-lilith`. |
| **Body ecliptic positions** (Sun, Moon, planets, asteroids) | EXISTS | Core backend; Sun–Neptune are release-grade major bodies, Moon / selected asteroids are source-backed bodies per `crates/pleiades-backend/src/release_body_claims.rs:16-43` |
| **Chart angles** — Ascendant, Descendant, Midheaven, Imum Coeli | EXISTS | `HouseAngles` struct with all four fields exposed at `crates/pleiades-houses/src/systems/mod.rs:85-105`; stored in `HouseSnapshot.angles` (`crates/pleiades-houses/src/systems/mod.rs:110-125`); propagated through `ChartSnapshot.houses` (`crates/pleiades-core/src/chart/snapshot.rs:75`) and applied sidereal correction in `crates/pleiades-core/src/chart/mod.rs:175-194` |
| **House cusps** | EXISTS | `HouseSnapshot.cusps: Vec<Longitude>` at `crates/pleiades-houses/src/systems/mod.rs:124`; 12 cusps (most systems) or 36 (Gauquelin sectors) |

## Conclusion

The downstream crate has everything it needs: `MeanApogee` (Black Moon Lilith mean), `MeanNode` /
`TrueNode`, all body ecliptic positions, chart angles (Asc / MC), and house cusps are all exposed.
`TrueApogee` and `TruePerigee` (osculating True Lilith) are now release-grade via `PackagedDataBackend`
(`crates/pleiades-data` osculating path + `crates/pleiades-apsides`), gated against Swiss Ephemeris
`SE_OSCU_APOG` by `validate-lilith`. There are no remaining genuine gaps for the lunar-point primitive set
in the ecliptic frame. The next queued sub-project is equatorial/declination output for `TrueApogee` and
`TruePerigee`.
