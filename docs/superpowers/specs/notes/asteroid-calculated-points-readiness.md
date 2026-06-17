# Downstream Readiness: Calculated Astrological Points

Calculated points (Black Moon Lilith / lunar apogee, Part of Fortune, Vertex, Arabic Parts, etc.)
are **out of scope** for this library and are deferred to a downstream astrology crate that
consumes the primitives exposed here.

## Primitive availability

| Primitive | Status | Location |
|---|---|---|
| **Lunar node variants** — `MeanNode`, `TrueNode` | EXISTS | `crates/pleiades-types/src/bodies.rs:69,71` (enum variants); listed as source-backed release bodies in `crates/pleiades-backend/src/release_body_claims.rs:9-10` |
| **Lunar apogee variants** — `MeanApogee` | EXISTS | `crates/pleiades-types/src/bodies.rs:73`; source-backed release body in `crates/pleiades-backend/src/release_body_claims.rs:11` |
| **Lunar apogee variants** — `TrueApogee` | GAP (unsupported) | Variant exists in enum (`crates/pleiades-types/src/bodies.rs:75`) but is **explicitly unsupported** per release-claims policy: "True Apogee and True Perigee remain unsupported" (`crates/pleiades-backend/src/release_body_claims.rs:64`) |
| **Lunar perigee variants** — `MeanPerigee` | EXISTS | `crates/pleiades-types/src/bodies.rs:77`; source-backed release body in `crates/pleiades-backend/src/release_body_claims.rs:12` |
| **Lunar perigee variants** — `TruePerigee` | GAP (unsupported) | Enum variant present (`crates/pleiades-types/src/bodies.rs:79`) but explicitly unsupported alongside `TrueApogee` |
| **Body ecliptic positions** (Sun, Moon, planets, asteroids) | EXISTS | Core backend; Sun–Neptune are release-grade major bodies, Moon / selected asteroids are source-backed bodies per `crates/pleiades-backend/src/release_body_claims.rs:16-43` |
| **Chart angles** — Ascendant, Descendant, Midheaven, Imum Coeli | EXISTS | `HouseAngles` struct with all four fields exposed at `crates/pleiades-houses/src/systems/mod.rs:85-105`; stored in `HouseSnapshot.angles` (`crates/pleiades-houses/src/systems/mod.rs:110-125`); propagated through `ChartSnapshot.houses` (`crates/pleiades-core/src/chart/snapshot.rs:75`) and applied sidereal correction in `crates/pleiades-core/src/chart/mod.rs:175-194` |
| **House cusps** | EXISTS | `HouseSnapshot.cusps: Vec<Longitude>` at `crates/pleiades-houses/src/systems/mod.rs:124`; 12 cusps (most systems) or 36 (Gauquelin sectors) |

## Conclusion

The downstream crate has what it needs: `MeanApogee` (Black Moon Lilith mean), `MeanNode` /
`TrueNode`, all body ecliptic positions, and chart angles (Asc / MC) plus house cusps are all
exposed. The only genuine gap is that `TrueApogee` and `TruePerigee` are enum-representable but
explicitly unsupported by the backend — a downstream crate needing the oscillating true lunar
apogee must note this limitation or compute it from raw Moon orbital elements itself.
