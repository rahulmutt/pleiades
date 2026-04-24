# Time Scale, Delta T, Apparentness, and Observer Policy

Status: initial policy for the current pre-release backends.

Pleiades keeps time-scale conversion and observer semantics explicit so backend accuracy claims remain truthful while the production ephemeris implementations are still being expanded.

## Time scales and Delta T

- Backend position requests carry a typed [`TimeScale`](../crates/pleiades-types/src/lib.rs) on the requested instant.
- The current first-party ephemeris backends accept **TT** (Terrestrial Time) for position queries.
- The library does **not** currently convert UTC/UT1 to TT/TDB internally for backend position requests.
- Callers that start from civil time or UT are responsible for applying an appropriate Delta T policy before constructing a TT instant.
- Until a project-level Delta T model is implemented, validation fixtures and reports should state the time scale of each epoch explicitly and should not imply UTC-to-TT conversion support.

## Apparent versus mean coordinates

- Current first-party source/data backends report **mean geometric** coordinates only.
- Backends whose metadata has `capabilities.apparent = false` must reject `Apparentness::Apparent` requests instead of silently returning mean coordinates.
- Light-time, aberration, deflection, nutation, and related apparent-place corrections are planned production work and must be documented per backend when implemented.

## Observer and topocentric behavior

- Chart-level observer locations are currently used for house calculations.
- Body positions in chart assembly are queried geocentrically unless a future API adds an explicit topocentric position mode.
- Geocentric-only backends must reject direct backend requests that include an observer location with a structured `InvalidObserver` error.
- This separation prevents an observer used for houses from being mistaken for topocentric planetary or lunar coordinates.

## Frame behavior

- Ecliptic and equatorial result fields may both be populated by a backend, but the request frame still records the caller's requested output frame.
- Frame transforms must document the obliquity/precession/nutation model used. Current VSOP87 and ELP placeholder paths use a mean-obliquity transform and remain approximate.

## Follow-up work

Production accuracy work should add:

1. a documented UTC/UT/TT/TDB conversion strategy or an explicit statement that conversions remain caller-provided;
2. backend-specific apparent-place correction support or structured rejection tests;
3. validation reports that label every reference epoch with its time scale;
4. topocentric position support only behind an explicit request/configuration surface.
