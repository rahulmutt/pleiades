# pleiades-apsides

Osculating lunar apsides (true apogee / true perigee) for the `pleiades`
workspace. Given the Moon's geocentric position and velocity vectors and the
Earth–Moon gravitational parameter, computes the instantaneous Keplerian apogee
and perigee as ecliptic longitude/latitude with geocentric distance (plus the
osculating ellipse's eccentricity and semi-major axis). Used by `pleiades-data`
to derive release-grade `TrueApogee` and `TruePerigee` positions from the
packaged Moon state; that end-to-end path is gated against Swiss Ephemeris
`SE_OSCU_APOG` by `validate-lilith` (max longitude residual ~306″).
