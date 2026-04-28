# Time Scale, Delta T, Apparentness, and Observer Policy

Status: initial policy for the current pre-release backends.

Pleiades keeps time-scale conversion and observer semantics explicit so backend accuracy claims remain truthful while the production ephemeris implementations are still being expanded.

## Time scales and Delta T

- Backend position requests carry a typed [`TimeScale`](../crates/pleiades-types/src/lib.rs) on the requested instant.
- The current first-party ephemeris backends accept **TT** (Terrestrial Time) and **TDB** (Barycentric Dynamical Time) for position queries.
- The library does **not** currently choose a UTC/UT1-to-TT/TDB model internally for backend position requests.
- Callers that start from civil time or UT are responsible for applying an appropriate Delta T, leap-second, DUT1, and/or relativistic policy before querying a backend that requires TT or TDB.

## Primary API entry points

The current policy is surfaced explicitly at the main request layers rather than hidden behind a global conversion model:

- [`pleiades-types::Instant`](../crates/pleiades-types/src/lib.rs): carries the tagged instant and exposes caller-supplied retagging helpers such as `with_time_scale_offset`, `with_time_scale_conversion`, `tt_from_ut1`, `tt_from_utc`, `tt_from_tdb`, `tdb_from_tt`, `tdb_from_ut1`, and `tdb_from_utc`.
- [`pleiades-core::ChartRequest`](../crates/pleiades-core/src/chart.rs): mirrors the same explicit conversion helpers for chart assembly and adds `validate_time_scale_conversion()` plus `validate_house_observer_policy()` so the façade can preflight request shape before dispatch.
- [`pleiades-backend::EphemerisRequest`](../crates/pleiades-backend/src/lib.rs): exposes the direct-backend conversion helpers and `validate_time_scale_conversion()` / `validate_against_metadata()` / `validate_requests_against_metadata()` so backend callers can check the same explicit contract before dispatch, whether they are validating one request or a batch.
- [`pleiades-houses::HouseRequest`](../crates/pleiades-houses/src/houses.rs): uses the observer for house calculations only and validates observer latitude/longitude, obliquity overrides, and topocentric elevation up front; it is not a body-position time-conversion surface.
- [`pleiades-cli chart`](../crates/pleiades-cli/src/main.rs): provides the CLI-facing `--tt`, `--tdb`, `--utc`, `--ut1`, `--tt-offset-seconds`, `--tdb-offset-seconds`, and `--tt-from-tdb-offset-seconds` flags so the same caller-supplied policy can be exercised from the command line.

The table below summarizes the current responsibility split between the typed request layers and their caller-supplied conversion helpers:

| Layer | Time-scale surface | Who supplies conversion policy? | Library behavior |
| --- | --- | --- | --- |
| `pleiades-types::Instant` | tagged instant plus explicit retagging helpers | caller supplies the offset or `TimeScaleConversion` | validates and retags only; no built-in Delta T, leap-second, DUT1, or relativistic model |
| `pleiades-core::ChartRequest` | chart-level retagging and preflight helpers | caller supplies the offset or `TimeScaleConversion` before chart assembly | validates and retags only; preserves the rest of the chart request shape |
| `pleiades-backend::EphemerisRequest` | direct-backend preflight helpers | caller supplies the offset or `TimeScaleConversion` before dispatch | validates the explicit conversion contract; backend queries still expect the request scale already chosen by the caller |
| `pleiades-houses::HouseRequest` | house-observer requests only | not a time-scale conversion surface | uses the observer for house calculations, and `HouseRequest::validate()` preflights the observer latitude/longitude, any obliquity override, and topocentric elevation before calculation; it does not infer body-position time conversion |

Batch callers should use `validate_requests_against_metadata()` on slices before dispatch so batch support and per-request policy failures surface with the same explicit contract as single-request preflight.

Validation and release summaries also surface a compact `Primary request surfaces:` line that mirrors this split so the entry-point contract stays visible in report output, not just in the prose table.

## Current request-scale contract

The current contract is intentionally mechanical rather than modeled:

- `pleiades-types::Instant`, `pleiades-core::ChartRequest`, and `pleiades-backend::EphemerisRequest` carry a caller-tagged instant; they only retag that instant when the caller supplies an explicit conversion policy.
- The first-party ephemeris backends accept TT/TDB request instants for position queries. UTC/UT1 inputs are the caller's responsibility until they are converted with the explicit helpers above.
- `pleiades-houses::HouseRequest` is not a body-position time-conversion surface; it uses the observer only for house calculations, and `HouseRequest::validate()` preflights the observer latitude/longitude, any obliquity override, and topocentric elevation before calculation.
- `pleiades-cli chart` mirrors the same explicit policy with `--utc`, `--ut1`, `--tt`, `--tdb`, and the offset flags that feed the typed retagging helpers.

