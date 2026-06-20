//! The top-level `CompressedArtifact` type with encode/decode/lookup.

use core::fmt;

use pleiades_types::{
    Angle, CelestialBody, EclipticCoordinates, EquatorialCoordinates, Instant, Motion, TimeScale,
};

use crate::channels::{BodyArtifact, ChannelKind};
use crate::codec::validate_body_artifacts;
use crate::codec::{
    decode_artifact_profile, decode_body, decode_endian_policy, encode_artifact_profile,
    encode_body, encode_endian_policy, fnv1a64, write_u16, write_u64, Cursor,
};
use crate::error::{CompressionError, CompressionErrorKind};
use crate::format::{
    ArtifactHeader, ArtifactOutput, ArtifactProfileCoverageSummary,
    ArtifactResidualBodyCoverageSummary,
};

/// A compressed ephemeris artifact.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct CompressedArtifact {
    /// File header metadata.
    pub header: ArtifactHeader,
    /// Checksum over the payload bytes.
    pub checksum: u64,
    /// Body series stored in the artifact.
    pub bodies: Vec<BodyArtifact>,
}

impl CompressedArtifact {
    /// Creates a new artifact with an unset checksum.
    pub fn new(header: ArtifactHeader, bodies: Vec<BodyArtifact>) -> Self {
        Self {
            header,
            checksum: 0,
            bodies,
        }
    }

    /// Validates the in-memory artifact before encoding or regeneration.
    ///
    /// This checks the header metadata, the bundled body/profile coverage,
    /// canonical body ordering, duplicate body entries, and every body
    /// segment's metadata. It is useful for generators that want to fail fast
    /// before writing a deterministic binary payload.
    pub fn validate(&self) -> Result<(), CompressionError> {
        self.header.validate()?;
        validate_body_artifacts(&self.bodies)?;
        self.profile_coverage_summary().validate()?;
        for body in &self.bodies {
            body.validate()?;
        }

        // A heliocentric body is reconstructed against the geocentric Sun at lookup,
        // so a Sun body must be present. Fail closed rather than mis-reconstruct.
        let has_heliocentric = self
            .bodies
            .iter()
            .any(|b| b.frame == crate::channels::StoredFrame::Heliocentric);
        if has_heliocentric {
            let sun = self
                .bodies
                .iter()
                .find(|b| b.body == CelestialBody::Sun);
            match sun {
                Some(s) if s.frame == crate::channels::StoredFrame::Geocentric => {}
                Some(_) => {
                    return Err(CompressionError::new(
                        CompressionErrorKind::InvalidFormat,
                        "artifact has heliocentric bodies but the Sun is not stored geocentric",
                    ));
                }
                None => {
                    return Err(CompressionError::new(
                        CompressionErrorKind::InvalidFormat,
                        "artifact has heliocentric bodies but contains no Sun reference",
                    ));
                }
            }
        }

        Ok(())
    }

    /// Returns the artifact profile paired with the bundled body list.
    pub fn profile_coverage_summary(&self) -> ArtifactProfileCoverageSummary {
        ArtifactProfileCoverageSummary::new(
            self.header.profile.clone(),
            self.bodies.iter().map(|body| body.body.clone()).collect(),
        )
    }

    /// Returns the bodies that include at least one residual-correction segment.
    pub fn residual_body_coverage_summary(&self) -> ArtifactResidualBodyCoverageSummary {
        ArtifactResidualBodyCoverageSummary::new(self.residual_bodies())
    }

    /// Returns the body artifact for the requested body, if present.
    pub fn body_artifact(&self, body: &CelestialBody) -> Option<&BodyArtifact> {
        self.bodies.iter().find(|series| &series.body == body)
    }

    /// Returns the total number of segments stored in the artifact.
    pub fn segment_count(&self) -> usize {
        self.bodies.iter().map(|body| body.segments.len()).sum()
    }

    /// Returns the number of segments that carry residual-correction channels.
    pub fn residual_segment_count(&self) -> usize {
        self.bodies
            .iter()
            .flat_map(|body| body.segments.iter())
            .filter(|segment| !segment.residual_channels.is_empty())
            .count()
    }

