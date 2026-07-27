//! Unit tests for the per-formula-family gate ceilings. Relocated from the
//! inline `#[cfg(test)] mod tests` in `thresholds.rs` per AGENTS.md.

use super::*;

#[test]
fn space_division_is_tighter_than_quadrant() {
    let equal = house_family_ceiling(HouseFormulaFamily::Equal);
    let quad = house_family_ceiling(HouseFormulaFamily::Quadrant);
    assert!(equal.cusp_arcsec <= quad.cusp_arcsec);
    assert!(equal.cusp_arcsec > 0.0);
}

#[test]
fn every_family_has_finite_positive_ceilings() {
    for family in [
        HouseFormulaFamily::Equal,
        HouseFormulaFamily::WholeSign,
        HouseFormulaFamily::Quadrant,
        HouseFormulaFamily::EquatorialProjection,
        HouseFormulaFamily::GreatCircle,
        HouseFormulaFamily::SolarArc,
        HouseFormulaFamily::Sector,
        HouseFormulaFamily::Custom,
        HouseFormulaFamily::Unknown,
    ] {
        let c = house_family_ceiling(family);
        assert!(c.cusp_arcsec.is_finite() && c.cusp_arcsec > 0.0);
        assert!(c.angle_arcsec.is_finite() && c.angle_arcsec > 0.0);
    }
}
