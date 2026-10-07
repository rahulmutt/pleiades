//! The stars the star-anchored ayanamsas are fixed to, and each star's mean
//! place of date (issue #164 (c)).
//!
//! Swiss Ephemeris defines each star-anchored ayanamsa as a star's longitude
//! less a constant (`sweph.c`, `swi_get_ayanamsa_ex`). A star's mean place is
//! therefore its primary mode's mean ayanamsa plus that constant: Swiss
//! Ephemeris's own geometric `swe_fixstar` longitude equals that sum to
//! 0.000000″ over 1900–2100 for all five stars (measured 2026-10-07). The
//! latitude is a straight-line fit to `swe_fixstar` over 1900–2100, within
//! 0.12″ of it everywhere; the fit errs most at the window ends (0.112″ for
//! ζ Psc at 2100, measured 2026-10-07, issue #226).

use crate::sidereal_offset;
use pleiades_types::{Ayanamsa, Instant};

/// A star a sidereal ayanamsa is anchored to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum AnchorStar {
    /// α Virginis (Citra): True Citra and True Chitra.
    Spica,
    /// ζ Piscium (Revati): True Revati.
    ZetaPiscium,
    /// δ Cancri (Asellus Australis, Pushya): True Pushya and True Sheoran.
    DeltaCancri,
    /// λ Scorpii (Mula): True Mula.
    LambdaScorpii,
    /// Sgr A*: the Galactic Center modes other than Mardyks.
    GalacticCenter,
}

/// How an anchored ayanamsa reads its star.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum AnchorProjection {
    /// The star's ecliptic longitude.
    EclipticLongitude,
    /// The ecliptic longitude whose right ascension is the star's: Swiss
    /// Ephemeris's polar projection (`swi_armc_to_mc`), used by Galactic
    /// Center (Mula/Wilhelm).
    PolarRightAscension,
}

/// The star an ayanamsa is anchored to, and how it reads it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StarAnchor {
    /// The anchor star.
    pub star: AnchorStar,
    /// How the ayanamsa reads the star's place.
    pub projection: AnchorProjection,
}

/// The star anchor of `ayanamsa`, or `None` for an ayanamsa that is not read
/// from a star's place (every other built-in mode, the galactic-equator modes
/// and Mardyks included, and every custom ayanamsa).
pub fn star_anchor(ayanamsa: &Ayanamsa) -> Option<StarAnchor> {
    use AnchorProjection::{EclipticLongitude, PolarRightAscension};
    let (star, projection) = match ayanamsa {
        Ayanamsa::TrueCitra | Ayanamsa::TrueChitra => (AnchorStar::Spica, EclipticLongitude),
        Ayanamsa::TrueRevati => (AnchorStar::ZetaPiscium, EclipticLongitude),
        Ayanamsa::TruePushya | Ayanamsa::TrueSheoran => {
            (AnchorStar::DeltaCancri, EclipticLongitude)
        }
        Ayanamsa::TrueMula => (AnchorStar::LambdaScorpii, EclipticLongitude),
        Ayanamsa::GalacticCenter
        | Ayanamsa::GalacticCenterRgilbrand
        | Ayanamsa::GalacticCenterCochrane => (AnchorStar::GalacticCenter, EclipticLongitude),
        Ayanamsa::GalacticCenterMulaWilhelm => (AnchorStar::GalacticCenter, PolarRightAscension),
        _ => return None,
    };
    Some(StarAnchor { star, projection })
}

/// A star's mean place on the mean ecliptic and equinox of date, degrees.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnchorStarPlace {
    /// Ecliptic longitude, in `[0, 360)`.
    pub longitude_deg: f64,
    /// Ecliptic latitude.
    pub latitude_deg: f64,
}

/// `(primary mode, anchor longitude in that mode, latitude at J2000, latitude
/// rate per Julian century)`. The anchors are Swiss Ephemeris's
/// (`sweph.c` 3048–3090); the latitude fit is to its geometric `swe_fixstar`.
fn star_data(star: AnchorStar) -> (Ayanamsa, f64, f64, f64) {
    match star {
        AnchorStar::Spica => (Ayanamsa::TrueCitra, 180.0, -2.054_501_717, -0.007_550_067),
        AnchorStar::ZetaPiscium => (
            Ayanamsa::TrueRevati,
            359.833_333_333_3,
            -0.213_417_908,
            0.002_565_390,
        ),
        AnchorStar::DeltaCancri => (Ayanamsa::TruePushya, 106.0, 0.077_157_475, 0.003_140_560),
        AnchorStar::LambdaScorpii => (Ayanamsa::TrueMula, 240.0, -13.788_460_720, -0.013_920_718),
        AnchorStar::GalacticCenter => (
            Ayanamsa::GalacticCenter,
            240.0,
            -5.607_682_523,
            -0.013_203_301,
        ),
    }
}

/// `star`'s mean place of date at `instant` (read as TT), or `None` when its
/// primary mode has no offset there.
pub fn anchor_star_mean_place(star: AnchorStar, instant: Instant) -> Option<AnchorStarPlace> {
    let (primary, anchor, beta_j2000, beta_rate) = star_data(star);
    let ayanamsa = sidereal_offset(&primary, instant)?.degrees();
    let centuries = (instant.julian_day.days() - 2_451_545.0) / 36_525.0;
    Some(AnchorStarPlace {
        longitude_deg: (ayanamsa + anchor).rem_euclid(360.0),
        latitude_deg: beta_j2000 + beta_rate * centuries,
    })
}

#[cfg(test)]
mod tests;
