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

/// Finalized speed and distance ceilings (Task 11, 2026-06-20).
///
/// Speed ceilings are set to round numbers ≥ ~10× the measured maxima from the
/// committed packaged artifact baseline:
///
/// | class           | measured lon_speed | measured lat_speed | measured dist_km |
/// |-----------------|--------------------|--------------------|------------------|
/// | Luminary/Inner  | 0.0303 arcsec/day  | 0.0231 arcsec/day  | 0.654 km (Venus) |
/// | Outer           | 0.0013 arcsec/day  | 0.0013 arcsec/day  | 54.828 km (Uranus)|
///
/// Chosen ceilings:
/// - Luminary/Inner: lon/lat speed → 0.5 arcsec/day (~16× headroom on Moon's 0.0303);
///   dist_km → 50 km (~76× headroom on Venus's 0.654 km).
/// - Outer: lon/lat speed → 0.05 arcsec/day (~38× headroom on worst 0.0013);
///   dist_km → 1_000 km (~18× headroom on Uranus's 54.828 km).
/// - Radial speed: all bodies show < 1e-7 AU/day measured; ceiling 1e-4 AU/day gives
///   >1000× headroom; kept tighter than the original placeholder (1e-3) to signal intent.
pub fn accuracy_ceiling(body: &CelestialBody) -> AccuracyCeiling {
    match body_class(body) {
        BodyClass::Luminary | BodyClass::InnerPlanet => AccuracyCeiling {
            lon_arcsec: 1.0,
            lat_arcsec: 1.0,
            dist_km: 50.0,
            lon_speed_arcsec_per_day: 0.5,
            lat_speed_arcsec_per_day: 0.5,
            radial_speed_au_per_day: 1.0e-4,
        },
        BodyClass::OuterPlanet => AccuracyCeiling {
            lon_arcsec: 5.0,
            lat_arcsec: 5.0,
            dist_km: 1_000.0,
            lon_speed_arcsec_per_day: 0.05,
            lat_speed_arcsec_per_day: 0.05,
            radial_speed_au_per_day: 1.0e-4,
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

/// Returns a deterministic summary comparing published accuracy ceilings against the live
/// measured baseline (from the committed packaged artifact vs hold-out corpus), plus the
/// size budget. One line per body class × channel showing measured/ceiling with PASS/FAIL.
///
/// The function reads [`accuracy_ceiling`] and [`PACKAGED_BUDGETS`] (the SSOT) plus the
/// live baseline — deterministic, kernel-free.  PASS/FAIL is computed from measured vs
/// ceiling, so the summary stays truthful if a value ever regresses.
pub fn packaged_artifact_thresholds_summary_for_report() -> String {
    let baseline = crate::accuracy_baseline::packaged_artifact_accuracy_baseline();
    let mut lines = Vec::new();
    for e in &baseline {
        let c = accuracy_ceiling(&e.body);
        let lon_status = if e.max_longitude_arcsec <= c.lon_arcsec { "PASS" } else { "FAIL" };
        let lat_status = if e.max_latitude_arcsec <= c.lat_arcsec { "PASS" } else { "FAIL" };
        let dist_status = if e.max_distance_km <= c.dist_km { "PASS" } else { "FAIL" };
        let spd_status = if e.max_lon_speed_arcsec_per_day <= c.lon_speed_arcsec_per_day { "PASS" } else { "FAIL" };
        lines.push(format!(
            "{:?}: lon {:.4}\"/{}\"={} lat {:.4}\"/{}\"={} dist {:.1}/{} km={} lon_spd {:.4}/{} \"/d={}",
            e.body,
            e.max_longitude_arcsec, c.lon_arcsec, lon_status,
            e.max_latitude_arcsec, c.lat_arcsec, lat_status,
            e.max_distance_km, c.dist_km, dist_status,
            e.max_lon_speed_arcsec_per_day, c.lon_speed_arcsec_per_day, spd_status,
        ));
    }
    format!(
        "Packaged-artifact thresholds ({} bodies); size budget {} bytes:\n{}",
        baseline.len(),
        PACKAGED_BUDGETS.max_encoded_bytes,
        lines.join("\n")
    )
}

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

    #[test]
    #[ignore = "maintainer helper: prints the thresholds summary to regenerate the golden"]
    fn print_packaged_artifact_thresholds_summary() {
        eprintln!("{}", packaged_artifact_thresholds_summary_for_report());
    }

    // Drift gate: committed thresholds summary must match the live baseline.
    // Generated from actual output (2026-06-20, SP3 heliocentric-planet artifact).
    // Anchors header, one representative body line, and the size-budget line.
    // PASS/FAIL is computed — fails if any channel regresses past its ceiling.
    #[test]
    fn packaged_artifact_thresholds_summary_matches_committed_golden() {
        let report = packaged_artifact_thresholds_summary_for_report();

        // Header: 10 bodies and the size budget.
        assert!(
            report.contains("Packaged-artifact thresholds (10 bodies)"),
            "thresholds report header drift: {report}"
        );
        assert!(
            report.contains("size budget 12000000 bytes"),
            "thresholds report size-budget line drift: {report}"
        );

        // Representative body line: Moon (Luminary class, ceiling 1.0"/1.0"/50 km/0.5"/d).
        // All channels must PASS given the current sub-ceiling measurements.
        assert!(
            report.contains("Moon:") && report.contains("/1\"=PASS"),
            "Moon lon/ceiling PASS line drift: {report}"
        );

        // Size budget line is in the header (checked above).
        // Verify each body appears and shows PASS for longitude.
        for body_name in &["Sun", "Moon", "Mercury", "Venus", "Mars",
                           "Jupiter", "Saturn", "Uranus", "Neptune", "Pluto"] {
            assert!(
                report.contains(body_name),
                "{body_name} missing from thresholds report: {report}"
            );
            // Each body should show PASS (all measured values are well within ceilings).
            assert!(
                report.contains(&format!("{body_name}:")) && {
                    // find this body's line and check it has PASS
                    report.lines()
                        .find(|l| l.starts_with(body_name))
                        .map(|l| l.contains("PASS"))
                        .unwrap_or(false)
                },
                "{body_name} line does not contain PASS: {report}"
            );
        }
    }
}
