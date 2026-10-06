//! The default chain never serves an asteroid position that disagrees with
//! the JPL `sb441-n373s` rows (issue #158).

use crate::commands::chart::default_chart_backend;
use pleiades_core::{
    Apparentness, CelestialBody, CoordinateFrame, CustomBodyId, EphemerisBackend, EphemerisRequest,
    ZodiacMode,
};
use pleiades_jpl::SnapshotEntry;

const TRUTH_TOLERANCE_ARCSEC: f64 = 5.0;
const SHARED_EPOCH_JD: f64 = 2_453_000.5;

/// The row's ecliptic longitude and latitude in degrees, from its ecliptic
/// Cartesian position.
fn truth_longitude_latitude(row: &SnapshotEntry) -> (f64, f64) {
    let radius_km = (row.x_km * row.x_km + row.y_km * row.y_km + row.z_km * row.z_km).sqrt();
    let longitude = row.y_km.atan2(row.x_km).to_degrees();
    let latitude = (row.z_km / radius_km).clamp(-1.0, 1.0).asin().to_degrees();
    (longitude, latitude)
}

#[test]
fn the_default_chain_serves_each_truth_epoch_within_tolerance_or_refuses_it() {
    let backend = default_chart_backend();
    let bodies = [
        CelestialBody::Ceres,
        CelestialBody::Pallas,
        CelestialBody::Juno,
        CelestialBody::Vesta,
        CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros")),
    ];
    for body in bodies {
        let mut served = Vec::new();
        let mut refused = 0_usize;
        for row in pleiades_jpl::asteroid_reference_corpus()
            .iter()
            .filter(|row| row.body == body)
        {
            let jd = row.epoch.julian_day.days();
            let request = EphemerisRequest {
                body: body.clone(),
                instant: row.epoch,
                observer: None,
                frame: CoordinateFrame::Ecliptic,
                zodiac_mode: ZodiacMode::Tropical,
                apparent: Apparentness::Mean,
            };
            match backend.position(&request) {
                Ok(result) => {
                    let got = result
                        .ecliptic
                        .expect("a served row has ecliptic coordinates");
                    let (truth_longitude, truth_latitude) = truth_longitude_latitude(row);
                    let longitude = ((got.longitude.degrees() - truth_longitude + 180.0)
                        .rem_euclid(360.0)
                        - 180.0)
                        .abs()
                        * truth_latitude.to_radians().cos()
                        * 3600.0;
                    let latitude = (got.latitude.degrees() - truth_latitude).abs() * 3600.0;
                    assert!(
                        longitude <= TRUTH_TOLERANCE_ARCSEC && latitude <= TRUTH_TOLERANCE_ARCSEC,
                        "{body} at JD {jd}: served {longitude:.3}″ / {latitude:.3}″ from truth"
                    );
                    assert_eq!(
                        result.backend_id.as_str(),
                        "jpl-snapshot",
                        "{body} at JD {jd}"
                    );
                    served.push(jd);
                }
                Err(_) => refused += 1,
            }
        }
        assert_eq!(served, vec![SHARED_EPOCH_JD], "{body}: served epochs");
        assert_eq!(refused, 406, "{body}: refused epochs");
    }
}

#[test]
fn stations_of_an_asteroid_report_the_refusal() {
    let error = crate::commands::events::render_stations(&[
        "--body",
        "Ceres",
        "--from",
        "2451545.0",
        "--to",
        "2451645.0",
    ])
    .expect_err("Ceres has no rows across this range");
    assert!(error.contains("SpkBackend"), "{error}");
}
