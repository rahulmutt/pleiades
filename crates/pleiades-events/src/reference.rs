//! The frame and zodiac a longitude is measured in, and the one place both
//! become a position.
//!
//! A tropical reference returns the frame's place untouched, which keeps the
//! pre-existing frames bit-identical. A sidereal reference moves the longitude
//! to the mean equinox of date (removing nutation in longitude where the frame
//! carries it) and subtracts the mean ayanamsa — the Swiss Ephemeris
//! `SEFLG_SIDEREAL` convention. Latitude and distance never change.

use crate::crossings::{body_label, CrossingFrame};
use crate::ephemeris::{
    geocentric_apparent_ecliptic, geocentric_mean_of_date_ecliptic, heliocentric_j2000,
    heliocentric_of_date, j2000_spherical, read_mean_ecliptic_with_motion,
};
use crate::error::EventError;
use crate::state_vector::spherical_rates;
use pleiades_apparent::nutation::nutation;
use pleiades_ayanamsa::sidereal_offset;
use pleiades_backend::EphemerisBackend;
use pleiades_types::{Ayanamsa, CelestialBody, Instant, JulianDay, Motion, TimeScale, ZodiacMode};

/// `(longitude_deg, latitude_deg, distance_au)`.
type EclipticTriple = (f64, f64, f64);

/// The frame and zodiac a longitude is measured in.
///
/// A [`CrossingFrame`] converts into the tropical reference, so every method
/// that takes a reference also takes a bare frame:
///
/// ```
/// use pleiades_events::{CrossingFrame, CrossingReference};
/// use pleiades_types::{Ayanamsa, ZodiacMode};
///
/// let tropical: CrossingReference = CrossingFrame::GeocentricApparentOfDate.into();
/// assert_eq!(tropical.zodiac, ZodiacMode::Tropical);
///
/// let sidereal =
///     CrossingReference::sidereal(CrossingFrame::GeocentricApparentOfDate, Ayanamsa::Lahiri);
/// assert_eq!(sidereal.frame, CrossingFrame::GeocentricApparentOfDate);
/// ```
///
/// A sidereal longitude is the longitude on the **mean** equinox of date minus
/// the mean ayanamsa (`pleiades_ayanamsa::sidereal_offset`), so nutation does
/// not move a body through a sidereal zodiac. This is the Swiss Ephemeris
/// `SEFLG_SIDEREAL` convention. A sidereal `pleiades-core` apparent chart
/// keeps nutation and can differ from it by up to about 17″.
///
/// The mean ayanamsa is used in every frame. For star-anchored ayanamsas
/// (True Citra, Galactic Center) Swiss Ephemeris's apparent sidereal positions
/// additionally fold the anchoring star's annual aberration (up to about 20″)
/// into the ayanamsa, so they differ from pleiades by that amount in the
/// apparent frame.
///
/// The heliocentric frame is tropical only.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CrossingReference {
    /// The coordinate and centre convention.
    pub frame: CrossingFrame,
    /// The zodiac longitudes are read in.
    pub zodiac: ZodiacMode,
}

impl CrossingReference {
    /// `frame` in the tropical zodiac; the same as `frame.into()`.
    pub fn tropical(frame: CrossingFrame) -> Self {
        Self {
            frame,
            zodiac: ZodiacMode::Tropical,
        }
    }

    /// `frame` in the sidereal zodiac of `ayanamsa`.
    pub fn sidereal(frame: CrossingFrame, ayanamsa: Ayanamsa) -> Self {
        Self {
            frame,
            zodiac: ZodiacMode::Sidereal { ayanamsa },
        }
    }
}

impl From<CrossingFrame> for CrossingReference {
    fn from(frame: CrossingFrame) -> Self {
        Self::tropical(frame)
    }
}

impl From<&CrossingReference> for CrossingReference {
    fn from(reference: &CrossingReference) -> Self {
        reference.clone()
    }
}

fn unsupported(detail: impl Into<String>) -> EventError {
    EventError::UnsupportedFrame {
        detail: detail.into(),
    }
}

/// Mean ayanamsa in degrees at `julian_day`.
fn ayanamsa_deg(ayanamsa: &Ayanamsa, julian_day: f64) -> Result<f64, EventError> {
    // `sidereal_offset` reads the day as TT; the engine's day is TDB. The two
    // differ by under 2 ms, which moves the ayanamsa by less than 1e-9″.
    let instant = Instant::new(JulianDay::from_days(julian_day), TimeScale::Tt);
    sidereal_offset(ayanamsa, instant)
        .map(|offset| offset.degrees())
        .ok_or_else(|| unsupported(format!("ayanamsa {ayanamsa} has no sidereal offset data")))
}