    /// Returns the bundled bodies that include at least one residual-correction segment.
    pub fn residual_bodies(&self) -> Vec<CelestialBody> {
        self.bodies
            .iter()
            .filter(|body| {
                body.segments
                    .iter()
                    .any(|segment| !segment.residual_channels.is_empty())
            })
            .map(|body| body.body.clone())
            .collect()
    }

    /// Returns a compact one-line summary of the artifact body, segment, and
    /// residual-correction coverage.
    pub fn summary_line(&self) -> String {
        let residual_bodies = self.residual_bodies();
        let residual_bodies = if residual_bodies.is_empty() {
            "none".to_string()
        } else {
            crate::join_display(&residual_bodies)
        };

        format!(
            "bodies: {}; segments: {}; residual-bearing segments: {}; residual-bearing bodies: {}",
            self.bodies.len(),
            self.segment_count(),
            self.residual_segment_count(),
            residual_bodies,
        )
    }

    /// Returns the body segment covering the requested instant.
    pub fn segment_for(
        &self,
        body: &CelestialBody,
        instant: Instant,
    ) -> Result<&crate::channels::Segment, CompressionError> {
        let series = self.body_artifact(body).ok_or_else(|| {
            CompressionError::new(
                CompressionErrorKind::MissingBody,
                format!("no packed data exists for {body:?}"),
            )
        })?;

        series.segment_at(instant).ok_or_else(|| {
            CompressionError::new(
                CompressionErrorKind::OutOfRangeInstant,
                format!("no packed segment covers {body:?} at {instant:?}"),
            )
        })
    }

    /// Returns the on-disk checksum for this artifact.
    pub fn checksum(&self) -> Result<u64, CompressionError> {
        Ok(fnv1a64(&self.encode_payload()?))
    }

    /// Encodes the artifact as a deterministic binary blob.
    pub fn encode(&self) -> Result<Vec<u8>, CompressionError> {
        let payload = self.encode_payload()?;
        let checksum = fnv1a64(&payload);

        let mut bytes = Vec::new();
        bytes.extend_from_slice(&crate::ARTIFACT_MAGIC);
        write_u16(&mut bytes, self.header.version);
        write_u64(&mut bytes, checksum);
        bytes.extend_from_slice(&payload);
        Ok(bytes)
    }

    /// Decodes an artifact from a binary blob.
    ///
    /// Checksum mismatches are rejected with [`CompressionErrorKind::ChecksumMismatch`]
    /// before the payload is parsed.
    pub fn decode(bytes: &[u8]) -> Result<Self, CompressionError> {
        let mut cursor = Cursor::new(bytes);
        let magic = cursor.read_array::<8>()?;
        if magic != crate::ARTIFACT_MAGIC {
            return Err(CompressionError::new(
                CompressionErrorKind::InvalidMagic,
                "compressed artifact magic header did not match",
            ));
        }

        let version = cursor.read_u16()?;
        if version != crate::ARTIFACT_VERSION {
            return Err(CompressionError::new(
                CompressionErrorKind::UnsupportedVersion,
                format!("artifact version {version} is not supported"),
            ));
        }

        let checksum = cursor.read_u64()?;
        let payload = cursor.remaining();
        if fnv1a64(payload) != checksum {
            return Err(CompressionError::new(
                CompressionErrorKind::ChecksumMismatch,
                "compressed artifact checksum did not match",
            ));
        }

        let mut payload_cursor = Cursor::new(payload);
        let header = ArtifactHeader {
            version,
            generation_label: payload_cursor.read_string()?,
            source: payload_cursor.read_string()?,
            endian_policy: decode_endian_policy(payload_cursor.read_u8()?)?,
            profile: decode_artifact_profile(&mut payload_cursor)?,
        };
        let body_count = payload_cursor.read_u16()? as usize;
        let mut bodies = Vec::with_capacity(body_count);
        for _ in 0..body_count {
            let body = decode_body(&mut payload_cursor)?;
            bodies.push(body);
        }

        if !payload_cursor.is_finished() {
            return Err(CompressionError::new(
                CompressionErrorKind::InvalidFormat,
                "compressed artifact contained trailing bytes",
            ));
        }

        let artifact = Self {
            header,
            checksum,
            bodies,
        };
        artifact.validate()?;

        Ok(artifact)
    }

