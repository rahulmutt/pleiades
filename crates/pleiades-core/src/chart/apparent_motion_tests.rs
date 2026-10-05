use core::f64::consts::TAU;

use pleiades_backend::{
    AccuracyClass, Apparentness, BackendCapabilities, BackendFamily, BackendId, BackendMetadata,
    BackendProvenance, BodyClaim, ClaimEvidence, EphemerisBackend, EphemerisError,
    EphemerisErrorKind, EphemerisRequest, EphemerisResult, QualityAnnotation,
};
use pleiades_types::{
    CelestialBody, EclipticCoordinates, Instant, JulianDay, Latitude, Longitude, Motion,
    MotionDirection, TimeRange, TimeScale,
};

use crate::chart::{BodyPlacement, ChartEngine, ChartRequest};

/// Epoch the smooth backend's analytic series are counted from.
const EPOCH_JD: f64 = 2_460_000.5;
/// A generic instant, away from every turning point of the analytic series.
const SAMPLE_JD: f64 = EPOCH_JD + 37.25;
/// The smooth Mars turns retrograde here in the mean place: its longitude is
/// `77 + 5 sin(2π d / 400)`, whose speed crosses zero at `d = 100`.
const MEAN_STATION_JD: f64 = EPOCH_JD + 100.0;

/// One analytic position sample with its exact time derivative.
struct Sample {
    longitude_deg: f64,
    latitude_deg: f64,
    distance_au: f64,
    motion: Motion,
}

/// An analytic backend whose positions are smooth functions of time and whose
/// motion is their exact derivative, so a finite difference of the chart's
/// output measures the chart layer alone. Distances vary slowly enough that
/// the light-time iteration always converges in the same number of steps.
#[derive(Clone, Copy)]
struct SmoothBackend {
    /// When set, only instants inside this closed Julian-day range are served.
    range_jd: Option<(f64, f64)>,
}

impl SmoothBackend {
    const fn unbounded() -> Self {
        Self { range_jd: None }
    }

    fn sample(body: &CelestialBody, jd: f64) -> Option<Sample> {
        let d = jd - EPOCH_JD;
        let sample = match body {
            CelestialBody::Sun => Sample {
                longitude_deg: 280.0 + 0.985_647 * d,
                latitude_deg: 0.0,
                distance_au: 1.0,
                motion: Motion::new(Some(0.985_647), Some(0.0), Some(0.0)),
            },
            CelestialBody::Mars | CelestialBody::Jupiter => {
                let (w_lon, w_lat, w_dist) = (TAU / 400.0, TAU / 300.0, TAU / 700.0);
                Sample {
                    longitude_deg: 77.0 + 5.0 * (w_lon * d).sin(),
                    latitude_deg: 1.5 * (w_lat * d + 0.4).sin(),
                    distance_au: 1.5 + 0.3 * (w_dist * d).cos(),
                    motion: Motion::new(
                        Some(5.0 * w_lon * (w_lon * d).cos()),
                        Some(1.5 * w_lat * (w_lat * d + 0.4).cos()),
                        Some(-0.3 * w_dist * (w_dist * d).sin()),
                    ),
                }
            }
            CelestialBody::TrueNode => {
                let w = TAU / 173.0;
                Sample {
                    longitude_deg: 30.0 - 0.052_9 * d + 1.5 * (w * d).sin(),
                    latitude_deg: 0.0,
                    distance_au: 0.002_5,
                    motion: Motion::new(
                        Some(-0.052_9 + 1.5 * w * (w * d).cos()),
                        Some(0.0),
                        Some(0.0),
                    ),
                }
            }
            _ => return None,
        };
        Some(sample)
    }
}

