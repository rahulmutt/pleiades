//! SP-6-FU KNOWN GAP 3 differential harness. Ignored by default (it is a
//! diagnosis surface, not a gate); run with:
//!   cargo test -p pleiades-validate --test occult_graze_diagnosis -- --ignored --nocapture

use pleiades_apparent::fnv1a64;
use pleiades_backend::{CompositeBackend, RoutingBackend};
use pleiades_data::PackagedDataBackend;
use pleiades_elp::ElpBackend;
use pleiades_events::{EventEngine, OccultTarget, OccultationType};
use pleiades_fict::FictitiousBackend;
use pleiades_jpl::JplSnapshotBackend;
use pleiades_types::{CelestialBody, Latitude, Longitude, ObserverLocation};
use pleiades_vsop87::Vsop87Backend;

const FIXTURE: &str = include_str!("../data/occultations-corpus/graze-diagnosis.csv");
/// Pinned at generation time (Task 4 Step 3 stderr output). Drift fails closed.
const FIXTURE_CHECKSUM: u64 = 5834603876026131102;

fn great_circle_deg(ra1: f64, dec1: f64, ra2: f64, dec2: f64) -> f64 {
    let (a1, d1) = (ra1.to_radians(), dec1.to_radians());
    let (a2, d2) = (ra2.to_radians(), dec2.to_radians());
    (d1.sin() * d2.sin() + d1.cos() * d2.cos() * (a1 - a2).cos())
        .clamp(-1.0, 1.0)
        .acos()
        .to_degrees()
}

fn body_from_se(se: i64) -> Option<CelestialBody> {
    // Mirrors occult_validation::body_from_se for the three corpus planets.
    match se {
        3 => Some(CelestialBody::Venus),
        5 => Some(CelestialBody::Jupiter),
        6 => Some(CelestialBody::Saturn),
        _ => None,
    }
}

#[test]
#[ignore = "diagnosis harness, not a gate — run with --ignored --nocapture"]
fn graze_boundary_differential_table() {
    assert_eq!(
        fnv1a64(FIXTURE),
        FIXTURE_CHECKSUM,
        "graze-diagnosis.csv drifted; regenerate via the tool's --diagnosis mode \
         and re-pin (actual checksum: {})",
        fnv1a64(FIXTURE)
    );
    let backend = RoutingBackend::new(vec![
        Box::new(PackagedDataBackend::new()),
        Box::new(CompositeBackend::new(
            Vsop87Backend::new(),
            ElpBackend::new(),
        )),
        Box::new(JplSnapshotBackend::new()),
        Box::new(FictitiousBackend::new(PackagedDataBackend::new())),
    ]);
    let engine = EventEngine::new(backend);

    eprintln!("row                    dt_diff_s  geoMoon\" topoMoon\"  topoTgt\"  sdMoon\" ourMargin' seMargin'    type flag");
    let mut rows = 0usize;
    let mut disagreements = 0usize;
    for line in FIXTURE.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') || line.starts_with("label,") {
            continue;
        }
        let f: Vec<&str> = line.split(',').collect();
        assert_eq!(f.len(), 20, "fixture schema drift: {line}");
        let p = |i: usize| -> f64 { f[i].parse().unwrap_or_else(|_| panic!("col {i}: {line}")) };
        let se_body: i64 = f[1].parse().expect("se_body");
        let star = f[2].trim();
        let target = if se_body == -1 {
            OccultTarget::Star(star.to_string())
        } else {
            OccultTarget::Body(body_from_se(se_body).expect("corpus planet"))
        };
        let (anchor, lat, lon) = (p(3), p(4), p(5));
        let obs = ObserverLocation::new(
            Latitude::from_degrees(lat),
            Longitude::from_degrees(lon),
            Some(0.0),
        );
        let d = engine
            .occult_stage_diagnostics(&target, &obs, anchor)
            .unwrap_or_else(|e| panic!("{}: {e}", f[0]));

        let dt_diff = d.delta_t_seconds - p(6);
        let geo_moon = great_circle_deg(d.moon_geo.0, d.moon_geo.1, p(7), p(8)) * 3600.0;
        let topo_moon = great_circle_deg(d.moon_topo.0, d.moon_topo.1, p(10), p(11)) * 3600.0;
        let topo_tgt = great_circle_deg(d.target_topo.0, d.target_topo.1, p(16), p(17)) * 3600.0;
        let sd_moon = (d.s_moon_deg - p(19)) * 3600.0;
        // SE-side margin from SE's own topocentric numbers (star s_tgt = 0;
        // planet s_tgt from our radius table at SE's topo distance — sub-0.1"
        // either way at these distances).
        let se_sep = great_circle_deg(p(10), p(11), p(16), p(17));
        let se_margin = (se_sep - (p(19) + d.s_tgt_deg)) * 60.0;
        let our_margin = d.refined_margin_deg * 60.0;
        let disagree = d.occultation_type != OccultationType::Miss;
        if disagree {
            disagreements += 1;
        }
        for (name, v) in [
            ("dt_diff", dt_diff),
            ("geo_moon", geo_moon),
            ("topo_moon", topo_moon),
            ("topo_tgt", topo_tgt),
            ("sd_moon", sd_moon),
            ("our_margin", our_margin),
            ("se_margin", se_margin),
        ] {
            assert!(v.is_finite(), "{}: non-finite {name}", f[0]);
        }
        eprintln!(
            "{:<22} {:>8.3} {:>9.3} {:>9.3} {:>9.3} {:>8.3} {:>9.3} {:>9.3} {:>7?} {}",
            f[0],
            dt_diff,
            geo_moon,
            topo_moon,
            topo_tgt,
            sd_moon,
            our_margin,
            se_margin,
            d.occultation_type,
            match (disagree, d.delta_t_predicted) {
                (true, true) => "<-- DISAGREE (SE: Miss) [dT predicted]",
                (true, false) => "<-- DISAGREE (SE: Miss)",
                (false, true) => "[dT predicted]",
                (false, false) => "",
            }
        );
        rows += 1;
    }
    assert_eq!(
        rows, 18,
        "fixture must cover all sibling-anchored miss rows"
    );
    eprintln!("disagreements: {disagreements}/18 (SP-6 measured 8: 3 knife-edge + 5 genuine)");
}