    /// Returns the ecliptic coordinates for a body at a given instant.
    ///
    /// The artifact profile must advertise `EclipticCoordinates` as a derived
    /// output before this helper will serve the result.
    pub fn lookup_ecliptic(
        &self,
        body: &CelestialBody,
        instant: Instant,
    ) -> Result<EclipticCoordinates, CompressionError> {
        self.require_output_support(ArtifactOutput::EclipticCoordinates)?;

        if !matches!(instant.scale, TimeScale::Tt | TimeScale::Tdb) {
            return Err(CompressionError::new(
                CompressionErrorKind::UnsupportedTimeScale,
                "packaged lookup only accepts TT or TDB instants",
            ));
        }

        let segment = self.segment_for(body, instant)?;

        let span = segment.span_days();
        let x = if span == 0.0 {
            0.0
        } else {
            (instant.julian_day.days() - segment.start.julian_day.days()) / span
        };

        use pleiades_types::{Latitude, Longitude};
        let longitude = segment.evaluate_channel(ChannelKind::Longitude, x)?;
        let latitude = segment.evaluate_channel(ChannelKind::Latitude, x)?;
        let distance_au = segment.evaluate_channel(ChannelKind::DistanceAu, x)?;

        let stored = EclipticCoordinates::new(
            Longitude::from_degrees(longitude),
            Latitude::from_degrees(latitude),
            Some(distance_au),
        );

        let frame = self
            .body_artifact(body)
            .map(|b| b.frame)
            .unwrap_or(crate::channels::StoredFrame::Geocentric);

        match frame {
            crate::channels::StoredFrame::Geocentric => Ok(stored),
            crate::channels::StoredFrame::Heliocentric => {
                let sun_geo = self.lookup_ecliptic(&CelestialBody::Sun, instant)?;
                crate::frame_recombine::geocentric_from_heliocentric(&stored, &sun_geo).ok_or_else(
                    || {
                        CompressionError::new(
                            CompressionErrorKind::InvalidFormat,
                            "heliocentric reconstruction requires finite distances on body and Sun",
                        )
                    },
                )
            }
        }
    }

    /// Returns equatorial coordinates reconstructed from the stored ecliptic channels.
    ///
    /// This keeps the artifact format focused on the stored channels while still allowing
    /// the runtime to reconstruct a derived coordinate family when the caller supplies the
    /// geometric obliquity used for the mean-obliquity frame rotation. The artifact profile
    /// must advertise `EquatorialCoordinates` as a derived output before this helper will
    /// serve the result.
    pub fn lookup_equatorial(
        &self,
        body: &CelestialBody,
        instant: Instant,
        obliquity: Angle,
    ) -> Result<EquatorialCoordinates, CompressionError> {
        self.require_output_support(ArtifactOutput::EquatorialCoordinates)?;
        Ok(self
            .lookup_ecliptic(body, instant)?
            .to_equatorial(obliquity))
    }

