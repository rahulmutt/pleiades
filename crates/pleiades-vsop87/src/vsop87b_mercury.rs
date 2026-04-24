//! VSOP87B Mercury coefficient tables backed by the full public IMCCE/CELMECH
//! source file.
//!
//! Mercury now uses a generated binary table derived from the vendored public
//! IMCCE/CELMECH VSOP87B source file, so the backend keeps a reproducible
//! coefficient artifact for the innermost planet without relying on a
//! hand-trimmed coefficient slice.

use std::sync::OnceLock;

use crate::vsop87b_earth::{
    evaluate, parse_generated_vsop87b_tables, SphericalLbr, Vsop87SeriesTables,
};

static MERCURY_TABLES: OnceLock<Vsop87SeriesTables> = OnceLock::new();

pub(crate) fn mercury_lbr(julian_day_tt: f64) -> SphericalLbr {
    let t = (julian_day_tt - 2_451_545.0) / 365_250.0;
    let tables = mercury_tables();
    SphericalLbr {
        longitude_rad: evaluate(tables.longitude.iter().map(Vec::as_slice), t)
            .rem_euclid(core::f64::consts::TAU),
        latitude_rad: evaluate(tables.latitude.iter().map(Vec::as_slice), t),
        radius_au: evaluate(tables.radius.iter().map(Vec::as_slice), t),
    }
}

fn mercury_tables() -> &'static Vsop87SeriesTables {
    MERCURY_TABLES
        .get_or_init(|| parse_generated_vsop87b_tables(include_bytes!("../data/VSOP87B.mer.bin")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vsop87b_earth::parse_vsop87b_tables;

    #[test]
    fn parses_full_mercury_tables_with_expected_series_counts() {
        let tables = parse_vsop87b_tables(include_str!("../data/VSOP87B.mer"));
        assert_eq!(tables.longitude.len(), 6);
        assert_eq!(tables.latitude.len(), 6);
        assert_eq!(tables.radius.len(), 6);

        let longitude_terms: Vec<usize> = tables.longitude.iter().map(Vec::len).collect();
        let latitude_terms: Vec<usize> = tables.latitude.iter().map(Vec::len).collect();
        let radius_terms: Vec<usize> = tables.radius.iter().map(Vec::len).collect();

        assert_eq!(longitude_terms, vec![1583, 931, 438, 162, 23, 12]);
        assert_eq!(latitude_terms, vec![818, 492, 231, 39, 13, 10]);
        assert_eq!(radius_terms, vec![1209, 706, 318, 111, 17, 10]);
    }

    #[test]
    fn parses_generated_mercury_table_blob_with_expected_series_counts() {
        let tables = parse_generated_vsop87b_tables(include_bytes!("../data/VSOP87B.mer.bin"));
        assert_eq!(tables.longitude.len(), 6);
        assert_eq!(tables.latitude.len(), 6);
        assert_eq!(tables.radius.len(), 6);

        let longitude_terms: Vec<usize> = tables.longitude.iter().map(Vec::len).collect();
        let latitude_terms: Vec<usize> = tables.latitude.iter().map(Vec::len).collect();
        let radius_terms: Vec<usize> = tables.radius.iter().map(Vec::len).collect();

        assert_eq!(longitude_terms, vec![1583, 931, 438, 162, 23, 12]);
        assert_eq!(latitude_terms, vec![818, 492, 231, 39, 13, 10]);
        assert_eq!(radius_terms, vec![1209, 706, 318, 111, 17, 10]);
    }

    #[test]
    fn evaluates_j2000_mercury_coordinates_from_the_full_source_file() {
        let mercury = mercury_lbr(2_451_545.0);
        assert!((mercury.longitude_rad.to_degrees() - 253.782_952_369_111_57).abs() < 1e-12);
        assert!((mercury.latitude_rad.to_degrees() + 3.022_772_980_727_858).abs() < 1e-12);
        assert!((mercury.radius_au - 0.466_471_475_117_196_9).abs() < 1e-15);
    }
}
