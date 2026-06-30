# pleiades-apsides

Osculating lunar apsides (true apogee / true perigee) for the `pleiades`
workspace. Given the Moon's geocentric position and velocity vectors and the
Earth–Moon gravitational parameter, computes the instantaneous Keplerian apogee
and perigee directions as ecliptic unit vectors. Used by `pleiades-data` to
derive release-grade `TrueApogee` and `TruePerigee` positions from the packaged
Moon state.
