# pleiades-events

Ephemeris event-finding for the `pleiades` astrology workspace: longitude crossings of the Sun, Moon, and planets, and their ecliptic positions and speeds.

`EventEngine::position_at(body, frame, instant)` returns the full ecliptic
position in either `CrossingFrame`: longitude, latitude and distance, with
their speeds per day. Its longitude is exactly the one `longitude_at` returns,
so a position is consistent with the crossings found in the same frame.

- `GeocentricApparentOfDate`: the apparent place in the true ecliptic of date,
  with the same speed a `pleiades-core` apparent chart reports.
- `Heliocentric`: the geometric place from the Sun (no light-time, no
  aberration) in the true ecliptic and equinox of date — Swiss Ephemeris
  `SEFLG_HELCTR | SEFLG_TRUEPOS` — gated by `validate-helio-position`. Plain
  `SEFLG_HELCTR` output is retarded by the heliocentric light-time and differs
  by up to ≈ 41″ (Mercury). The Sun and Moon are an error in this frame.
- `GeocentricMeanOfDate`: the geometric place from the Earth's centre (no
  light-time, no aberration, no nutation) in the mean ecliptic and equinox of
  date. This is not the J2000 longitude a `pleiades-core` mean chart reports.

A speed channel is `None` when the backend reports no speed to derive it from.

Every crossing and position method takes a `CrossingFrame` (tropical zodiac) or
a `CrossingReference`, which adds a zodiac:

    let lahiri = CrossingReference::sidereal(
        CrossingFrame::GeocentricApparentOfDate,
        Ayanamsa::Lahiri,
    );
    let ingress = engine.next_longitude_crossing(
        CelestialBody::Sun,
        Longitude::from_degrees(0.0),
        lahiri,
        after,
    )?;

A sidereal longitude is the longitude on the mean equinox of date minus the
mean ayanamsa, the Swiss Ephemeris `SEFLG_SIDEREAL` convention: nutation does
not move a body through a sidereal zodiac. The ayanamsa is evaluated at every
trial instant of the search. The heliocentric frame is tropical only, and an
ayanamsa without offset data is an error, never a silent tropical result. For
the star-anchored ayanamsa classes (`TrueStar` and `Galactic`) the mean
ayanamsa is used in every frame, whereas Swiss Ephemeris's apparent sidereal
positions add the anchoring star's annual aberration (up to about 20″) to the
ayanamsa. That is about 8 minutes of crossing time for the Sun and hours for a
slow planet such as Saturn, more near a station. True Citra and Galactic Center
were measured; the other ayanamsas in those classes follow from the same
mechanism and were not.
Mean-of-date and sidereal crossings are gated by `validate-crossings`.
