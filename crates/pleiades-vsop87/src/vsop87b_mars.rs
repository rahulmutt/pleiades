//! VSOP87B Mars coefficient tables backed by the full public IMCCE/CELMECH
//! source file.
//!
//! Mars now follows the same full-file parsing path as the Sun, Mercury, and
//! Venus backends, so the backend keeps a source-backed path for the outer
//! inner planet without relying on a hand-trimmed coefficient slice.

use std::sync::OnceLock;

use crate::vsop87b_earth::{evaluate, parse_vsop87b_tables, SphericalLbr, Vsop87SeriesTables};

static MARS_TABLES: OnceLock<Vsop87SeriesTables> = OnceLock::new();

pub(crate) fn mars_lbr(julian_day_tt: f64) -> SphericalLbr {
    let t = (julian_day_tt - 2_451_545.0) / 365_250.0;
    let tables = mars_tables();
    SphericalLbr {
        longitude_rad: evaluate(tables.longitude.iter().map(Vec::as_slice), t)
            .rem_euclid(core::f64::consts::TAU),
        latitude_rad: evaluate(tables.latitude.iter().map(Vec::as_slice), t),
        radius_au: evaluate(tables.radius.iter().map(Vec::as_slice), t),
    }
}

fn mars_tables() -> &'static Vsop87SeriesTables {
    MARS_TABLES.get_or_init(|| parse_vsop87b_tables(include_str!("../data/VSOP87B.mar")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_full_mars_tables_with_expected_series_counts() {
        let tables = parse_vsop87b_tables(include_str!("../data/VSOP87B.mar"));
        assert_eq!(tables.longitude.len(), 6);
        assert_eq!(tables.latitude.len(), 6);
        assert_eq!(tables.radius.len(), 6);

        let longitude_terms: Vec<usize> = tables.longitude.iter().map(Vec::len).collect();
        let latitude_terms: Vec<usize> = tables.latitude.iter().map(Vec::len).collect();
        let radius_terms: Vec<usize> = tables.radius.iter().map(Vec::len).collect();

        assert_eq!(longitude_terms, vec![1409, 891, 442, 194, 75, 24]);
        assert_eq!(latitude_terms, vec![441, 291, 161, 64, 18, 9]);
        assert_eq!(radius_terms, vec![1107, 672, 368, 160, 57, 17]);
    }

    #[test]
    fn evaluates_j2000_mars_coordinates_from_the_full_source_file() {
        let mars = mars_lbr(2_451_545.0);
        assert!((mars.longitude_rad.to_degrees() - 359.447_306_577_956_9).abs() < 1e-12);
        assert!((mars.latitude_rad.to_degrees() + 1.419_673_814_332_492_7).abs() < 1e-12);
        assert!((mars.radius_au - 1.391_207_693_715_968).abs() < 1e-12);
    }
}
