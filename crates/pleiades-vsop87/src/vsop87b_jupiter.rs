//! VSOP87B Jupiter coefficient tables backed by the full public IMCCE/CELMECH
//! source file.
//!
//! Jupiter now uses a generated binary table derived from the vendored public
//! IMCCE/CELMECH VSOP87B source file, mirroring the Earth/Sun, Mercury, Venus,
//! and Mars paths so the backend keeps a source-backed path for every
//! supported VSOP87 major planet while Pluto remains outside the VSOP87
//! major-planet files.

use crate::vsop87b_earth::{
    evaluate, parse_generated_vsop87b_tables, SphericalLbr, Vsop87SeriesTables,
};
use std::sync::OnceLock;

static JUPITER_TABLES: OnceLock<Vsop87SeriesTables> = OnceLock::new();

pub(crate) fn jupiter_lbr(julian_day_tt: f64) -> SphericalLbr {
    let t = (julian_day_tt - 2_451_545.0) / 365_250.0;
    let tables = jupiter_tables();
    SphericalLbr {
        longitude_rad: evaluate(tables.longitude.iter().map(Vec::as_slice), t)
            .rem_euclid(core::f64::consts::TAU),
        latitude_rad: evaluate(tables.latitude.iter().map(Vec::as_slice), t),
        radius_au: evaluate(tables.radius.iter().map(Vec::as_slice), t),
    }
}

fn jupiter_tables() -> &'static Vsop87SeriesTables {
    JUPITER_TABLES
        .get_or_init(|| parse_generated_vsop87b_tables(include_bytes!("../data/VSOP87B.jup.bin")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_generated_jupiter_table_blob_with_expected_series_counts() {
        let tables = parse_generated_vsop87b_tables(include_bytes!("../data/VSOP87B.jup.bin"));
        assert_eq!(tables.longitude.len(), 6);
        assert_eq!(tables.latitude.len(), 6);
        assert_eq!(tables.radius.len(), 6);

        let longitude_terms: Vec<usize> = tables.longitude.iter().map(Vec::len).collect();
        let latitude_terms: Vec<usize> = tables.latitude.iter().map(Vec::len).collect();
        let radius_terms: Vec<usize> = tables.radius.iter().map(Vec::len).collect();

        assert_eq!(longitude_terms, vec![860, 426, 225, 120, 48, 11]);
        assert_eq!(latitude_terms, vec![249, 120, 82, 33, 13, 3]);
        assert_eq!(radius_terms, vec![727, 371, 186, 97, 45, 9]);
    }

    #[test]
    fn evaluates_j2000_jupiter_coordinates_from_the_full_source_file() {
        let jupiter = jupiter_lbr(2_451_545.0);
        assert!((jupiter.longitude_rad.to_degrees() - 36.294_665_945_668_87).abs() < 1e-12);
        assert!((jupiter.latitude_rad.to_degrees() + 1.174_569_431_512_513_5).abs() < 1e-12);
        assert!((jupiter.radius_au - 4.965_381_280_273_759).abs() < 1e-12);
    }
}