    /// Returns the angular and radial speeds for a body at a given instant.
    ///
    /// Speeds are computed analytically from the fitted polynomial segment derivatives.
    /// For geocentric-stored bodies (Sun, Moon, Eros) the derivative of the stored
    /// channel is returned directly. For heliocentric-stored planets the heliocentric
    /// spherical state is converted to Cartesian, added to the Sun's geocentric
    /// Cartesian state, and converted back to geocentric spherical rates.
    ///
    /// The artifact profile must advertise `Motion` as a supported output (via a
    /// non-`Unsupported` `SpeedPolicy`) before this helper will serve the result.
    pub fn lookup_motion(
        &self,
        body: &CelestialBody,
        instant: Instant,
    ) -> Result<Motion, CompressionError> {
        self.require_output_support(ArtifactOutput::Motion)?;

        if !matches!(instant.scale, TimeScale::Tt | TimeScale::Tdb) {
            return Err(CompressionError::new(
                CompressionErrorKind::UnsupportedTimeScale,
                "packaged lookup only accepts TT or TDB instants",
            ));
        }

        let segment = self.segment_for(body, instant)?;

        let span = segment.span_days();
        let x = if span == 0.0 {
            0.0
        } else {
            (instant.julian_day.days() - segment.start.julian_day.days()) / span
        };

        // dP/dx × (1/span_days) converts normalized derivative to per-day rate.
        let dlon_dt = segment.evaluate_channel_derivative(ChannelKind::Longitude, x)? / span;
        let dlat_dt = segment.evaluate_channel_derivative(ChannelKind::Latitude, x)? / span;
        let ddist_dt = segment.evaluate_channel_derivative(ChannelKind::DistanceAu, x)? / span;

        let frame = self
            .body_artifact(body)
            .map(|b| b.frame)
            .unwrap_or(crate::channels::StoredFrame::Geocentric);

        match frame {
            crate::channels::StoredFrame::Geocentric => Ok(Motion {
                longitude_deg_per_day: Some(dlon_dt),
                latitude_deg_per_day: Some(dlat_dt),
                distance_au_per_day: Some(ddist_dt),
            }),
            crate::channels::StoredFrame::Heliocentric => {
                // Build the heliocentric spherical state (channels are in degrees,
                // recombination functions use radians).
                let lon = segment.evaluate_channel(ChannelKind::Longitude, x)?;
                let lat = segment.evaluate_channel(ChannelKind::Latitude, x)?;
                let dist = segment.evaluate_channel(ChannelKind::DistanceAu, x)?;
                let helio =
                    crate::frame_recombine::spherical_state_to_cartesian(
                        crate::frame_recombine::SphericalState {
                            lon_rad: lon.to_radians(),
                            lat_rad: lat.to_radians(),
                            dist_au: dist,
                            lon_rate_rad_per_day: dlon_dt.to_radians(),
                            lat_rate_rad_per_day: dlat_dt.to_radians(),
                            dist_rate_au_per_day: ddist_dt,
                        },
                    );
                let sun = self.sun_cartesian_state(instant)?;
                let geo = crate::frame_recombine::CartesianState {
                    pos_au: [
                        helio.pos_au[0] + sun.pos_au[0],
                        helio.pos_au[1] + sun.pos_au[1],
                        helio.pos_au[2] + sun.pos_au[2],
                    ],
                    vel_au_per_day: [
                        helio.vel_au_per_day[0] + sun.vel_au_per_day[0],
                        helio.vel_au_per_day[1] + sun.vel_au_per_day[1],
                        helio.vel_au_per_day[2] + sun.vel_au_per_day[2],
                    ],
                };
                let s = crate::frame_recombine::cartesian_state_to_spherical(geo);
                Ok(Motion {
                    longitude_deg_per_day: Some(s.lon_rate_rad_per_day.to_degrees()),
                    latitude_deg_per_day: Some(s.lat_rate_rad_per_day.to_degrees()),
                    distance_au_per_day: Some(s.dist_rate_au_per_day),
                })
            }
        }
    }

    /// Returns the Sun's geocentric Cartesian position (AU) and velocity (AU/day)
    /// at the given instant by evaluating the Sun's stored geocentric segment.
    ///
    /// The Sun is always stored geocentric per the SP2 Sun-presence invariant
    /// enforced by `validate()`.
    fn sun_cartesian_state(
        &self,
        instant: Instant,
    ) -> Result<crate::frame_recombine::CartesianState, CompressionError> {
        let sun_segment = self.segment_for(&CelestialBody::Sun, instant)?;
        let span = sun_segment.span_days();
        let x = if span == 0.0 {
            0.0
        } else {
            (instant.julian_day.days() - sun_segment.start.julian_day.days()) / span
        };
        let lon = sun_segment.evaluate_channel(ChannelKind::Longitude, x)?;
        let lat = sun_segment.evaluate_channel(ChannelKind::Latitude, x)?;
        let dist = sun_segment.evaluate_channel(ChannelKind::DistanceAu, x)?;
        let dlon_dt = sun_segment.evaluate_channel_derivative(ChannelKind::Longitude, x)? / span;
        let dlat_dt = sun_segment.evaluate_channel_derivative(ChannelKind::Latitude, x)? / span;
        let ddist_dt =
            sun_segment.evaluate_channel_derivative(ChannelKind::DistanceAu, x)? / span;
        Ok(crate::frame_recombine::spherical_state_to_cartesian(
            crate::frame_recombine::SphericalState {
                lon_rad: lon.to_radians(),
                lat_rad: lat.to_radians(),
                dist_au: dist,
                lon_rate_rad_per_day: dlon_dt.to_radians(),
                lat_rate_rad_per_day: dlat_dt.to_radians(),
                dist_rate_au_per_day: ddist_dt,
            },
        ))
    }

