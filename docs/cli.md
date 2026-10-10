# Command-line tools

The workspace has two unpublished contributor tools: `pleiades-cli`, for
inspection and chart reports, and `pleiades-validate`, for audits, validation
gates and release rehearsal. Neither is on crates.io; run them from a checkout.

## `pleiades-cli`

Run contributor commands through Cargo:

```bash
cargo run -q -p pleiades-cli -- help
cargo run -q -p pleiades-validate -- help
```

Useful inspection commands:

```bash
# One-screen release posture
cargo run -q -p pleiades-cli -- release-summary

# Current compatibility catalog/profile
cargo run -q -p pleiades-cli -- profile-summary

# Backend capability matrix
cargo run -q -p pleiades-cli -- backend-matrix-summary

# Request semantics and time/observer policy
cargo run -q -p pleiades-cli -- request-surface-summary
cargo run -q -p pleiades-cli -- utc-convenience-policy-summary

# Packaged artifact posture
cargo run -q -p pleiades-cli -- artifact-summary
```

Regenerate the packaged artifact over a custom coverage window (maintainers; needs
both kernels, which are not committed). `--asteroid-kernel` names the `sb441-n373s`
small-body kernel that supplies Ceres, Pallas, Juno, Vesta and `asteroid:433-Eros`;
the command refuses to run without it. `--start`/`--end` take a calendar year or a
Julian Day with a decimal point and default to 1900-2100:

```bash
cargo run --release -q -p pleiades-cli -- generate-artifact de440.bsp \
  --asteroid-kernel sb441-n373s.bsp --out artifact.bin --start 1900 --end 2100
```

Render a basic chart report:

```bash
cargo run -q -p pleiades-cli -- chart \
  --jd 2451545.0 \
  --body Sun \
  --body Moon
```

Render a sidereal chart with houses for an observer:

```bash
cargo run -q -p pleiades-cli -- chart \
  --jd 2451545.0 \
  --lat 51.5074 \
  --lon 0.0 \
  --ayanamsa Lahiri \
  --house-system "Whole Sign" \
  --body Sun \
  --body Moon \
  --mean
```

Notes:

- `--mean` reports the geometric place: on the J2000 equinox in the tropical zodiac, and on the mean equinox of date less the ayanamsa when `--ayanamsa` is given.
- `chart` defaults to `JD 2451545.0` if `--jd` is omitted.
- If no `--body` flags are given, the CLI uses the default chart body set from `pleiades-core`.
- `--body` accepts built-in labels such as `Sun`, `Moon`, and `Ceres`, plus custom identifiers such as `asteroid:433-Eros`. Ceres, Pallas, Juno, Vesta and `asteroid:433-Eros` are served offline on every date in 1900-2100 by the packaged artifact (dense `sb441-n373s` fits); `asteroid:99942-Apophis` is served only at the sample epochs of a sparse JPL fixture (a cluster from 2001-01-06 to 2001-01-10), and any other date for it returns an out-of-range error that names `SpkBackend`, the kernel-backed way to compute other asteroids. `stations` and `aspects` search the five packaged asteroids. Ceres, Pallas, Juno and Vesta are gated event for event against Swiss Ephemeris SWIEPH (issues #167 (d), #168 (f)): stations are gated, and aspects are gated only for the geocentric Sun-asteroid pairs and Moon-Ceres (other partners, frames and sidereal modes are searched but not gated); `asteroid:433-Eros` is searched but not event-gated (its positions are gated against sb441); for Apophis they return the out-of-range error, which `aspects` reports wrapped in an apparent-place failure.
- `--star-place mean|apparent` (with `--ayanamsa`) reads a star-anchored ayanamsa (True Citra/True Chitra, True Revati, True Pushya, True Mula, True Sheoran, and the Galactic Center modes other than Mardyks) from its anchor star's mean place (default) or apparent place, as Swiss Ephemeris's default sidereal convention does; Galactic Center (Mardyks), the galactic-equator modes and every other ayanamsa are unaffected, and the `stations`/`aspects` header names the apparent star place only for an anchored ayanamsa; the two differ by up to about 22″. Example: `--ayanamsa "True Citra" --star-place apparent`. A mean chart (`--mean`), a mean-fallback placement and the `stations`/`aspects` mean-of-date and heliocentric frames keep the mean ayanamsa, so `--star-place apparent` is rejected with `--mean` and with `--frame mean|helio`.
- `--ayanamsa` accepts built-in names such as `Lahiri` and custom definitions such as `custom:True Balarama|2451545.0|12.5`.
- Built-in civil-time conversion: use `--civil <YYYY-MM-DDTHH:MM:SS> [--civil-scale utc|ut1] [--civil-target tt|tdb]` to convert a calendar datetime to TT/TDB automatically (1900–2100, tiered quality). Alternatively, supply caller-chosen offsets via the `--tt-*` or `--tdb-*` flags. See [docs/time-observer-policy.md](time-observer-policy.md).

### Stations and aspects

`stations` lists the instants bodies turn retrograde or direct, and `aspects` the instants pairs of bodies reach exact separations. Both search a range, or one event either side of an instant:

```bash
# Mercury's and Jupiter's stations in the first half of 2000
cargo run -q -p pleiades-cli -- stations \
  --body Mercury --body Jupiter \
  --from 2000-01-01T00:00:00 --to 2000-07-01T00:00:00

# The last full Moon before J2000
cargo run -q -p pleiades-cli -- aspects \
  --pair Sun,Moon --angle 180 --previous --at 2451545.0

# Conjunctions and oppositions of two pairs over twenty days
cargo run -q -p pleiades-cli -- aspects \
  --pair Jupiter,Saturn --pair Sun,Moon --angle 0 --angle 180 \
  --from 2451680.0 --to 2451700.0
```

```text
Stations (geocentric apparent; Tropical zodiac)
2000-02-21T12:46:24.374 UTC  JD 2451596.03297 TDB  Mercury turns retrograde at 347.1784°
2000-03-14T20:39:19.093 UTC  JD 2451618.36138 TDB  Mercury turns direct at 332.7788°
2000-06-23T08:31:54.148 UTC  JD 2451718.85623 TDB  Mercury turns retrograde at 109.9618°
```

- An instant (`--from`, `--to`, `--at`) is a TDB Julian day, or a civil `YYYY-MM-DDTHH:MM:SS` datetime read as UTC from 1972 on and as UT1 before.
- `--from`/`--to` list every event in the range. `--next` or `--previous` with `--at` give one event per body (`stations`) or per pair and angle (`aspects`).
- A `--next` or `--previous` search that reaches the end of the 1900–2100 window first prints a note for that body (or pair and angle), such as `Mercury: none before the window's end (2100-01-01)`, and the others still print. An `--at` outside the window is an error, not a note. A pair that never reaches an angle (the Sun and Mercury at 60°) gets that note.
- Events from every `--body`, or every `--pair` and `--angle`, are merged in time order. Each line gives the civil time (UTC from 1972, UT1 before), the TDB Julian day, and the event; an aspect line ends with the two longitudes.
- `--frame geo|mean|helio` picks the apparent geocentric (default), mean geocentric of date, or heliocentric frame. `--ayanamsa <name>` reads longitudes in a sidereal zodiac, which moves a slow planet's station by minutes to hours. `--star-place mean|apparent` applies as for `chart`; `--star-place apparent` needs the apparent geocentric frame and is rejected with `--frame mean` or `--frame helio`.
- An aspect angle is a separation from 0 to 180 degrees and is found on both sides: `--angle 90` returns both squares.
- The searches cover 1900–2100 and use the same backend chain as `chart`. Accuracy and limits are those of the `pleiades-events` finders; see that crate's README.

## `pleiades-validate`

`pleiades-validate` is the maintainer tool for audits, reports, artifact inspection, and release rehearsal:

```bash
# Native dependency / build-hook audit
cargo run -q -p pleiades-validate -- workspace-audit

# Compatibility profile verification
cargo run -q -p pleiades-validate -- verify-compatibility-profile

# Full packaged-artifact inspection
cargo run -q -p pleiades-validate -- validate-artifact

# Compact validation report
cargo run -q -p pleiades-validate -- report-summary --rounds 100

# Stage and verify a release bundle
cargo run -q -p pleiades-validate -- bundle-release --out /tmp/pleiades-release
cargo run -q -p pleiades-validate -- verify-release-bundle --out /tmp/pleiades-release
```

For release reproducibility details, see [docs/release-reproducibility.md](release-reproducibility.md).
