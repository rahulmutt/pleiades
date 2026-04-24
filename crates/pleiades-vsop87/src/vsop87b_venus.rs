//! VSOP87B Venus coefficient tables backed by the full public IMCCE/CELMECH
//! source file.
//!
//! Venus now follows the same full-file parsing path as the Earth/Sun backend
//! path, so the backend can keep a source-backed path for the smallest
//! remaining body without relying on a hand-trimmed coefficient slice.

use std::sync::OnceLock;

use crate::vsop87b_earth::{evaluate, parse_vsop87b_tables, SphericalLbr, Vsop87SeriesTables};

static VENUS_TABLES: OnceLock<Vsop87SeriesTables> = OnceLock::new();

pub(crate) fn venus_lbr(julian_day_tt: f64) -> SphericalLbr {
    let t = (julian_day_tt - 2_451_545.0) / 365_250.0;
    let tables = venus_tables();
    SphericalLbr {
        longitude_rad: evaluate(tables.longitude.iter().map(Vec::as_slice), t)
            .rem_euclid(core::f64::consts::TAU),
        latitude_rad: evaluate(tables.latitude.iter().map(Vec::as_slice), t),
        radius_au: evaluate(tables.radius.iter().map(Vec::as_slice), t),
    }
}

fn venus_tables() -> &'static Vsop87SeriesTables {
    VENUS_TABLES.get_or_init(|| parse_vsop87b_tables(include_str!("../data/VSOP87B.ven")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_full_venus_tables_with_expected_series_counts() {
        let tables = parse_vsop87b_tables(include_str!("../data/VSOP87B.ven"));
        assert_eq!(tables.longitude.len(), 6);
        assert_eq!(tables.latitude.len(), 6);
        assert_eq!(tables.radius.len(), 6);

        let longitude_terms: Vec<usize> = tables.longitude.iter().map(Vec::len).collect();
        let latitude_terms: Vec<usize> = tables.latitude.iter().map(Vec::len).collect();
        let radius_terms: Vec<usize> = tables.radius.iter().map(Vec::len).collect();

        assert_eq!(longitude_terms, vec![416, 235, 72, 7, 4, 2]);
        assert_eq!(latitude_terms, vec![210, 121, 51, 12, 4, 4]);
        assert_eq!(radius_terms, vec![323, 174, 62, 8, 3, 2]);
    }

    #[test]
    fn evaluates_j2000_venus_coordinates_from_the_full_source_file() {
        let venus = venus_lbr(2_451_545.0);
        assert!((venus.longitude_rad.to_degrees() - 182.602_920_758_433_04).abs() < 1e-12);
        assert!((venus.latitude_rad.to_degrees() - 3.264_615_250_767_216).abs() < 1e-12);
        assert!((venus.radius_au - 0.720_212_924_846_730_5).abs() < 1e-15);
    }
}
