//! Published per-body-class accuracy ceilings and size/latency budgets (the
//! public contract). The hold-out gate (accuracy_baseline.rs) asserts measured
//! <= ceiling; the tight golden drift test stays as the regression catcher.

use pleiades_backend::CelestialBody;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BodyClass {
    Luminary,
    InnerPlanet,
    OuterPlanet,
    Asteroid,
}

pub fn body_class(body: &CelestialBody) -> BodyClass {
    match body {
        CelestialBody::Sun | CelestialBody::Moon => BodyClass::Luminary,
        CelestialBody::Mercury | CelestialBody::Venus | CelestialBody::Mars => {
            BodyClass::InnerPlanet
        }
        CelestialBody::Jupiter
        | CelestialBody::Saturn
        | CelestialBody::Uranus
        | CelestialBody::Neptune
        | CelestialBody::Pluto => BodyClass::OuterPlanet,
        _ => BodyClass::Asteroid,
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AccuracyCeiling {
    pub lon_arcsec: f64,
    pub lat_arcsec: f64,
    pub dist_km: f64,
    pub lon_speed_arcsec_per_day: f64,
    pub lat_speed_arcsec_per_day: f64,
    pub radial_speed_au_per_day: f64,
}

pub fn accuracy_ceiling(body: &CelestialBody) -> AccuracyCeiling {
    match body_class(body) {
        BodyClass::Luminary | BodyClass::InnerPlanet => AccuracyCeiling {
            lon_arcsec: 1.0,
            lat_arcsec: 1.0,
            dist_km: 50_000.0,
            lon_speed_arcsec_per_day: 60.0,
            lat_speed_arcsec_per_day: 60.0,
            radial_speed_au_per_day: 1.0e-3,
        },
        BodyClass::OuterPlanet => AccuracyCeiling {
            lon_arcsec: 5.0,
            lat_arcsec: 5.0,
            dist_km: 5_000_000.0,
            lon_speed_arcsec_per_day: 60.0,
            lat_speed_arcsec_per_day: 60.0,
            radial_speed_au_per_day: 1.0e-3,
        },
        BodyClass::Asteroid => AccuracyCeiling {
            lon_arcsec: 30.0,
            lat_arcsec: 30.0,
            dist_km: 5_000_000.0,
            lon_speed_arcsec_per_day: 120.0,
            lat_speed_arcsec_per_day: 120.0,
            radial_speed_au_per_day: 1.0e-2,
        },
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ArtifactBudgets {
    pub max_encoded_bytes: usize,
    pub decode_latency_target_ms: f64,
    pub single_lookup_target_ms: f64,
    pub batch_throughput_target_per_s: f64,
    pub chart_workload_target_ms: f64,
}

pub const PACKAGED_BUDGETS: ArtifactBudgets = ArtifactBudgets {
    max_encoded_bytes: 12_000_000,    // ~10.0 MB measured + headroom
    decode_latency_target_ms: 400.0,  // ~260 ms measured
    single_lookup_target_ms: 6.0,     // ~3.3 ms measured
    batch_throughput_target_per_s: 1_000.0,
    chart_workload_target_ms: 50.0,
};

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_backend::CelestialBody;

    #[test]
    fn classes_map_bodies_correctly() {
        assert_eq!(body_class(&CelestialBody::Sun), BodyClass::Luminary);
        assert_eq!(body_class(&CelestialBody::Moon), BodyClass::Luminary);
        assert_eq!(body_class(&CelestialBody::Mercury), BodyClass::InnerPlanet);
        assert_eq!(body_class(&CelestialBody::Pluto), BodyClass::OuterPlanet);
    }

    #[test]
    fn outer_planets_have_looser_longitude_ceiling_than_inner() {
        assert!(
            accuracy_ceiling(&CelestialBody::Uranus).lon_arcsec
                > accuracy_ceiling(&CelestialBody::Mercury).lon_arcsec
        );
    }

    #[test]
    fn size_budget_exceeds_current_artifact() {
        // current ~10 MB; budget has headroom but is finite.
        assert!(PACKAGED_BUDGETS.max_encoded_bytes >= 10_000_000);
        assert!(PACKAGED_BUDGETS.max_encoded_bytes <= 16_000_000);
    }
}