impl EphemerisBackend for SmoothBackend {
    fn metadata(&self) -> BackendMetadata {
        let release_grade = |body| {
            BodyClaim::release_grade(
                body,
                AccuracyClass::Approximate,
                ClaimEvidence::AlgorithmicModel,
            )
        };
        BackendMetadata {
            id: BackendId::new("smooth"),
            version: "0.1.0".to_string(),
            family: BackendFamily::Algorithmic,
            provenance: BackendProvenance::new("smooth analytic backend"),
            nominal_range: TimeRange::new(None, None),
            supported_time_scales: vec![TimeScale::Tt],
            body_claims: vec![
                release_grade(CelestialBody::Sun),
                release_grade(CelestialBody::Mars),
                release_grade(CelestialBody::TrueNode),
                // Served without a distance (see `position`), so an apparent
                // chart falls back to its mean place.
                BodyClaim::approximate(CelestialBody::Jupiter),
            ],
            supported_frames: vec![pleiades_types::CoordinateFrame::Ecliptic],
            capabilities: BackendCapabilities::default(),
            accuracy: AccuracyClass::Approximate,
            deterministic: true,
            offline: true,
        }
    }

    fn supports_body(&self, body: CelestialBody) -> bool {
        Self::sample(&body, EPOCH_JD).is_some()
    }

    fn position(&self, request: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        let jd = request.instant.julian_day.days();
        if self
            .range_jd
            .is_some_and(|(first, last)| !(first..=last).contains(&jd))
        {
            return Err(EphemerisError::new(
                EphemerisErrorKind::OutOfRangeInstant,
                "outside the smooth backend's range",
            ));
        }
        let sample = Self::sample(&request.body, jd).ok_or_else(|| {
            EphemerisError::new(EphemerisErrorKind::UnsupportedBody, "unsupported")
        })?;
        let mut result = EphemerisResult::new(
            BackendId::new("smooth"),
            request.body.clone(),
            request.instant,
            request.frame,
            request.zodiac_mode.clone(),
            request.apparent,
        );
        result.quality = QualityAnnotation::Approximate;
        // Jupiter is served without a distance, so the light-time step fails
        // closed for it and an apparent chart keeps its mean place.
        let distance_au = (request.body != CelestialBody::Jupiter).then_some(sample.distance_au);
        result.ecliptic = Some(EclipticCoordinates::new(
            Longitude::from_degrees(sample.longitude_deg),
            Latitude::from_degrees(sample.latitude_deg),
            distance_au,
        ));
        result.motion = Some(sample.motion);
        Ok(result)
    }
}

fn placement<B: EphemerisBackend>(
    backend: B,
    body: &CelestialBody,
    jd: f64,
    scale: TimeScale,
    apparentness: Apparentness,
) -> BodyPlacement {
    let request = ChartRequest::new(Instant::new(JulianDay::from_days(jd), scale))
        .with_bodies(vec![body.clone()])
        .with_apparentness(apparentness);
    ChartEngine::new(backend)
        .chart(&request)
        .expect("chart should succeed")
        .placement_for(body)
        .expect("requested body must be placed")
        .clone()
}

fn apparent_smooth(body: &CelestialBody, jd: f64) -> BodyPlacement {
    placement(
        SmoothBackend::unbounded(),
        body,
        jd,
        TimeScale::Tt,
        Apparentness::Apparent,
    )
}

fn ecliptic(placement: &BodyPlacement) -> EclipticCoordinates {
    placement
        .position
        .ecliptic
        .expect("placement must carry ecliptic coordinates")
}

fn motion(placement: &BodyPlacement) -> Motion {
    placement
        .position
        .motion
        .expect("placement must carry motion")
}

/// Signed difference `a - b` of two longitudes, wrapped to `[-180, 180)`.
fn wrapped_difference(a: f64, b: f64) -> f64 {
    (a - b + 180.0).rem_euclid(360.0) - 180.0
}

/// Fourth-order five-point derivative of `value` at `jd`, in units per day.
/// `difference` subtracts two samples, so longitudes can wrap.
fn stencil_rate(value: impl Fn(f64) -> f64, difference: impl Fn(f64, f64) -> f64, jd: f64) -> f64 {
    const STEP_DAYS: f64 = 0.5;
    let near = difference(value(jd + STEP_DAYS), value(jd - STEP_DAYS));
    let far = difference(value(jd + 2.0 * STEP_DAYS), value(jd - 2.0 * STEP_DAYS));
    (8.0 * near - far) / (12.0 * STEP_DAYS)
}

/// The rate of change of the smooth chart's own apparent longitude.
fn apparent_longitude_rate(body: &CelestialBody, jd: f64) -> f64 {
    stencil_rate(
        |jd| ecliptic(&apparent_smooth(body, jd)).longitude.degrees(),
        wrapped_difference,
        jd,
    )
}