    fn require_output_support(&self, output: ArtifactOutput) -> Result<(), CompressionError> {
        if self.header.profile.supports_output(output) {
            Ok(())
        } else {
            Err(CompressionError::new(
                CompressionErrorKind::InvalidFormat,
                format!("artifact profile does not support {output}"),
            ))
        }
    }

    fn encode_payload(&self) -> Result<Vec<u8>, CompressionError> {
        self.validate()?;

        let mut bytes = Vec::new();
        crate::codec::write_string(&mut bytes, &self.header.generation_label);
        crate::codec::write_string(&mut bytes, &self.header.source);
        crate::codec::write_u8(&mut bytes, encode_endian_policy(self.header.endian_policy));
        encode_artifact_profile(&mut bytes, &self.header.profile)?;
        write_u16(&mut bytes, self.bodies.len() as u16);
        for body in &self.bodies {
            encode_body(&mut bytes, body)?;
        }
        Ok(bytes)
    }
}

impl fmt::Display for CompressedArtifact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

#[cfg(test)]
mod reframe_lookup_tests {
    use super::*;
    use crate::channels::{BodyArtifact, ChannelKind, PolynomialChannel, Segment, StoredFrame};
    use crate::frame_recombine::heliocentric_from_geocentric;
    use pleiades_types::{
        CelestialBody, EclipticCoordinates, Instant, JulianDay, Latitude, Longitude, TimeScale,
    };

    // Builds a single-segment body whose three channels are constant (degree-0)
    // equal to the given ecliptic coordinates across the whole span.
    fn const_body(
        body: CelestialBody,
        frame: StoredFrame,
        start: f64,
        end: f64,
        coords: &EclipticCoordinates,
    ) -> BodyArtifact {
        let channels = vec![
            PolynomialChannel::new(ChannelKind::Longitude, 9, vec![coords.longitude.degrees()]),
            PolynomialChannel::new(ChannelKind::Latitude, 9, vec![coords.latitude.degrees()]),
            PolynomialChannel::new(ChannelKind::DistanceAu, 10, vec![coords.distance_au.unwrap()]),
        ];
        let seg = Segment::new(
            Instant::new(JulianDay::from_days(start), TimeScale::Tt),
            Instant::new(JulianDay::from_days(end), TimeScale::Tt),
            channels,
        );
        BodyArtifact::with_frame(body, vec![seg], frame)
    }

    // Constructs a CompressedArtifact from the given bodies and validates it,
    // panicking if validation fails.
    fn build_test_artifact(bodies: Vec<BodyArtifact>) -> CompressedArtifact {
        try_build_test_artifact(bodies).expect("test artifact should be valid")
    }

    // Constructs a CompressedArtifact from the given bodies and returns the
    // validation Result so callers can assert on error cases.
    fn try_build_test_artifact(
        bodies: Vec<BodyArtifact>,
    ) -> Result<CompressedArtifact, CompressionError> {
        let artifact = CompressedArtifact::new(
            ArtifactHeader::new("reframe-test", "synthetic test fixture"),
            bodies,
        );
        artifact.validate().map(|_| artifact)
    }

