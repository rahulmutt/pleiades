//! Civil datetimes of event instants (issue #165).
//!
//! Every event result carries a TDB [`Instant`](pleiades_types::Instant). A
//! caller who wants the calendar date and clock time used to pass it to
//! `pleiades_time::civil_from_tdb`; the `civil` methods here do that.

use pleiades_time::{civil_from_tdb, CivilConversion, CivilTimeError};

use crate::{AspectEvent, Crossing, RiseSet, Station};

macro_rules! impl_civil {
    ($($result:ty),+ $(,)?) => {$(
        impl $result {
            /// The civil datetime of [`instant`](Self::instant): UTC from
            /// 1972-01-01 on and UT1 before, rounded to the millisecond.
            /// [`CivilConversion::scale`] says which, and its provenance
            /// carries the quality tier of the Delta T or leap-second data
            /// used.
            ///
            /// This is `pleiades_time::civil_from_tdb(self.instant)`; the
            /// result and error types are `pleiades-time`'s.
            ///
            /// # Errors
            ///
            /// Those of `pleiades_time::civil_from_tdb`. None arises for an
            /// instant this engine returned; a result whose `instant` was
            /// replaced with one not tagged TDB is
            /// [`CivilTimeError::UnsupportedScale`].
            pub fn civil(&self) -> Result<CivilConversion, CivilTimeError> {
                civil_from_tdb(self.instant)
            }
        }
    )+};
}

impl_civil!(Crossing, Station, AspectEvent, RiseSet);

#[cfg(test)]
mod tests {
    use pleiades_time::{civil_from_tdb, CivilDateTime, CivilTimeError};
    use pleiades_types::{
        Angle, CelestialBody, Instant, JulianDay, Longitude, TimeScale, ZodiacMode,
    };

    use crate::{
        AspectEvent, Crossing, CrossingFrame, RiseSet, RiseSetEvent, RiseSetTarget, Station,
        StationKind,
    };

    const FRAME: CrossingFrame = CrossingFrame::GeocentricApparentOfDate;

    fn crossing(instant: Instant) -> Crossing {
        Crossing {
            body: CelestialBody::Sun,
            target_longitude: Longitude::from_degrees(0.0),
            instant,
            frame: FRAME,
            zodiac: ZodiacMode::Tropical,
        }
    }

    fn station(instant: Instant) -> Station {
        Station {
            body: CelestialBody::Mercury,
            instant,
            longitude: Longitude::from_degrees(0.0),
            kind: StationKind::TurnsDirect,
            frame: FRAME,
            zodiac: ZodiacMode::Tropical,
        }
    }

    fn aspect(instant: Instant) -> AspectEvent {
        AspectEvent {
            first: CelestialBody::Sun,
            second: CelestialBody::Moon,
            angle: Angle::from_degrees(90.0),
            instant,
            first_longitude: Longitude::from_degrees(90.0),
            second_longitude: Longitude::from_degrees(0.0),
            frame: FRAME,
            zodiac: ZodiacMode::Tropical,
        }
    }

    fn rise(instant: Instant) -> RiseSet {
        RiseSet {
            event: RiseSetEvent::Rise,
            instant,
            target: RiseSetTarget::Body(CelestialBody::Sun),
        }
    }

    fn tdb(jd: f64) -> Instant {
        Instant::new(JulianDay::from_days(jd), TimeScale::Tdb)
    }

    #[test]
    fn every_result_reads_its_instant_as_the_civil_time_of_its_era() {
        // JD 2460000.0 is 2023-02-24 12:00:00 TDB. TT is 69.184 s ahead of
        // UTC, and TDB a further 1.3 ms ahead of TT in late February
        // (1.657 ms × sin 51°), so UTC reads 11:58:50.8147.
        let modern = tdb(2_460_000.0);
        // JD 2430000.0 is in 1941, before UTC existed.
        let early = tdb(2_430_000.0);
        for instant in [modern, early] {
            let expected = civil_from_tdb(instant).expect("in the conversion window");
            assert_eq!(crossing(instant).civil(), Ok(expected));
            assert_eq!(station(instant).civil(), Ok(expected));
            assert_eq!(aspect(instant).civil(), Ok(expected));
            assert_eq!(rise(instant).civil(), Ok(expected));
        }
        let civil = crossing(modern).civil().expect("modern instant");
        assert_eq!(civil.scale, TimeScale::Utc);
        assert_eq!(civil.civil, CivilDateTime::new(2023, 2, 24, 11, 58, 50.815));
        assert_eq!(
            crossing(early).civil().expect("early").scale,
            TimeScale::Ut1
        );
    }

    #[test]
    fn an_instant_not_tagged_tdb_is_refused() {
        let utc = Instant::new(JulianDay::from_days(2_460_000.0), TimeScale::Utc);
        assert!(matches!(
            crossing(utc).civil(),
            Err(CivilTimeError::UnsupportedScale { .. })
        ));
    }
}