fn assert_longitude_speed_is_apparent(body: CelestialBody) {
    let reference = apparent_longitude_rate(&body, SAMPLE_JD);
    let mean = SmoothBackend::sample(&body, SAMPLE_JD)
        .expect("smooth body")
        .motion
        .longitude_deg_per_day
        .expect("smooth longitude speed");
    assert!(
        (mean - reference).abs() > 5e-6,
        "{body}: the mean speed {mean} must differ from the apparent rate {reference} \
         for this test to discriminate"
    );

    let speed = motion(&apparent_smooth(&body, SAMPLE_JD))
        .longitude_deg_per_day
        .expect("apparent longitude speed");
    assert!(
        (speed - reference).abs() < 1e-6,
        "{body}: longitude speed {speed} deg/day should be the apparent rate {reference}"
    );
}

#[test]
fn apparent_planet_longitude_speed_is_the_rate_of_its_apparent_longitude() {
    assert_longitude_speed_is_apparent(CelestialBody::Mars);
}

#[test]
fn apparent_sun_longitude_speed_is_the_rate_of_its_apparent_longitude() {
    assert_longitude_speed_is_apparent(CelestialBody::Sun);
}

#[test]
fn apparent_lunar_point_longitude_speed_is_the_rate_of_its_apparent_longitude() {
    assert_longitude_speed_is_apparent(CelestialBody::TrueNode);
}

#[test]
fn apparent_latitude_speed_is_the_rate_of_the_apparent_latitude() {
    let body = CelestialBody::Mars;
    let reference = stencil_rate(
        |jd| ecliptic(&apparent_smooth(&body, jd)).latitude.degrees(),
        |a, b| a - b,
        SAMPLE_JD,
    );
    let mean = SmoothBackend::sample(&body, SAMPLE_JD)
        .expect("smooth body")
        .motion
        .latitude_deg_per_day
        .expect("smooth latitude speed");
    assert!(
        (mean - reference).abs() > 5e-6,
        "the mean latitude speed {mean} must differ from the apparent rate {reference}"
    );

    let speed = motion(&apparent_smooth(&body, SAMPLE_JD))
        .latitude_deg_per_day
        .expect("apparent latitude speed");
    assert!(
        (speed - reference).abs() < 1e-6,
        "latitude speed {speed} deg/day should be the apparent rate {reference}"
    );
}

#[test]
fn apparent_distance_speed_is_the_rate_of_the_light_time_retarded_distance() {
    let body = CelestialBody::Mars;
    let reference = stencil_rate(
        |jd| {
            ecliptic(&apparent_smooth(&body, jd))
                .distance_au
                .expect("apparent distance")
        },
        |a, b| a - b,
        SAMPLE_JD,
    );
    let mean = SmoothBackend::sample(&body, SAMPLE_JD)
        .expect("smooth body")
        .motion
        .distance_au_per_day
        .expect("smooth distance speed");
    assert!(
        (mean - reference).abs() > 1e-7,
        "the mean distance speed {mean} must differ from the apparent rate {reference}"
    );

    let speed = motion(&apparent_smooth(&body, SAMPLE_JD))
        .distance_au_per_day
        .expect("apparent distance speed");
    assert!(
        (speed - reference).abs() < 1e-8,
        "distance speed {speed} au/day should be the apparent rate {reference}"
    );
}