    #[test]
    fn heliocentric_body_reconstructs_geocentric() {
        let sun_geo = EclipticCoordinates::new(
            Longitude::from_degrees(95.0),
            Latitude::from_degrees(0.0),
            Some(1.0),
        );
        let jupiter_geo = EclipticCoordinates::new(
            Longitude::from_degrees(200.0),
            Latitude::from_degrees(1.2),
            Some(5.4),
        );
        let jupiter_helio = heliocentric_from_geocentric(&jupiter_geo, &sun_geo).unwrap();

        let artifact = build_test_artifact(vec![
            const_body(CelestialBody::Sun, StoredFrame::Geocentric, 0.0, 100.0, &sun_geo),
            const_body(CelestialBody::Jupiter, StoredFrame::Heliocentric, 0.0, 100.0, &jupiter_helio),
        ]);

        let at = Instant::new(JulianDay::from_days(50.0), TimeScale::Tt);
        let out = artifact.lookup_ecliptic(&CelestialBody::Jupiter, at).unwrap();
        assert!((out.longitude.degrees() - 200.0).abs() < 1e-6);
        assert!((out.latitude.degrees() - 1.2).abs() < 1e-6);
        assert!((out.distance_au.unwrap() - 5.4).abs() < 1e-6);
    }

    #[test]
    fn heliocentric_body_without_sun_fails_validation() {
        let jupiter_helio = EclipticCoordinates::new(
            Longitude::from_degrees(120.0),
            Latitude::from_degrees(0.5),
            Some(5.0),
        );
        let result = try_build_test_artifact(vec![const_body(
            CelestialBody::Jupiter,
            StoredFrame::Heliocentric,
            0.0,
            100.0,
            &jupiter_helio,
        )]);
        assert!(result.is_err(), "heliocentric body without a Sun must fail validation");
    }

    #[test]
    fn heliocentric_body_with_heliocentric_sun_fails_validation() {
        // A Sun stored as Heliocentric would cause infinite recursion at lookup
        // (lookup_ecliptic(Sun) → geocentric_from_heliocentric → lookup_ecliptic(Sun) → …).
        // validate() must reject this configuration fail-closed.
        let sun_coords = EclipticCoordinates::new(
            Longitude::from_degrees(95.0),
            Latitude::from_degrees(0.0),
            Some(1.0),
        );
        let jupiter_helio = EclipticCoordinates::new(
            Longitude::from_degrees(120.0),
            Latitude::from_degrees(0.5),
            Some(5.0),
        );
        let result = try_build_test_artifact(vec![
            const_body(CelestialBody::Sun, StoredFrame::Heliocentric, 0.0, 100.0, &sun_coords),
            const_body(CelestialBody::Jupiter, StoredFrame::Heliocentric, 0.0, 100.0, &jupiter_helio),
        ]);
        assert!(
            result.is_err(),
            "artifact with heliocentric Jupiter and heliocentric Sun must fail validation"
        );
    }
}

