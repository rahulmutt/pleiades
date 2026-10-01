# pleiades-apsides

Lunar orbit points (osculating and mean nodes and apsides) for the `pleiades`
workspace. Given the Moon's geocentric position and velocity vectors and the
Earth–Moon gravitational parameter, computes the instantaneous Keplerian apogee
and perigee as ecliptic longitude/latitude with geocentric distance (plus the
osculating ellipse's eccentricity and semi-major axis). Used by `pleiades-data`
to derive release-grade `TrueApogee` and `TruePerigee` positions from the
packaged Moon state; that end-to-end path is gated against Swiss Ephemeris
`SE_OSCU_APOG` by `validate-lilith` (max longitude residual ~306″). The same Kepler helpers serve the osculating true node (`TrueNode`) through `PackagedDataBackend`, gated by `validate-true-node`.

The crate also owns the **mean** lunar orbit: the Meeus mean node and mean
perigee longitudes (mean equinox of date) and the Moon's mean inclination,
eccentricity and semi-major axis, assembled by `mean_lunar_elements_of_date`.
Passing those elements to `points_from_elements` gives the mean node and the
mean apogee/perigee as Swiss Ephemeris defines them (`SE_MEAN_NODE`,
`SE_MEAN_APOG`); `pleiades-data` serves them that way, gated by
`validate-mean-lunar-points`. Directions and the apsis distances match Swiss Ephemeris; for
`SE_MEAN_NODE` Swiss Ephemeris reports the mean distance `a` rather than the
orbit radius at the node, which `pleiades-data` follows.
