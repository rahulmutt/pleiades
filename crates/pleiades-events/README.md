# pleiades-events

Ephemeris event-finding for the `pleiades` astrology workspace: longitude crossings of the Sun, Moon, and planets.

`EventEngine::position_at(body, frame, instant)` returns the full ecliptic
position in either `CrossingFrame`: longitude, latitude and distance, with
their speeds per day. Its longitude is exactly the one `longitude_at` returns,
so a position is consistent with the crossings found in the same frame.

- `GeocentricApparentOfDate`: the apparent place in the true ecliptic of date,
  with the same speed a `pleiades-core` apparent chart reports.
- `Heliocentric`: the geometric place from the Sun in the true ecliptic and
  equinox of date (Swiss Ephemeris `SEFLG_HELCTR`), gated by
  `validate-helio-position`. The Sun and Moon are an error in this frame.

A speed channel is `None` when the backend reports no speed to derive it from.
