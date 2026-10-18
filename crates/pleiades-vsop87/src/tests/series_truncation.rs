use crate::tables::series::{
    truncate_tables, SeriesFidelity, TRUNCATION_THRESHOLD, TRUNCATION_T_MAX_MILLENNIA,
};
use crate::tables::vsop87b_earth::{
    earth_lbr, evaluate, parse_generated_vsop87b_tables, SphericalLbr, Vsop87SeriesTables,
};
use crate::tables::vsop87b_jupiter::jupiter_lbr;
use crate::tables::vsop87b_mars::mars_lbr;
use crate::tables::vsop87b_mercury::mercury_lbr;
use crate::tables::vsop87b_neptune::neptune_lbr;
use crate::tables::vsop87b_saturn::saturn_lbr;
use crate::tables::vsop87b_uranus::uranus_lbr;
use crate::tables::vsop87b_venus::venus_lbr;

type LbrFn = fn(f64, SeriesFidelity) -> SphericalLbr;

struct Body {
    name: &'static str,
    bytes: &'static [u8],
    lbr: LbrFn,
    truncated_terms: usize,
}

fn bodies() -> [Body; 8] {
    [
        Body {
            name: "Earth",
            bytes: include_bytes!("../../data/VSOP87B.ear.bin"),
            lbr: earth_lbr,
            truncated_terms: 578,
        },
        Body {
            name: "Mercury",
            bytes: include_bytes!("../../data/VSOP87B.mer.bin"),
            lbr: mercury_lbr,
            truncated_terms: 672,
        },
        Body {
            name: "Venus",
            bytes: include_bytes!("../../data/VSOP87B.ven.bin"),
            lbr: venus_lbr,
            truncated_terms: 419,
        },
        Body {
            name: "Mars",
            bytes: include_bytes!("../../data/VSOP87B.mar.bin"),
            lbr: mars_lbr,
            truncated_terms: 1390,
        },
        Body {
            name: "Jupiter",
            bytes: include_bytes!("../../data/VSOP87B.jup.bin"),
            lbr: jupiter_lbr,
            truncated_terms: 1654,
        },
        Body {
            name: "Saturn",
            bytes: include_bytes!("../../data/VSOP87B.sat.bin"),
            lbr: saturn_lbr,
            truncated_terms: 3115,
        },
        Body {
            name: "Uranus",
            bytes: include_bytes!("../../data/VSOP87B.ura.bin"),
            lbr: uranus_lbr,
            truncated_terms: 3159,
        },
        Body {
            name: "Neptune",
            bytes: include_bytes!("../../data/VSOP87B.nep.bin"),
            lbr: neptune_lbr,
            truncated_terms: 1278,
        },
    ]
}

fn term_count(tables: &Vsop87SeriesTables) -> usize {
    [&tables.longitude, &tables.latitude, &tables.radius]
        .into_iter()
        .flatten()
        .map(Vec::len)
        .sum()
}

#[test]
fn truncated_tables_keep_the_expected_term_counts() {
    for body in bodies() {
        let full = parse_generated_vsop87b_tables(body.bytes);
        let truncated = truncate_tables(&full, TRUNCATION_THRESHOLD, TRUNCATION_T_MAX_MILLENNIA);
        assert_eq!(
            term_count(&truncated),
            body.truncated_terms,
            "{}",
            body.name
        );
    }
}

#[test]
fn truncate_tables_keeps_power_alignment() {
    for body in bodies() {
        let full = parse_generated_vsop87b_tables(body.bytes);
        let emptied = truncate_tables(&full, f64::INFINITY, TRUNCATION_T_MAX_MILLENNIA);
        assert_eq!(term_count(&emptied), 0, "{}", body.name);
        assert_eq!(
            emptied.longitude.len(),
            full.longitude.len(),
            "{}",
            body.name
        );
        assert_eq!(emptied.latitude.len(), full.latitude.len(), "{}", body.name);
        assert_eq!(emptied.radius.len(), full.radius.len(), "{}", body.name);
    }
}

#[test]
fn full_fidelity_is_bit_identical_to_the_generated_tables() {
    for body in bodies() {
        let tables = parse_generated_vsop87b_tables(body.bytes);
        for jd in [2_451_545.0, 2_305_447.5, 2_670_690.5] {
            let t = (jd - 2_451_545.0) / 365_250.0;
            let actual = (body.lbr)(jd, SeriesFidelity::Full);
            let longitude = evaluate(tables.longitude.iter().map(Vec::as_slice), t)
                .rem_euclid(core::f64::consts::TAU);
            let latitude = evaluate(tables.latitude.iter().map(Vec::as_slice), t);
            let radius = evaluate(tables.radius.iter().map(Vec::as_slice), t);
            assert!(
                actual.longitude_rad == longitude,
                "{} lon at {jd}",
                body.name
            );
            assert!(actual.latitude_rad == latitude, "{} lat at {jd}", body.name);
            assert!(actual.radius_au == radius, "{} rad at {jd}", body.name);
        }
    }
}

#[test]
fn truncated_fidelity_differs_but_stays_close_at_j2000() {
    let full = mars_lbr(2_451_545.0, SeriesFidelity::Full);
    let truncated = mars_lbr(2_451_545.0, SeriesFidelity::Truncated);
    assert!(
        full.longitude_rad != truncated.longitude_rad
            || full.latitude_rad != truncated.latitude_rad
            || full.radius_au != truncated.radius_au
    );
    let delta_arcsec = (full.longitude_rad - truncated.longitude_rad)
        .to_degrees()
        .abs()
        * 3600.0;
    assert!(delta_arcsec < 0.30, "longitude delta {delta_arcsec} arcsec");
}
