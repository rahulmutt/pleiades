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
    apparent_lunar_point_of, geocentric_apparent_ecliptic, geocentric_apparent_ecliptic_from,
    geocentric_apparent_lunar_point, geocentric_mean_of_date_ecliptic,
    geocentric_mean_of_date_lunar_point, heliocentric_j2000, heliocentric_of_date, is_lunar_point,
    j2000_spherical, mean_place_of_date, read_mean_place,
};
use crate::error::EventError;
use crate::state_vector::spherical_rates;
use pleiades_apparent::nutation::nutation;
use pleiades_ayanamsa::sidereal_offset;
use pleiades_backend::EphemerisBackend;
use pleiades_types::{Ayanamsa, CelestialBody, Instant, JulianDay, Motion, TimeScale, ZodiacMode};

/// `(longitude_deg, latitude_deg, distance_au)`. The distance is `None` only
/// for a lunar orbit point the backend serves as a direction (issue #118).
type EclipticTriple = (f64, f64, Option<f64>);

/// A body's place, whose distance is always known, as an [`EclipticTriple`].
fn with_distance((lon, lat, dist): (f64, f64, f64)) -> EclipticTriple {
    (lon, lat, Some(dist))
}

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
/// `SEFLG_SIDEREAL` convention, and a sidereal `pleiades-core` apparent chart
/// follows it too (issue #120). A sidereal `pleiades-core` *mean* chart
/// subtracts the ayanamsa from the backend's J2000 longitude and differs by
/// the precession since J2000 (FU-18 (b)).
///
/// The mean ayanamsa is used in every frame. For the star-anchored ayanamsa
/// classes (`TrueStar` and `Galactic`) Swiss Ephemeris's apparent sidereal
/// positions additionally fold the anchoring star's annual aberration (up to
/// about 20″) into the ayanamsa, so they differ from pleiades by that amount in
/// the apparent frame. True Citra and Galactic Center were measured; the other
/// ayanamsas in those classes follow from the same mechanism and were not. Such
/// an offset moves a crossing time by about 8 minutes for the Sun and by hours
/// for a slow planet such as Saturn, more near a station.
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
    let degrees = sidereal_offset(ayanamsa, instant)
        .map(|offset| offset.degrees())
        .ok_or_else(|| unsupported(format!("ayanamsa {ayanamsa} has no sidereal offset data")))?;
    // A custom ayanamsa can carry a NaN or infinite offset; reading it as a
    // zodiac would turn every longitude into NaN.
    if degrees.is_finite() {
        Ok(degrees)
    } else {
        Err(unsupported(format!(
            "ayanamsa {ayanamsa} has a non-finite sidereal offset"
        )))
    }
}

/// The zodiac of data serialized before the `zodiac` field existed: tropical.
#[cfg(feature = "serde")]
pub(crate) fn tropical_zodiac() -> ZodiacMode {
    ZodiacMode::Tropical
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
    // A lunar orbit point is a direction from the Earth; "node minus Sun" is
    // not a place (issue #118, FU-17 (b)).
    if heliocentric && is_lunar_point(body) {
        return Err(unsupported(format!(
            "heliocentric {what} undefined for {body:?}: a lunar orbit point is a direction \
             of the lunar orbit, not a body with a place from the Sun"
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

/// Geocentric apparent-of-date tropical place of `body`: the light-time
/// pipeline for a body, precession plus nutation for a lunar orbit point.
fn geocentric_apparent<B: EphemerisBackend>(
    backend: &B,
    body: &CelestialBody,
    label: &'static str,
    julian_day: f64,
) -> Result<EclipticTriple, EventError> {
    if is_lunar_point(body) {
        geocentric_apparent_lunar_point(backend, body.clone(), label, julian_day)
    } else {
        geocentric_apparent_ecliptic(backend, body.clone(), label, julian_day).map(with_distance)
    }
}

/// Geocentric mean-of-date tropical place of `body`: precession only, with a
/// distance required for a body and optional for a lunar orbit point.
fn geocentric_mean_of_date<B: EphemerisBackend>(
    backend: &B,
    body: &CelestialBody,
    label: &'static str,
    julian_day: f64,
) -> Result<EclipticTriple, EventError> {
    if is_lunar_point(body) {
        geocentric_mean_of_date_lunar_point(backend, body.clone(), label, julian_day)
    } else {
        geocentric_mean_of_date_ecliptic(backend, body.clone(), label, julian_day)
            .map(with_distance)
    }
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
            geocentric_apparent(backend, body, label, julian_day)?
        }
        CrossingFrame::Heliocentric => {
            let helio = heliocentric_j2000(backend, body.clone(), label, julian_day)?;
            with_distance(heliocentric_of_date(helio.position, julian_day)?)
        }
        CrossingFrame::GeocentricMeanOfDate => {
            geocentric_mean_of_date(backend, body, label, julian_day)?
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
    // The geocentric frames read the mean place once and reduce that same
    // read, instead of reading it again inside the reduction (issue #128);
    // the backend is deterministic, so the place is bit-identical.
    let (base, base_motion, tropical) = match reference.frame {
        CrossingFrame::GeocentricApparentOfDate => {
            let (mean, motion) = read_mean_place(backend, body.clone(), label, julian_day)?;
            let apparent = if is_lunar_point(body) {
                apparent_lunar_point_of(mean, label, julian_day)?
            } else {
                // A body without a distance gets no seed, so the reduction's
                // own read fails with `MissingDistance` exactly as before.
                let seed = mean.2.map(|distance| (mean.0, mean.1, distance));
                geocentric_apparent_ecliptic_from(backend, body.clone(), label, julian_day, seed)
                    .map(with_distance)?
            };
            (mean, motion, apparent)
        }
        CrossingFrame::Heliocentric => {
            let helio = heliocentric_j2000(backend, body.clone(), label, julian_day)?;
            let of_date = heliocentric_of_date(helio.position, julian_day)?;
            (
                with_distance(j2000_spherical(helio.position)),
                helio
                    .velocity
                    .map(|velocity| spherical_rates(helio.position, velocity)),
                with_distance(of_date),
            )
        }
        CrossingFrame::GeocentricMeanOfDate => {
            let (mean, motion) = read_mean_place(backend, body.clone(), label, julian_day)?;
            if !is_lunar_point(body) && mean.2.is_none() {
                return Err(EventError::MissingDistance {
                    body_label: label,
                    julian_day,
                });
            }
            let of_date = mean_place_of_date(mean, label, julian_day)?;
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
