//! VSOP87B Saturn coefficient tables backed by the full public IMCCE/CELMECH
//! source file.
//!
//! Saturn now uses a generated binary table derived from the vendored public
//! IMCCE/CELMECH VSOP87B source file, mirroring the Earth/Sun, Mercury, Venus,
//! Mars, and Jupiter paths so the backend keeps a source-backed path for every
//! supported VSOP87 major planet while Pluto remains outside the VSOP87
//! major-planet files.

use crate::vsop87b_earth::{
    evaluate, parse_generated_vsop87b_tables, SphericalLbr, Vsop87SeriesTables,
};
use std::sync::OnceLock;

static SATURN_TABLES: OnceLock<Vsop87SeriesTables> = OnceLock::new();

pub(crate) fn saturn_lbr(julian_day_tt: f64) -> SphericalLbr {
    let t = (julian_day_tt - 2_451_545.0) / 365_250.0;
    let tables = saturn_tables();
    SphericalLbr {
        longitude_rad: evaluate(tables.longitude.iter().map(Vec::as_slice), t)
            .rem_euclid(core::f64::consts::TAU),
        latitude_rad: evaluate(tables.latitude.iter().map(Vec::as_slice), t),
        radius_au: evaluate(tables.radius.iter().map(Vec::as_slice), t),
    }
}

fn saturn_tables() -> &'static Vsop87SeriesTables {
    SATURN_TABLES
        .get_or_init(|| parse_generated_vsop87b_tables(include_bytes!("../data/VSOP87B.sat.bin")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_generated_saturn_table_blob_with_expected_series_counts() {
        let tables = parse_generated_vsop87b_tables(include_bytes!("../data/VSOP87B.sat.bin"));
        assert_eq!(tables.longitude.len(), 6);
        assert_eq!(tables.latitude.len(), 6);
        assert_eq!(tables.radius.len(), 6);

        let longitude_terms: Vec<usize> = tables.longitude.iter().map(Vec::len).collect();
        let latitude_terms: Vec<usize> = tables.latitude.iter().map(Vec::len).collect();
        let radius_terms: Vec<usize> = tables.radius.iter().map(Vec::len).collect();

        assert_eq!(longitude_terms, vec![1437, 817, 438, 192, 85, 30]);
        assert_eq!(latitude_terms, vec![500, 247, 111, 54, 24, 11]);
        assert_eq!(radius_terms, vec![1208, 627, 338, 154, 65, 27]);
    }

    #[test]
    fn evaluates_j2000_saturn_coordinates_from_the_full_source_file() {
        let saturn = saturn_lbr(2_451_545.0);
        assert!((saturn.longitude_rad.to_degrees() - 45.722_254_745_568_414).abs() < 1e-12);
        assert!((saturn.latitude_rad.to_degrees() + 2.303_199_518_162_094).abs() < 1e-12);
        assert!((saturn.radius_au - 9.183_848_288_052_635).abs() < 1e-12);
    }
}
