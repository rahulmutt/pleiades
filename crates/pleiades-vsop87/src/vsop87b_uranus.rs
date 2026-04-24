//! VSOP87B Uranus coefficient tables backed by the full public IMCCE/CELMECH
//! source file.
//!
//! Uranus now follows the same full-file parsing path as the Earth/Sun,
//! Mercury, Venus, Mars, Jupiter, and Saturn backends, so the backend keeps a
//! source-backed path for every supported VSOP87 major planet while the Pluto
//! special case remains outside the VSOP87 major-planet files.

use crate::vsop87b_earth::{evaluate, parse_vsop87b_tables, SphericalLbr, Vsop87SeriesTables};
use std::sync::OnceLock;

static URANUS_TABLES: OnceLock<Vsop87SeriesTables> = OnceLock::new();

pub(crate) fn uranus_lbr(julian_day_tt: f64) -> SphericalLbr {
    let t = (julian_day_tt - 2_451_545.0) / 365_250.0;
    let tables = uranus_tables();
    SphericalLbr {
        longitude_rad: evaluate(tables.longitude.iter().map(Vec::as_slice), t)
            .rem_euclid(core::f64::consts::TAU),
        latitude_rad: evaluate(tables.latitude.iter().map(Vec::as_slice), t),
        radius_au: evaluate(tables.radius.iter().map(Vec::as_slice), t),
    }
}

fn uranus_tables() -> &'static Vsop87SeriesTables {
    URANUS_TABLES.get_or_init(|| parse_vsop87b_tables(include_str!("../data/VSOP87B.ura")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_full_uranus_tables_with_expected_series_counts() {
        let tables = parse_vsop87b_tables(include_str!("../data/VSOP87B.ura"));
        assert_eq!(tables.longitude.len(), 6);
        assert_eq!(tables.latitude.len(), 6);
        assert_eq!(tables.radius.len(), 6);

        let longitude_terms: Vec<usize> = tables.longitude.iter().map(Vec::len).collect();
        let latitude_terms: Vec<usize> = tables.latitude.iter().map(Vec::len).collect();
        let radius_terms: Vec<usize> = tables.radius.iter().map(Vec::len).collect();

        assert_eq!(longitude_terms, vec![1441, 655, 259, 69, 8, 0]);
        assert_eq!(latitude_terms, vec![311, 130, 39, 15, 0, 0]);
        assert_eq!(radius_terms, vec![1387, 625, 249, 69, 12, 0]);
    }

    #[test]
    fn evaluates_j2000_uranus_coordinates_from_the_full_source_file() {
        let uranus = uranus_lbr(2_451_545.0);
        assert!((uranus.longitude_rad.to_degrees() - 316.418_722_907_632_7).abs() < 1e-12);
        assert!((uranus.latitude_rad.to_degrees() + 0.684_844_296_148_084_3).abs() < 1e-12);
        assert!((uranus.radius_au - 19.924_047_895_208_606).abs() < 1e-12);
    }
}