#[test]
fn apparent_motion_direction_follows_the_apparent_longitude_near_a_station() {
    let body = CelestialBody::Mars;

    // Bisect the apparent station: the instant the apparent longitude turns.
    let (mut before, mut after) = (MEAN_STATION_JD - 5.0, MEAN_STATION_JD + 5.0);
    assert!(apparent_longitude_rate(&body, before) > 0.0);
    assert!(apparent_longitude_rate(&body, after) < 0.0);
    for _ in 0..40 {
        let middle = 0.5 * (before + after);
        if apparent_longitude_rate(&body, middle) > 0.0 {
            before = middle;
        } else {
            after = middle;
        }
    }
    let apparent_station_jd = 0.5 * (before + after);
    assert!(
        (apparent_station_jd - MEAN_STATION_JD).abs() > 0.01,
        "the apparent station {apparent_station_jd} must be separated from the mean station"
    );

    // Between the two stations the mean and apparent places move opposite ways.
    let between_jd = 0.5 * (apparent_station_jd + MEAN_STATION_JD);
    let apparent_direction = if apparent_longitude_rate(&body, between_jd) > 0.0 {
        MotionDirection::Direct
    } else {
        MotionDirection::Retrograde
    };
    let mean_direction = placement(
        SmoothBackend::unbounded(),
        &body,
        between_jd,
        TimeScale::Tt,
        Apparentness::Mean,
    )
    .motion_direction()
    .expect("mean direction");
    assert_ne!(mean_direction, apparent_direction);

    assert_eq!(
        apparent_smooth(&body, between_jd).motion_direction(),
        Some(apparent_direction)
    );
}

#[test]
fn apparent_speed_survives_a_neighbour_instant_out_of_range() {
    let body = CelestialBody::Mars;
    let reference = apparent_longitude_rate(&body, SAMPLE_JD);

    // The backend ends 0.2 day after the chart instant, so the later
    // neighbour of the speed difference is out of range.
    let bounded = SmoothBackend {
        range_jd: Some((EPOCH_JD, SAMPLE_JD + 0.2)),
    };
    let placed = placement(
        bounded,
        &body,
        SAMPLE_JD,
        TimeScale::Tt,
        Apparentness::Apparent,
    );

    assert_eq!(placed.position.apparent, Apparentness::Apparent);
    let speed = motion(&placed)
        .longitude_deg_per_day
        .expect("apparent longitude speed");
    assert!(
        (speed - reference).abs() < 1e-5,
        "one-sided longitude speed {speed} deg/day should be near the apparent rate {reference}"
    );
}

/// Serves the Sun only inside `sun_range_jd` and every other body of the
/// unbounded smooth backend everywhere: a Sun source whose window is narrower
/// than a body's, like a routed chain whose first backend bounds the Sun.
struct WindowedSunBackend {
    sun_range_jd: (f64, f64),
}

impl EphemerisBackend for WindowedSunBackend {
    fn metadata(&self) -> BackendMetadata {
        SmoothBackend::unbounded().metadata()
    }

    fn supports_body(&self, body: CelestialBody) -> bool {
        SmoothBackend::unbounded().supports_body(body)
    }

    fn position(&self, request: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        let (first, last) = self.sun_range_jd;
        if request.body == CelestialBody::Sun
            && !(first..=last).contains(&request.instant.julian_day.days())
        {
            return Err(EphemerisError::new(
                EphemerisErrorKind::OutOfRangeInstant,
                "outside the windowed Sun's range",
            ));
        }
        SmoothBackend::unbounded().position(request)
    }
}

#[test]
fn apparent_speed_is_central_when_only_the_sun_window_ends() {
    // The Sun's window ends 0.2 day after the chart instant, inside the
    // half-span of the speed difference, while Mars is served everywhere.
    // Before issue #128 the later neighbour was dropped because the backend
    // could not serve the Sun there, so Mars's speed was one-sided. The
    // aberration Sun is now the backend-free Meeus Sun, so a neighbour is
    // dropped only when the body's own sample fails, and the speed is the
    // central difference, bit for bit.
    let body = CelestialBody::Mars;
    let windowed_sun = WindowedSunBackend {
        sun_range_jd: (EPOCH_JD, SAMPLE_JD + 0.2),
    };
    let placed = placement(
        windowed_sun,
        &body,
        SAMPLE_JD,
        TimeScale::Tt,
        Apparentness::Apparent,
    );
    let central = apparent_smooth(&body, SAMPLE_JD);

    assert_eq!(placed.position.apparent, Apparentness::Apparent);
    assert_eq!(ecliptic(&placed), ecliptic(&central));
    assert_eq!(motion(&placed), motion(&central));

    // A one-sided difference has different bits here, so the equality above
    // tells the central difference from the pre-#128 one-sided fallback.
    let one_sided = placement(
        SmoothBackend {
            range_jd: Some((EPOCH_JD, SAMPLE_JD + 0.2)),
        },
        &body,
        SAMPLE_JD,
        TimeScale::Tt,
        Apparentness::Apparent,
    );
    assert_ne!(motion(&one_sided), motion(&central));
}