`pleiades-types` provides mechanical caller-supplied offset helpers: `JulianDay::add_seconds`, `Instant::with_time_scale_offset`, `Instant::with_time_scale_offset_checked`, `Instant::with_time_scale_conversion`, `Instant::validate_time_scale_conversion`, `Instant::tt_from_ut1`, `Instant::tt_from_ut1_signed`, `Instant::tt_from_utc`, `Instant::tt_from_utc_signed`, `Instant::tt_from_tdb`, `Instant::tt_from_tdb_signed`, `Instant::tdb_from_tt`, `Instant::tdb_from_tt_signed`, `Instant::tdb_from_ut1`, `Instant::tdb_from_ut1_signed`, `Instant::tdb_from_utc`, and `Instant::tdb_from_utc_signed`. These helpers make an already chosen conversion policy explicit (`target - source` seconds) but do not model Delta T, UTC leap seconds, or relativistic TDB terms themselves. The new `TimeScaleConversion` policy record packages a source scale, target scale, and caller-supplied offset into one typed value so applications can keep the explicit conversion contract alongside the instant. Its compact `summary_line()`/`Display` rendering names the structured `source=...; target=...; offset_seconds=...` fields directly, which keeps the diagnostics aligned with the typed policy record instead of inferring the offset direction from an arrow notation. The policy record also exposes `TimeScaleConversion::validate(instant)` so callers can preflight the source-scale match and finite-offset check without mutating the instant yet, and `Instant::validate_time_scale_conversion()` mirrors that check at the foundation layer when callers want the validation to sit on the instant itself. `pleiades-core::ChartRequest::validate_time_scale_conversion()` and `pleiades-backend::EphemerisRequest::validate_time_scale_conversion()` expose the same preflight at the façade and direct-backend layers, so the explicit conversion contract can be checked wherever the request enters the workspace. The signed helpers reject non-finite offsets with a structured time-scale conversion error so malformed caller policy fails explicitly instead of propagating `NaN`/infinite instants, and that error now has a stable `summary_line()`/`Display` rendering for diagnostics and release-facing summaries. `pleiades-core::ChartRequest` also exposes `with_instant_time_scale_offset_checked()` for the same checked chart-level convenience when the caller wants the façade to reject non-finite offsets before retagging the instant.
- `pleiades-core::ChartRequest` mirrors that policy with builder conveniences for applying a caller-supplied instant offset or converting UT1-tagged, UTC-tagged, TT-tagged, or TDB-tagged chart requests to TT/TDB before chart assembly. UT1-tagged and UTC-tagged requests can now be lifted directly to TT with caller-supplied signed or unsigned offsets, UTC-tagged requests can now also be lifted directly to TDB through caller-supplied TT-UTC and TDB-TT offsets, and TDB-tagged requests can be converted back to TT with a caller-supplied signed offset or its `*_signed` alias. When a signed TDB-TT correction is needed, the `*_signed` TDB helpers preserve that policy explicitly instead of forcing the caller to pre-quantize the offset into a non-negative duration. `ChartRequest::validate_time_scale_conversion()` lets callers preflight the same explicit source/target/offset contract against the request instant before mutating it, `ChartRequest::validate_house_observer_policy()` preflights the house-observer contract so house placement continues to stay separate from geocentric body-position requests, and `ChartRequest::validate_against_metadata()` preflights the chart request shape against backend metadata so body coverage, zodiac routing, frame support, apparentness, and the house-observer contract can fail fast before chart assembly. These conversion helpers only retag the instant; they preserve the rest of the chart request shape, and `ChartRequest::summary_line()` continues to render the converted instant together with the same bodies, observer, apparentness, zodiac, and house-system details for diagnostics and release-facing request summaries. It also preserves custom house-system names, aliases, and notes instead of collapsing them to an opaque `Custom` label.
- `pleiades-core::ChartRequest::observer_policy()` and `pleiades-core::ChartSnapshot::observer_policy()` expose the same geocentric-versus-house-only decision as a typed helper, so callers that need to inspect the observer posture do not need to rederive the summary text from scratch.
- `pleiades-core::ChartSnapshot::summary_line()` mirrors the same request-shape vocabulary for computed charts, adding the backend ID and placement count so release-facing snapshot summaries can stay aligned with the request contract without reopening the verbose report.
- The shared validation and release report layers reuse `pleiades-backend::request_policy_summary_for_report()` so the compact summary wording for the time-scale, observer, apparentness, and frame policy stays anchored to one typed backend record instead of being rebuilt separately in each reporting crate. The report renderers now also validate that shared summary before formatting it, which makes whitespace-padded or blank policy drift fail closed instead of being rendered into release-facing text. The direct metadata preflight helper (`pleiades-backend::validate_request_against_metadata()`) now also applies the current tropical-only zodiac guardrail when a backend does not advertise native sidereal support, so tropical-only backends can reject sidereal requests before dispatch even when callers bypass the façade.
- `pleiades-houses::HouseRequest` mirrors the same explicit instant/observer/system/obliquity shape for direct house calculations, and `HouseRequest::summary_line()` / `fmt::Display` expose that request shape for diagnostics and validation reports without introducing hidden policy. `HouseRequest::validate()` now also rejects non-finite observer longitudes alongside the existing latitude, obliquity, and topocentric elevation checks so malformed observer locations fail fast before house derivation.
- `pleiades-cli chart` exposes matching `--utc` / `--ut1` tags plus explicit `--tt-offset-seconds` and signed `--tdb-offset-seconds` flags so the command-line chart workflow can exercise the same caller-supplied conversion policy without introducing built-in Delta T or relativistic modeling. TDB-tagged chart instants can also be explicitly re-tagged back to TT with `--tt-from-tdb-offset-seconds` when a caller already starts from TDB. The CLI rejects offset flags that do not match the tagged instant, so invalid combinations fail explicitly instead of being silently ignored.
- `pleiades-backend` exposes shared request-shape helpers (`validate_request_policy`, `validate_zodiac_policy`, and `validate_observer_policy`) so backend crates can enforce the same time-scale, frame, zodiac-mode, apparentness, and observer rules without duplicating the policy checks.
- `pleiades-backend::current_request_policy_summary()` centralizes the compact report wording for those shared request-shape rules so validation and release summaries can reuse one typed summary record instead of reconstructing the policy block ad hoc.
- The default batch adapter preserves those same structured failures request-by-request, so a batch that includes an unsupported apparentness or observer shape still fails with the backend's explicit error kind rather than a generic batch wrapper error.
- The default batch adapter also preserves each request's own time-scale label, so mixed TT/TDB batches remain mixed in the results instead of being normalized to a hidden batch-wide scale.
- Validation, release, and backend-matrix summaries now include a short observer policy line alongside the time-scale policy line so the geocentric-only posture remains visible without opening the full policy document.
- Until a project-level Delta T model is implemented, validation fixtures and reports should state the time scale of each epoch explicitly and should not imply automatic UTC-to-TT or UTC-to-TDB conversion support.

