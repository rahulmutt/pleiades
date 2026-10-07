# Status and known limits

`pleiades` is a release-hardening foundation, not a finished end-user
ephemeris. Each surface below is guarded by a fail-closed numeric gate; measured
residuals, carve-outs, and caveats live in the linked crate docs and in the
[`pleiades-core` compatibility registry](../crates/pleiades-core/src/compatibility/mod.rs),
not restated here.

| Surface | Crate | Gate | Accuracy class |
| --- | --- | --- | --- |
| Body positions / packaged artifact | [`pleiades-data`](../crates/pleiades-data) | `validate-corpus` | sub-arcsecond (majors) |
| House systems | [`pleiades-houses`](https://docs.rs/pleiades-houses) | `validate-houses` | sub-arcsecond |
| Ayanamsas | [`pleiades-ayanamsa`](https://docs.rs/pleiades-ayanamsa) | `validate-ayanamsa` | sub-arcsecond |
| Sidereal time & chart angles | [`pleiades-houses`](https://docs.rs/pleiades-houses) | `validate-angles` | sub-arcsecond |
| Apparent place (of-date ecliptic) | [`pleiades-core`](https://docs.rs/pleiades-core) | `validate-apparent` | sub-arcsecond |
| Apparent equatorial (RA/Dec) | [`pleiades-core`](https://docs.rs/pleiades-core) | `validate-equatorial` | sub-arcsecond |
| Sidereal chart (mean equinox of date, less the ayanamsa; mean, and apparent with the apparent-star ayanamsa) | [`pleiades-core`](https://docs.rs/pleiades-core) | `validate-sidereal-position` | arcsecond-class against Swiss Ephemeris |
| Apparent-star ayanamsa (issue #164) | [`pleiades-core`](https://docs.rs/pleiades-core) | `validate-ayanamsa-apparent` | 0.011″ against Swiss Ephemeris; δ Cnc near the Sun 0.49″ |
| Civil time conversion | [`pleiades-time`](https://docs.rs/pleiades-time) | (unit/property) | leap-second-exact |
| Topocentric correction | [`pleiades-core`](https://docs.rs/pleiades-core) | `validate-topocentric` | opt-in correction |
| Backend frame consistency (J2000) | [`pleiades-core`](https://docs.rs/pleiades-core) | `release-gate` | invariant gate |
| Eclipses (global) | [`pleiades-eclipse`](../crates/pleiades-eclipse) | `validate-eclipses` | arcsecond-class; timing seconds-of-time |
| Eclipses (local circumstances) | [`pleiades-eclipse`](../crates/pleiades-eclipse) | `validate-eclipses-local` | arcsecond-class; timing seconds-of-time |
| Longitude crossings (geocentric apparent or mean of date, heliocentric; tropical or sidereal) | [`pleiades-events`](../crates/pleiades-events) | `validate-crossings` | arcsecond-class |
| Ecliptic position & speed (geocentric apparent, geocentric mean-of-date, heliocentric; tropical or sidereal zodiac) | [`pleiades-events`](../crates/pleiades-events) | `validate-helio-position` (heliocentric), `validate-apparent` (geocentric); mean-of-date and sidereal longitudes through the `validate-crossings` gate (their speeds are unit-tested only) | arcsecond-class geocentric; arcsecond-class heliocentric |
| Planetary stations (geocentric apparent or mean of date; tropical or sidereal) | [`pleiades-events`](../crates/pleiades-events) | `validate-stations` | arcsecond-class longitude; timing within 22 minutes of Swiss Ephemeris (planets; true node existence-checked only) |
| Exact aspects between two bodies (geocentric apparent or mean of date, heliocentric; tropical or sidereal) | [`pleiades-events`](../crates/pleiades-events) | `validate-aspects` | event for event with Swiss Ephemeris; separation within 3″ at the exact moment |
| Rise/set/transit & horizontal | [`pleiades-events`](../crates/pleiades-events) | `validate-rise-trans` | sub-arcsecond (horizontal); timing seconds-of-time |
| Fictitious bodies | [`pleiades-fict`](../crates/pleiades-fict) | `validate-fictitious` | definitional (sub-arcsecond) |
| Nodes & apsides | [`pleiades-events`](../crates/pleiades-events) | `validate-nod-aps` | sub-arcsecond (mean) / arcminute-class (osculating) |
| Phase & magnitude | [`pleiades-events`](../crates/pleiades-events) | `validate-pheno` | arcsecond-class |
| Lunar occultations | [`pleiades-events`](../crates/pleiades-events) | `validate-occultations` | timing seconds-of-time; position arcminute-class |
| True (osculating) Lilith | [`pleiades-apsides`](../crates/pleiades-apsides) | `validate-lilith` | arcminute-class |
| True (osculating) Node | [`pleiades-data`](../crates/pleiades-data), [`pleiades-elp`](../crates/pleiades-elp) | `validate-true-node` | arcsecond-class (cross-theory, packaged) / arcminute-class (ELP, ≤1.3′) |
| Mean lunar node & apsides (Mean Lilith) | [`pleiades-data`](../crates/pleiades-data) | `validate-mean-lunar-points` | sub-arcsecond |

Crate names link to their docs.rs API docs where published, otherwise to the
crate source in this repo; gate names are the runnable `validate-*` subcommands
(and `release-gate`) that guard each surface.

### Known limits

- Body/backend grades are **per-backend**: Pluto and the Moon are release-grade
  via the packaged artifact; VSOP87 Pluto and the compact ELP Moon stay
  constrained. See `packaged_body_claims` in
  [`crates/pleiades-data/src/lib.rs`](../crates/pleiades-data/src/lib.rs).
- **Asteroids offline are served only at their sample rows.** Ceres, Pallas,
  Juno, Vesta, `asteroid:433-Eros` and `asteroid:99942-Apophis` come from a
  sparse JPL Horizons fixture: a handful of epochs and a nine-day cluster in
  January 2001 (Apophis from 2001-01-06). Any other date returns an
  out-of-range error. The packaged artifact still carries an Eros fit that the
  backend does not serve. For asteroid positions across 1900–2100, use `pleiades_jpl::SpkBackend` with a
  JPL kernel (`docs/spk-kernel-sourcing.md`); offline coverage is tracked in
  issue #201.
- Apparent place omits gravitational light-deflection. Rise/set/transit and
  horizontal coordinates read the `TimeScale` tag on their query instants and
  return **TDB** instants; their accuracy in civil time is bounded by the
  packaged ΔT model (observed through 2020, leap-second-bound to within 0.9 s
  through the leap table's horizon, extrapolated beyond) — see
  [docs/time-observer-policy.md](time-observer-policy.md).
- Several surfaces carry documented, non-gated bounds (occultation planet-total
  obscuration and `central` flag; fictitious Nibiru; osculating small-body
  nodes/apsides). Each is recorded in its crate's rustdoc and in
  `crates/pleiades-core/src/compatibility/mod.rs`.
- Ingestion and kernel/corpus parsing are treated as untrusted input — see
  [docs/threat-model.md](threat-model.md).
- Lunar theory selection and its limits: [docs/lunar-theory-policy.md](lunar-theory-policy.md).