#[test]
fn apparent_speed_is_unknown_when_no_neighbour_instant_is_served() {
    let body = CelestialBody::Mars;
    let isolated = SmoothBackend {
        range_jd: Some((SAMPLE_JD - 0.2, SAMPLE_JD + 0.2)),
    };
    let placed = placement(
        isolated,
        &body,
        SAMPLE_JD,
        TimeScale::Tt,
        Apparentness::Apparent,
    );

    // The place is apparent, so the backend's mean-place speed no longer
    // describes it; without a neighbour the apparent speed is unknown.
    assert_eq!(placed.position.apparent, Apparentness::Apparent);
    assert_eq!(placed.position.motion, None);
    assert_eq!(placed.motion_direction(), None);
}

#[test]
fn mean_fallback_placement_keeps_the_backend_motion() {
    let body = CelestialBody::Jupiter;
    let placed = apparent_smooth(&body, SAMPLE_JD);

    assert_eq!(placed.position.apparent, Apparentness::Mean);
    assert_eq!(
        motion(&placed),
        SmoothBackend::sample(&body, SAMPLE_JD)
            .expect("smooth body")
            .motion
    );
}

#[test]
fn mean_chart_keeps_the_backend_motion() {
    let body = CelestialBody::Mars;
    let placed = placement(
        SmoothBackend::unbounded(),
        &body,
        SAMPLE_JD,
        TimeScale::Tt,
        Apparentness::Mean,
    );

    assert_eq!(
        motion(&placed),
        SmoothBackend::sample(&body, SAMPLE_JD)
            .expect("smooth body")
            .motion
    );
}

/// Regression for issue #91: Mars at JD 2460000.5 TDB on the packaged backend
/// reported the mean-place speed 0.378709 deg/day next to an apparent
/// longitude whose own rate is 0.378611 deg/day.
#[test]
fn packaged_mars_apparent_speed_matches_its_apparent_longitude() {
    use pleiades_data::PackagedDataBackend;

    let body = CelestialBody::Mars;
    let apparent = |jd| {
        placement(
            PackagedDataBackend::new(),
            &body,
            jd,
            TimeScale::Tdb,
            Apparentness::Apparent,
        )
    };
    let reference = stencil_rate(
        |jd| ecliptic(&apparent(jd)).longitude.degrees(),
        wrapped_difference,
        EPOCH_JD,
    );

    let speed = motion(&apparent(EPOCH_JD))
        .longitude_deg_per_day
        .expect("apparent longitude speed");
    assert!(
        (speed - reference).abs() < 2e-5,
        "longitude speed {speed} deg/day should be the apparent rate {reference}"
    );
}

/// Every packaged planet, direct and retrograde, across the packaged range.
/// The backend's fitted speeds leave a residual below 1e-6 deg/day against the
/// stencil; the mean-place speed is up to 4e-4 deg/day away.
#[test]
fn packaged_planet_apparent_speeds_match_their_apparent_longitudes() {
    use pleiades_data::PackagedDataBackend;

    for body in [
        CelestialBody::Sun,
        CelestialBody::Mercury,
        CelestialBody::Venus,
        CelestialBody::Mars,
        CelestialBody::Jupiter,
        CelestialBody::Saturn,
        CelestialBody::Uranus,
        CelestialBody::Neptune,
        CelestialBody::Pluto,
    ] {
        for jd in [2_415_100.25, 2_451_545.0, 2_470_000.125] {
            let apparent = |jd| {
                placement(
                    PackagedDataBackend::new(),
                    &body,
                    jd,
                    TimeScale::Tdb,
                    Apparentness::Apparent,
                )
            };
            let reference = stencil_rate(
                |jd| ecliptic(&apparent(jd)).longitude.degrees(),
                wrapped_difference,
                jd,
            );

            let speed = motion(&apparent(jd))
                .longitude_deg_per_day
                .expect("apparent longitude speed");
            assert!(
                (speed - reference).abs() < 2e-6,
                "{body} at JD {jd}: longitude speed {speed} deg/day should be the apparent \
                 rate {reference}"
            );
        }
    }
}