## Apparent versus mean coordinates

- Current first-party source/data backends report **mean geometric** coordinates only.
- Backends whose metadata has `capabilities.apparent = false` must reject `Apparentness::Apparent` requests instead of silently returning mean coordinates.
- The current first-party backends are tropical-only at the backend layer; sidereal requests are handled by the chart façade when supported or rejected explicitly when a backend is queried directly.
- `pleiades-backend::EphemerisRequest::new` defaults to `Apparentness::Mean` so bare requests line up with the current mean-only first-party backends.
- `pleiades-core::ChartSnapshot` now renders an explicit apparentness policy line so chart output states whether the snapshot was built from a mean or apparent request before backend-specific accuracy details are consulted.
- Light-time, aberration, deflection, nutation, and related apparent-place corrections are planned production work and must be documented per backend when implemented.

## Observer and topocentric behavior

- Chart-level observer locations are currently used for house calculations.
- Body positions in chart assembly are queried geocentrically unless a future API adds an explicit topocentric position mode.
- Geocentric-only backends must reject direct backend requests that include an observer location with a structured `InvalidObserver` error.
- House calculations validate obliquity overrides up front; non-finite overrides are rejected with a structured invalid-obliquity house error instead of flowing into the quadrant formulas.
- This separation prevents an observer used for houses from being mistaken for topocentric planetary or lunar coordinates.

## Frame behavior

- Ecliptic and equatorial result fields may both be populated by a backend, but the request frame still records the caller's requested output frame.
- The shared type layer exposes the mean-obliquity forward rotation as `EclipticCoordinates::to_equatorial` and the inverse rotation as `EquatorialCoordinates::to_ecliptic`, so callers and backend tests can round-trip frame conversions with the same documented obliquity policy.
- Frame transforms must document the obliquity/precession/nutation model used. The current VSOP87 source-backed path and the compact ELP lunar baseline both use a mean-obliquity transform, and that frame rotation should stay documented explicitly until a higher-order model is introduced.

## Follow-up work

Production accuracy work should add:

1. a documented UTC/UT/TT/TDB conversion strategy if Pleiades adopts built-in Delta T/leap-second modeling beyond caller-supplied offsets;
2. backend-specific apparent-place correction support or structured rejection tests;
3. validation reports that label every reference epoch with its time scale;
4. topocentric position support only behind an explicit request/configuration surface.