#[cfg(test)]
mod motion_lookup_tests {
    use super::*;
    use crate::channels::{BodyArtifact, ChannelKind, PolynomialChannel, Segment, StoredFrame};
    use crate::format::{ArtifactProfile, SpeedPolicy};
    use pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale};

    /// Creates a header whose profile advertises Motion as a derived output
    /// via FittedDerivative speed policy.
    fn motion_header(label: &str) -> ArtifactHeader {
        ArtifactHeader::with_profile(
            label,
            "motion test fixture",
            ArtifactProfile::new(
                vec![ChannelKind::Longitude, ChannelKind::Latitude, ChannelKind::DistanceAu],
                vec![
                    ArtifactOutput::EclipticCoordinates,
                    ArtifactOutput::EquatorialCoordinates,
                ],
                vec![
                    ArtifactOutput::ApparentCorrections,
                    ArtifactOutput::TopocentricCoordinates,
                    ArtifactOutput::SiderealCoordinates,
                ],
                SpeedPolicy::FittedDerivative,
            ),
        )
    }

    #[test]
    fn lookup_motion_for_geocentric_body_is_direct_derivative() {
        // Sun segment: longitude linear from 100 to 102 over a 10-day span (x ∈ [0,1]).
        // dλ/dx = 2 deg; dλ/dt = 2 / 10 days = 0.2 deg/day.
        // latitude and distance are constant, so their rates are 0.
        let start = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
        let end = Instant::new(JulianDay::from_days(2_451_555.0), TimeScale::Tt);
        let segment = Segment::new(
            start,
            end,
            vec![
                PolynomialChannel::linear(ChannelKind::Longitude, 9, 100.0, 102.0),
                PolynomialChannel::linear(ChannelKind::Latitude, 9, 5.0, 5.0),
                PolynomialChannel::linear(ChannelKind::DistanceAu, 10, 1.0, 1.0),
            ],
        );
        let artifact = CompressedArtifact::new(
            motion_header("geocentric-derivative-test"),
            vec![BodyArtifact::new(CelestialBody::Sun, vec![segment])],
        );
        let at = Instant::new(JulianDay::from_days(2_451_550.0), TimeScale::Tt);
        let m = artifact.lookup_motion(&CelestialBody::Sun, at).unwrap();
        assert!(
            (m.longitude_deg_per_day.unwrap() - 0.2).abs() < 1e-9,
            "longitude rate should be 0.2 deg/day, got {:?}",
            m.longitude_deg_per_day
        );
        assert!(
            m.latitude_deg_per_day.unwrap().abs() < 1e-9,
            "latitude rate should be 0, got {:?}",
            m.latitude_deg_per_day
        );
        assert!(
            m.distance_au_per_day.unwrap().abs() < 1e-9,
            "distance rate should be 0, got {:?}",
            m.distance_au_per_day
        );
    }

    #[test]
    fn lookup_motion_heliocentric_body_returns_finite_geocentric_rates() {
        // Two-body artifact: Sun geocentric with known motion, Jupiter heliocentric with known motion.
        // We only assert that the returned motion components are all Some(_) and finite —
        // correctness of the vector recombination is covered by the velocity round-trip test
        // in frame_recombine.
        let t0 = 2_451_545.0_f64;
        let t1 = t0 + 100.0;
        let mid = (t0 + t1) / 2.0;

        let make_seg = |body_lon: f64, body_lat: f64, body_dist: f64| {
            Segment::new(
                Instant::new(JulianDay::from_days(t0), TimeScale::Tt),
                Instant::new(JulianDay::from_days(t1), TimeScale::Tt),
                vec![
                    // linear longitude: body_lon at x=0, body_lon+1 at x=1
                    PolynomialChannel::linear(ChannelKind::Longitude, 9, body_lon, body_lon + 1.0),
                    PolynomialChannel::linear(ChannelKind::Latitude, 9, body_lat, body_lat),
                    PolynomialChannel::linear(
                        ChannelKind::DistanceAu,
                        10,
                        body_dist,
                        body_dist,
                    ),
                ],
            )
        };

        // Sun geocentric
        let sun_seg = make_seg(95.0, 0.0, 1.0);
        // Jupiter heliocentric (approximate heliocentric position)
        let jup_seg = make_seg(120.0, 0.5, 5.2);

        let artifact = CompressedArtifact::new(
            motion_header("heliocentric-motion-smoke-test"),
            vec![
                BodyArtifact::new(CelestialBody::Sun, vec![sun_seg]),
                BodyArtifact::with_frame(
                    CelestialBody::Jupiter,
                    vec![jup_seg],
                    StoredFrame::Heliocentric,
                ),
            ],
        );

        let at = Instant::new(JulianDay::from_days(mid), TimeScale::Tt);
        let m = artifact.lookup_motion(&CelestialBody::Jupiter, at).unwrap();
        assert!(m.longitude_deg_per_day.is_some(), "longitude_deg_per_day must be Some");
        assert!(m.latitude_deg_per_day.is_some(), "latitude_deg_per_day must be Some");
        assert!(m.distance_au_per_day.is_some(), "distance_au_per_day must be Some");
        assert!(
            m.longitude_deg_per_day.unwrap().is_finite(),
            "longitude_deg_per_day must be finite"
        );
        assert!(
            m.latitude_deg_per_day.unwrap().is_finite(),
            "latitude_deg_per_day must be finite"
        );
        assert!(
            m.distance_au_per_day.unwrap().is_finite(),
            "distance_au_per_day must be finite"
        );
    }
}
