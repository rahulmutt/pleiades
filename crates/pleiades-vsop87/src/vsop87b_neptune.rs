//! VSOP87B Neptune coefficient tables backed by the full public IMCCE/CELMECH
//! source file.
//!
//! Neptune now follows the same full-file parsing path as the Earth/Sun,
//! Mercury, Venus, Mars, Jupiter, Saturn, and Uranus backends, so the backend
//! keeps a source-backed path for every supported VSOP87 major planet while the
//! Pluto special case remains outside the VSOP87 major-planet files.

use crate::vsop87b_earth::{evaluate, parse_vsop87b_tables, SphericalLbr, Vsop87SeriesTables};
use std::sync::OnceLock;

static NEPTUNE_TABLES: OnceLock<Vsop87SeriesTables> = OnceLock::new();

pub(crate) fn neptune_lbr(julian_day_tt: f64) -> SphericalLbr {
    let t = (julian_day_tt - 2_451_545.0) / 365_250.0;
    let tables = neptune_tables();
    SphericalLbr {
        longitude_rad: evaluate(tables.longitude.iter().map(Vec::as_slice), t)
            .rem_euclid(core::f64::consts::TAU),
        latitude_rad: evaluate(tables.latitude.iter().map(Vec::as_slice), t),
        radius_au: evaluate(tables.radius.iter().map(Vec::as_slice), t),
    }
}

fn neptune_tables() -> &'static Vsop87SeriesTables {
    NEPTUNE_TABLES.get_or_init(|| parse_vsop87b_tables(include_str!("../data/VSOP87B.nep")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_full_neptune_tables_with_expected_series_counts() {
        let tables = parse_vsop87b_tables(include_str!("../data/VSOP87B.nep"));
        assert_eq!(tables.longitude.len(), 6);
        assert_eq!(tables.latitude.len(), 6);
        assert_eq!(tables.radius.len(), 6);

        let longitude_terms: Vec<usize> = tables.longitude.iter().map(Vec::len).collect();
        let latitude_terms: Vec<usize> = tables.latitude.iter().map(Vec::len).collect();
        let radius_terms: Vec<usize> = tables.radius.iter().map(Vec::len).collect();

        assert_eq!(longitude_terms, vec![539, 224, 59, 18, 0, 0]);
        assert_eq!(latitude_terms, vec![172, 49, 13, 2, 0, 0]);
        assert_eq!(radius_terms, vec![596, 251, 71, 23, 7, 0]);
    }

    #[test]
    fn evaluates_j2000_neptune_coordinates_from_the_full_source_file() {
        let neptune = neptune_lbr(2_451_545.0);
        assert!((neptune.longitude_rad.to_degrees() - 303.929_067_957_673_9).abs() < 1e-12);
        assert!((neptune.latitude_rad.to_degrees() - 0.241_998_979_517_422_16).abs() < 1e-12);
        assert!((neptune.radius_au - 30.120_532_933_188_983).abs() < 1e-12);
    }
}
