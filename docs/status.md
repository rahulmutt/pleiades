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
| Planetary stations (geocentric apparent or mean of date; tropical or sidereal) | [`pleiades-events`](../crates/pleiades-events) | `validate-stations` | arcsecond-class longitude; timing within 22 minutes of Swiss Ephemeris (planets; true node existence-checked only); Ceres–Vesta, the osculating apogee (663.1 s, 1.873″) and the 17 stationing fictitious bodies (94.3 s, 1.565″) station for station against SWIEPH |
| Exact aspects between two bodies (geocentric apparent or mean of date, heliocentric; tropical or sidereal) | [`pleiades-events`](../crates/pleiades-events) | `validate-aspects`, `validate-aspects-lunar` | event for event with Swiss Ephemeris (planets, plus Sun–Ceres..Vesta, Moon–Ceres, Mars–Vesta and Jupiter–Ceres, and the 19 fictitious bodies in the mean-of-date frame only, in `validate-aspects` (also `validate-aspects-fictitious`), and the lunar points in `validate-aspects-lunar` (its own nightly job `aspects-lunar-gate`): TrueNode with seven planets, TrueApogee with four, MeanNode with seven and MeanApogee with four, against SWIEPH; inside a graze window events may exist in one ephemeris only and are compared by count (by parity alone when the turn misses the level by under half the margin), with the engine's side of the level checked at the window edges); outside graze windows, separation within a 4.4″ ceiling at the exact moment (3.113″ measured maximum, TrueApogee–Saturn; the true node and mean points within 2.3″); events inside graze windows are reported, not bounded |
| Rise/set/transit & horizontal | [`pleiades-events`](../crates/pleiades-events) | `validate-rise-trans` | sub-arcsecond (horizontal); timing seconds-of-time |
| Fictitious bodies | [`pleiades-fict`](../crates/pleiades-fict) | `validate-fictitious` | definitional (sub-arcsecond) |
| Nodes & apsides | [`pleiades-events`](../crates/pleiades-events) | `validate-nod-aps` | sub-arcsecond (mean) / arcminute-class (osculating; Ceres–Vesta within 38″ against SWIEPH) |
| Phase & magnitude | [`pleiades-events`](../crates/pleiades-events) | `validate-pheno` | arcsecond-class |
| Lunar occultations | [`pleiades-events`](../crates/pleiades-events) | `validate-occultations` | timing seconds-of-time; position arcminute-class |
| True (osculating) Lilith | [`pleiades-apsides`](../crates/pleiades-apsides) | `validate-lilith` | arcminute-class |
| True (osculating) Node | [`pleiades-data`](../crates/pleiades-data), [`pleiades-elp`](../crates/pleiades-elp) | `validate-true-node` | arcsecond-class (cross-theory, packaged) / arcminute-class (ELP, ≤1.3′) |
| Mean lunar node & apsides (Mean Lilith) | [`pleiades-data`](../crates/pleiades-data) | `validate-mean-lunar-points` | sub-arcsecond |

Crate names link to their docs.rs API docs where published, otherwise to the
crate source in this repo; gate names are the runnable `validate-*` subcommands
(and `release-gate`) that guard each surface.

**Asteroid stations (issue #167 (d)).** Ceres, Pallas, Juno and Vesta are compared station for station against `stations-corpus/asteroids.csv`, which `tools/se-stations-reference --asteroids` generates from Swiss Ephemeris SWIEPH with the SHA-256-pinned `seas_18`/`sepl_18`/`semo_18` files. Swiss Ephemeris's asteroid positions agree with JPL's sb441-n373s within 2.26″ (`docs/superpowers/specs/notes/2026-10-10-se-asteroid-agreement.md`). These series are compared in the full gate only (`mise run gate-stations`, the nightly `stations-gate` job, about 30 s on the 4-core runner with the planets, before the apogee and fictitious passes below); `release-smoke` checks the file against its manifest (checksum and row count, no scan; issue #254). asteroid:433-Eros is not gated for events: Swiss Ephemeris keeps it in a separate per-asteroid file that is not pinned. Its positions are gated against sb441.

**Apogee and fictitious-body stations (issue #167 (d)).** After the planets and asteroids, `validate-stations` (`mise run gate-stations`) compares `TrueApogee` (the osculating apogee, geocentric apparent, 1900-2100) against `stations-corpus/true-apogee.csv` (SWIEPH, 4776 stations; the tool drops 5 SE artefact pairs; 3 speed-graze windows, one strict; 4774 stations compared station for station outside the windows; measured maximum 663.1 s and 1.873″, ceiling 930 s / 2.7″), and the 17 stationing fictitious bodies (SE 40-58 from `seorbel.txt`; WhiteMoon and Waldemath never station; geocentric mean of date, 1900-2100) against `stations-corpus/fictitious.csv` (13845 stations, no windows; maximum 94.4 s and 0.307″ for Vulcan, every other body at most 15.3 s and 0.306″; Nibiru was at 40.6 s and 1.565″ before its elements were precessed with Vondrák 2011 as in SE). These two corpora take a station as a sign change of the central difference of SE's longitudes, not of its reported speed: SE's osculating-apogee speed jumps at isolated instants (+0.081 degrees/day between samples of -0.898 at JD 2424889.42, with a 0.7″ longitude step) and has step discontinuities, and SE's Proserpina speed is inconsistent with its own positions (its speed zeros lie about 4.4 h from the zeros of its longitude's derivative). The window margins are 3 times the measured engine-minus-SE speed difference at SE's extrema, rounded up to one significant figure: 8e-4 degrees/day for the apogee (measured 2.39e-4) and 3e-7 for the fictitious bodies (measured 7.39e-8). The tight fictitious margin is an intended tripwire: an engine change to fictitious-body speeds fails the margin test (a nightly `test-full` test, `margins_cover_the_engine_speed_difference_at_every_extremum`) and requires regenerating the windows with the reference tool. The manifest and windows checks are in the blocking `release-smoke`. With all four passes the "Run full stations gate" step took 2m14s and the `stations-gate` job 2m55s on the nightly runner (one run, 39101104860, 2026-10-18), against 28–29 s and 1m06s–1m27s before the apogee and fictitious passes.

**Pluto's station offset (issue #167 (f)).** The planet corpus's -56.6 s Pluto mean comes from SE's reported apparent speed, which omits part of the rate of nutation (about 2.5e-7 degrees/day; with SEFLG_NONUT the gap falls to about +-3 s). Against the zeros of the derivative of SE's own longitudes the engine is at -6 s (SWIEPH), within the 198 s rms scatter (measured 2026-10-18).

**Asteroid aspects (issue #168 (f)).** The exact aspects (0, 60, 90, 120 and 180 degrees) of Sun–Ceres, Sun–Pallas, Sun–Juno, Sun–Vesta, Mars–Vesta and Jupiter–Ceres over 1900–2100, and of Moon–Ceres over 1990–2030, are compared event for event (10118 events) against `aspects-corpus/asteroids.csv`, which `tools/se-aspects-reference --asteroids` generates from Swiss Ephemeris SWIEPH with the SHA-256-pinned `seas_18`/`sepl_18`/`semo_18` files. Swiss Ephemeris's asteroid positions agree with JPL's sb441-n373s within 2.26″ (`docs/superpowers/specs/notes/2026-10-10-se-asteroid-agreement.md`). These series are compared in the full gate only (`mise run gate-aspects`, the nightly `aspects-gate` job); `release-smoke` checks the file against its manifest (checksum and row count, no scan; issue #254) and compares only the planet mean subset. Mars–Vesta and Jupiter–Ceres (issue #253) are the pairs whose separation turns, so they carry the asteroids' retrograde loops into the gate; a Sun or Moon pair's separation never turns. The asteroid pass took 54.5 s in release on the 24-core dev box under load (2026-10-10), and roughly 30 s of the 77–151 s gate command on the 4-core nightly runner (2026-10-11..15). asteroid:433-Eros is not gated for events: Swiss Ephemeris keeps it in a separate per-asteroid file that is not pinned. Its positions are gated against sb441.

**Fictitious-body aspects (issue #168).** The exact aspects of the 19 fictitious bodies (SE 40–58) with the Sun, and of Cupido, Hades, Zeus, Kronos, Apollon, Admetos, Vulkanus, Poseidon and Transpluto with Saturn, over 1900–2100, are compared event for event (28 pairs, 38733 events, no graze windows; the closest turn is 43.3″, Admetos–Saturn) against `aspects-corpus/fictitious.csv` in the geometric mean-of-date frame, from SWIEPH with the pinned `seorbel.txt`. The separation is within 0.048″ (Vulcan–Sun) and the longitude within 0.342″ (Waldemath–Sun); all ceilings are 2.3″. Nibiru–Sun was at 1.284″ and 1.578″, a steady +1.26″ offset, until the element equinoxes moved from IAU-1976 to the Vondrák 2011 precession SE uses (2026-10-18). The longitude residual of about 0.31″ common to every pair is the IAU-1976 J2000-to-date precession of the mean-of-date frame against SE's Vondrák at 1900 and 2100; it cancels in the separation. The apparent frame is not gated for fictitious bodies: Swiss Ephemeris's light-time correction for them jumps by up to about 5″ at isolated instants (about 0.16% of samples on a 5-day grid) while the engine's series is smooth, and the jumps vanish with `SEFLG_TRUEPOS`. The pass runs inside `validate-aspects` / `mise run gate-aspects` (also alone as `validate-aspects-fictitious` / `mise run gate-aspects-fictitious`) and took 32.3 s locally in release (load 11–12); the `aspects-gate` step took 1m35s on the nightly runner against 53 s without it (runs 39052564875 and 39041130704, 2026-10-17). The mean and osculating perigees remain ungated for aspects.

**Asteroid nodes and apsides (issue #160).** The osculating (`SE_NODBIT_OSCU`: heliocentric orbit, geocentric output) ascending and descending nodes, perihelion and aphelion of Ceres, Pallas, Juno and Vesta at eight epochs over 1900–2100 are compared (32 rows) against `nod-aps-corpus/asteroids.csv`, which `tools/se-nodaps-reference --asteroids` generates from Swiss Ephemeris SWIEPH with the SHA-256-pinned `seas_18`/`sepl_18`/`semo_18` files. They form the `OSCU_ASTEROID` category of `validate-nod-aps`, so they run wherever that gate does, including `release-smoke`. Measured maxima (2026-10-10): longitude 27.0″ (Ceres perihelion; nodes within 8.3″), latitude 1.39″, distance 5.8e-5 relative, longitude speed 2.2e-4 °/day; ceilings are measured × 1.4, angles never below 2.3″. Fictitious bodies stay unreferenced: Swiss Ephemeris's `swe_nod_aps` does not implement them. asteroid:433-Eros is not gated for nodes and apsides, for the same reason as its events.

### Known limits

- Body/backend grades are **per-backend**: Pluto and the Moon are release-grade
  via the packaged artifact; VSOP87 Pluto and the compact ELP Moon stay
  constrained. See `packaged_body_claims` in
  [`crates/pleiades-data/src/lib.rs`](../crates/pleiades-data/src/lib.rs).
- **Dense packaged asteroids.** `PackagedDataBackend` serves Ceres, Pallas,
  Juno, Vesta and `asteroid:433-Eros` on every date in 1900-2100 from
  heliocentric fits to the JPL `sb441-n373s` kernel. They are gated against the
  407 `sb441-n373s` rows per body of `asteroid_reference.csv` by
  [`packaged_asteroids_match_the_sb441_rows`](../crates/pleiades-data/src/tests/asteroid_gate.rs)
  (row-gate ceiling `ASTEROID_CORPUS_CEILING`: 0.42″ longitude x cos latitude,
  0.38″ latitude, against those rows only). Row maxima: 0.2938″ longitude
  (Juno) and 0.2646″ latitude (Ceres). Dense 0.5-day sampling against the
  kernel over 1900-2100 finds at most 0.52″ longitude and 0.37″ latitude
  (acceptance rule 1″). Caller-supplied artifacts generated before
  compatibility profile 0.7.34 must be regenerated (two kernels), because their
  asteroid:433-Eros segments are the old sparse snapshot fit. `asteroid:99942-Apophis` stays snapshot-only: it is served only at
  the sample rows of the sparse JPL Horizons fixture (a cluster from 2001-01-05
  to 2001-01-10 and isolated rows in 1800, 2000, 2003, 2132 and 2500) and any
  other date returns an out-of-range
  error. Asteroid stations, aspects and osculating nodes/apsides are gated (see above). Other asteroids need `pleiades_jpl::SpkBackend` with a JPL kernel
  (`docs/spk-kernel-sourcing.md`).
- Apparent place omits gravitational light-deflection. Rise/set/transit and
  horizontal coordinates read the `TimeScale` tag on their query instants and
  return **TDB** instants; their accuracy in civil time is bounded by the
  packaged ΔT model (observed through 2020, leap-second-bound to within 0.9 s
  through the leap table's horizon, extrapolated beyond) — see
  [docs/time-observer-policy.md](time-observer-policy.md).
- Several surfaces carry documented, non-gated bounds (occultation planet-total
  obscuration and `central` flag; osculating nodes/apsides
  of fictitious bodies and of asteroids other than Ceres–Vesta). Each is recorded in its crate's rustdoc and in
  `crates/pleiades-core/src/compatibility/mod.rs`.
- Ingestion and kernel/corpus parsing are treated as untrusted input — see
  [docs/threat-model.md](threat-model.md).
- Lunar theory selection and its limits: [docs/lunar-theory-policy.md](lunar-theory-policy.md).
