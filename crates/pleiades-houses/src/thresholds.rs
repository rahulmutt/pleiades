//! Per-formula-family numeric ceilings for the house-system gate.
//!
//! Mirrors `pleiades-data/src/thresholds.rs`: a small struct of ceilings plus a
//! lookup keyed by the abstract family, and a release-facing summary line.
#![forbid(unsafe_code)]

use crate::catalog::HouseFormulaFamily;

/// Arcsecond ceilings for one formula family.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HouseFamilyCeiling {
    /// Max allowed |residual| on any house cusp, arcseconds.
    pub cusp_arcsec: f64,
    /// Max allowed |residual| on Ascendant/Midheaven, arcseconds.
    pub angle_arcsec: f64,
}

/// Returns the provisional arcsecond ceilings for a formula family.
///
/// Space-division systems (Equal/WholeSign/Quadrant-Porphyry) are tight; the
/// iterative/projected families are looser. Values are tightened in the gate
/// rollout from measured SE-vs-pleiades residuals (see plan Task 10).
pub fn house_family_ceiling(family: HouseFormulaFamily) -> HouseFamilyCeiling {
    match family {
        HouseFormulaFamily::Equal
        | HouseFormulaFamily::WholeSign => HouseFamilyCeiling { cusp_arcsec: 1.0, angle_arcsec: 1.0 },
        // Porphyry is a space-division Quadrant system: tight.
        HouseFormulaFamily::Quadrant => HouseFamilyCeiling { cusp_arcsec: 30.0, angle_arcsec: 5.0 },
        HouseFormulaFamily::EquatorialProjection => HouseFamilyCeiling { cusp_arcsec: 15.0, angle_arcsec: 5.0 },
        HouseFormulaFamily::GreatCircle => HouseFamilyCeiling { cusp_arcsec: 15.0, angle_arcsec: 5.0 },
        HouseFormulaFamily::SolarArc
        | HouseFormulaFamily::Sector
        | HouseFormulaFamily::Custom
        | HouseFormulaFamily::Unknown => HouseFamilyCeiling { cusp_arcsec: 60.0, angle_arcsec: 10.0 },
    }
}

/// Compact release-facing summary of the family ceilings.
pub fn house_thresholds_summary_for_report() -> String {
    let equal = house_family_ceiling(HouseFormulaFamily::Equal);
    let quad = house_family_ceiling(HouseFormulaFamily::Quadrant);
    format!(
        "House ceilings: space-division {:.1}\u{2033} cusp, quadrant {:.1}\u{2033} cusp",
        equal.cusp_arcsec, quad.cusp_arcsec
    )
}

#[cfg(test)]
mod tests {
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
}