/// Fails for a body/frame/zodiac combination that is not defined. `what`
/// completes "heliocentric {what} undefined for {body}" (for example
/// `"crossings are"`). `julian_day` is the instant the ayanamsa is probed at.
pub(crate) fn check_supported(
    body: &CelestialBody,
    reference: &CrossingReference,
    julian_day: f64,
    what: &str,
) -> Result<(), EventError> {
    let heliocentric = matches!(reference.frame, CrossingFrame::Heliocentric);
    if heliocentric && matches!(body, CelestialBody::Sun | CelestialBody::Moon) {
        return Err(unsupported(format!(
            "heliocentric {what} undefined for {body:?}"
        )));
    }
    match &reference.zodiac {
        ZodiacMode::Tropical => Ok(()),
        ZodiacMode::Sidereal { .. } if heliocentric => Err(unsupported(
            "a sidereal zodiac is not supported in the heliocentric frame",
        )),
        ZodiacMode::Sidereal { ayanamsa } => ayanamsa_deg(ayanamsa, julian_day).map(|_| ()),
        _ => Err(unsupported("unsupported zodiac mode")),
    }
}

/// Moves a frame's tropical place into the reference's zodiac.
fn in_zodiac(
    (lon, lat, dist): EclipticTriple,
    reference: &CrossingReference,
    julian_day: f64,
) -> Result<EclipticTriple, EventError> {
    let ayanamsa = match &reference.zodiac {
        // No arithmetic: the tropical frames stay bit-identical.
        ZodiacMode::Tropical => return Ok((lon, lat, dist)),
        ZodiacMode::Sidereal { ayanamsa } => ayanamsa,
        _ => return Err(unsupported("unsupported zodiac mode")),
    };
    // Nutation slides the equinox along the ecliptic, so removing Δψ from a
    // true-equinox longitude gives the mean-equinox longitude exactly.
    let nutation_deg = match reference.frame {
        CrossingFrame::GeocentricApparentOfDate => {
            nutation(julian_day)
                .map_err(|e| EventError::Backend(format!("sidereal nutation failed: {e}")))?
                .delta_psi_arcsec
                / 3600.0
        }
        CrossingFrame::GeocentricMeanOfDate => 0.0,
        CrossingFrame::Heliocentric => {
            return Err(unsupported(
                "a sidereal zodiac is not supported in the heliocentric frame",
            ))
        }
    };
    let shift = nutation_deg + ayanamsa_deg(ayanamsa, julian_day)?;
    Ok(((lon - shift).rem_euclid(360.0), lat, dist))
}

/// Ecliptic `(longitude_deg, latitude_deg, distance_au)` of `body` in
/// `reference` at `julian_day` (TDB). The crossing engine root-finds on the
/// longitude; `longitude_at` and `position_at` report it.
pub(crate) fn ecliptic_in<B: EphemerisBackend>(
    backend: &B,
    body: &CelestialBody,
    reference: &CrossingReference,
    julian_day: f64,
) -> Result<EclipticTriple, EventError> {
    let label = body_label(body);
    let tropical = match reference.frame {
        CrossingFrame::GeocentricApparentOfDate => {
            geocentric_apparent_ecliptic(backend, body.clone(), label, julian_day)?
        }
        CrossingFrame::Heliocentric => {
            let helio = heliocentric_j2000(backend, body.clone(), label, julian_day)?;
            heliocentric_of_date(helio.position, julian_day)?
        }
        CrossingFrame::GeocentricMeanOfDate => {
            geocentric_mean_of_date_ecliptic(backend, body.clone(), label, julian_day)?
        }
    };
    in_zodiac(tropical, reference, julian_day)
}

/// A place with the base place and speed its own speed is derived from.
pub(crate) struct SampledPlace {
    /// The J2000 place the backend's speed describes.
    pub(crate) base: EclipticTriple,
    /// Speed of `base`, when the backend reports one.
    pub(crate) base_motion: Option<Motion>,
    /// The place in the reference; equal to [`ecliptic_in`].
    pub(crate) corrected: EclipticTriple,
}

/// [`ecliptic_in`] together with its base place and speed, for `position_at`.
pub(crate) fn sampled_place<B: EphemerisBackend>(
    backend: &B,
    body: &CelestialBody,
    reference: &CrossingReference,
    julian_day: f64,
) -> Result<SampledPlace, EventError> {
    let label = body_label(body);
    let (base, base_motion, tropical) = match reference.frame {
        CrossingFrame::GeocentricApparentOfDate => {
            let (mean, motion) =
                read_mean_ecliptic_with_motion(backend, body.clone(), label, julian_day)?;
            let apparent = geocentric_apparent_ecliptic(backend, body.clone(), label, julian_day)?;
            (mean, motion, apparent)
        }
        CrossingFrame::Heliocentric => {
            let helio = heliocentric_j2000(backend, body.clone(), label, julian_day)?;
            let of_date = heliocentric_of_date(helio.position, julian_day)?;
            (
                j2000_spherical(helio.position),
                helio
                    .velocity
                    .map(|velocity| spherical_rates(helio.position, velocity)),
                of_date,
            )
        }
        CrossingFrame::GeocentricMeanOfDate => {
            let (mean, motion) =
                read_mean_ecliptic_with_motion(backend, body.clone(), label, julian_day)?;
            let of_date =
                geocentric_mean_of_date_ecliptic(backend, body.clone(), label, julian_day)?;
            (mean, motion, of_date)
        }
    };
    Ok(SampledPlace {
        base,
        base_motion,
        corrected: in_zodiac(tropical, reference, julian_day)?,
    })
}

#[cfg(test)]
mod tests;
