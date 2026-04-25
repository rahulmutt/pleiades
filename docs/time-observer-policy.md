# Time Scale, Delta T, Apparentness, and Observer Policy

Status: initial policy for the current pre-release backends.

Pleiades keeps time-scale conversion and observer semantics explicit so backend accuracy claims remain truthful while the production ephemeris implementations are still being expanded.

## Time scales and Delta T

- Backend position requests carry a typed [`TimeScale`](../crates/pleiades-types/src/lib.rs) on the requested instant.
- The current first-party ephemeris backends accept **TT** (Terrestrial Time) and **TDB** (Barycentric Dynamical Time) for position queries.
- The library does **not** currently choose a UTC/UT1-to-TT/TDB model internally for backend position requests.
- Callers that start from civil time or UT are responsible for applying an appropriate Delta T, leap-second, DUT1, and/or relativistic policy before querying a backend that requires TT or TDB.
- `pleiades-types` provides mechanical caller-supplied offset helpers: `JulianDay::add_seconds`, `Instant::with_time_scale_offset`, `Instant::tt_from_ut1`, `Instant::tt_from_utc`, `Instant::tt_from_tdb`, `Instant::tdb_from_tt`, `Instant::tdb_from_tt_signed`, `Instant::tdb_from_ut1`, `Instant::tdb_from_ut1_signed`, `Instant::tdb_from_utc`, and `Instant::tdb_from_utc_signed`. These helpers make an already chosen conversion policy explicit (`target - source` seconds) but do not model Delta T, UTC leap seconds, or relativistic TDB terms themselves.
- `pleiades-core::ChartRequest` mirrors that policy with builder conveniences for applying a caller-supplied instant offset or converting UT1-tagged, UTC-tagged, TT-tagged, or TDB-tagged chart requests to TT/TDB before chart assembly. UTC-tagged requests can now be lifted directly to TDB through caller-supplied TT-UTC and TDB-TT offsets, and TDB-tagged requests can be converted back to TT with a caller-supplied signed offset. When a signed TDB-TT correction is needed, the `*_signed` TDB helpers preserve that policy explicitly instead of forcing the caller to pre-quantize the offset into a non-negative duration.
- `pleiades-cli chart` exposes matching `--utc` / `--ut1` tags plus explicit `--tt-offset-seconds` and `--tdb-offset-seconds` flags so the command-line chart workflow can exercise the same caller-supplied conversion policy without introducing built-in Delta T or relativistic modeling.
- `pleiades-backend` exposes shared request-shape helpers (`validate_request_policy` and `validate_observer_policy`) so backend crates can enforce the same time-scale, frame, apparentness, and observer rules without duplicating the policy checks.
- The default batch adapter preserves those same structured failures request-by-request, so a batch that includes an unsupported apparentness or observer shape still fails with the backend's explicit error kind rather than a generic batch wrapper error.
- Until a project-level Delta T model is implemented, validation fixtures and reports should state the time scale of each epoch explicitly and should not imply automatic UTC-to-TT or UTC-to-TDB conversion support.

## Apparent versus mean coordinates

- Current first-party source/data backends report **mean geometric** coordinates only.
- Backends whose metadata has `capabilities.apparent = false` must reject `Apparentness::Apparent` requests instead of silently returning mean coordinates.
- `pleiades-backend::EphemerisRequest::new` defaults to `Apparentness::Mean` so bare requests line up with the current mean-only first-party backends.
- Light-time, aberration, deflection, nutation, and related apparent-place corrections are planned production work and must be documented per backend when implemented.

## Observer and topocentric behavior

- Chart-level observer locations are currently used for house calculations.
- Body positions in chart assembly are queried geocentrically unless a future API adds an explicit topocentric position mode.
- Geocentric-only backends must reject direct backend requests that include an observer location with a structured `InvalidObserver` error.
- This separation prevents an observer used for houses from being mistaken for topocentric planetary or lunar coordinates.

## Frame behavior

- Ecliptic and equatorial result fields may both be populated by a backend, but the request frame still records the caller's requested output frame.
- The shared type layer exposes the mean-obliquity forward rotation as `EclipticCoordinates::to_equatorial` and the inverse rotation as `EquatorialCoordinates::to_ecliptic`, so callers and backend tests can round-trip frame conversions with the same documented obliquity policy.
- Frame transforms must document the obliquity/precession/nutation model used. Current VSOP87 and ELP placeholder paths use a mean-obliquity transform and remain approximate.

## Follow-up work

Production accuracy work should add:

1. a documented UTC/UT/TT/TDB conversion strategy if Pleiades adopts built-in Delta T/leap-second modeling beyond caller-supplied offsets;
2. backend-specific apparent-place correction support or structured rejection tests;
3. validation reports that label every reference epoch with its time scale;
4. topocentric position support only behind an explicit request/configuration surface.
