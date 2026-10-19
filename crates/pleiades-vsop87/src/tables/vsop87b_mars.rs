//! VSOP87B Mars coefficient tables backed by the full public IMCCE/CELMECH
//! source file.
//!
//! Mars now uses a generated binary table derived from the vendored public
//! IMCCE/CELMECH VSOP87B source file, so the backend keeps a reproducible
//! coefficient artifact for the outer inner planet without relying on a
//! hand-trimmed coefficient slice.

use super::series::{SeriesFidelity, Vsop87SeriesSource};
use super::vsop87b_earth::SphericalLbr;

static MARS: Vsop87SeriesSource =
    Vsop87SeriesSource::new(include_bytes!("../../data/VSOP87B.mar.bin"));

pub(crate) fn mars_lbr(julian_day_tt: f64, fidelity: SeriesFidelity) -> SphericalLbr {
    MARS.lbr(julian_day_tt, fidelity)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tables::vsop87b_earth::{parse_generated_vsop87b_tables, parse_vsop87b_tables};

    #[test]
    fn parses_full_mars_tables_with_expected_series_counts() {
        let tables = parse_vsop87b_tables(include_str!("../../data/VSOP87B.mar"))
            .expect("Mars source file should parse");
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
    fn parses_generated_mars_table_blob_with_expected_series_counts() {
        let tables = parse_generated_vsop87b_tables(include_bytes!("../../data/VSOP87B.mar.bin"));
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
    fn evaluates_j2000_mars_coordinates_from_the_generated_table_file() {
        let mars = mars_lbr(2_451_545.0, SeriesFidelity::Full);
        assert!((mars.longitude_rad.to_degrees() - 359.447_306_577_956_9).abs() < 1e-12);
        assert!((mars.latitude_rad.to_degrees() + 1.419_673_814_332_492_7).abs() < 1e-12);
        assert!((mars.radius_au - 1.391_207_693_715_968).abs() < 1e-12);
    }
}
