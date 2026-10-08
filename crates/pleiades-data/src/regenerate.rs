use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};

use pleiades_backend::{
    Angle, CelestialBody, CustomBodyId, EclipticCoordinates, EphemerisBackend, EphemerisError,
    EphemerisErrorKind, EphemerisRequest, Instant, JulianDay, TimeRange, TimeScale,
};
use pleiades_compression::{
    join_display, ArtifactHeader, BodyArtifact, ChannelKind, CompressedArtifact, PolynomialChannel,
    Segment,
};
use pleiades_jpl::SnapshotEntry;

use crate::coverage::{packaged_artifact_body_cadence, PackagedArtifactBodyCadence};
use crate::data::{packaged_artifact_bytes, packaged_artifact_from_bytes};
use crate::{
    packaged_artifact_source_text, packaged_asteroids, packaged_bodies, ARTIFACT_LABEL, AU_IN_KM,
};

pub(crate) fn build_packaged_artifact() -> CompressedArtifact {
    packaged_artifact_from_bytes(packaged_artifact_bytes())
        .expect("checked-in packaged artifact fixture should decode and validate")
}

/// Returns the packaged artifact for kernel-free callers.
///
/// Runtime decode of the committed bytes is the only kernel-free path: this
/// decodes [`packaged_artifact_bytes()`] (via the crate-internal
/// `build_packaged_artifact` helper). Kernel-gated regeneration from de440 and
/// sb441-n373s lives in [`regenerate_packaged_artifact_from_kernels`].
pub fn regenerate_packaged_artifact() -> CompressedArtifact {
    build_packaged_artifact()
}

/// Returns the encoded bytes for the packaged artifact for kernel-free callers.
///
/// This returns the committed bytes directly ([`packaged_artifact_bytes()`]) so
/// kernel-free regeneration commands write the byte-identical committed payload.
pub fn regenerate_packaged_artifact_bytes() -> &'static [u8] {
    packaged_artifact_bytes()
}

/// Bodies fit in the heliocentric frame and recombined with the geocentric Sun
/// at lookup: the eight true planets and the dense asteroids. Sun, Moon, and
/// lunar points stay geocentric.
pub(crate) fn body_uses_heliocentric_frame(body: &CelestialBody) -> bool {
    matches!(
        body,
        CelestialBody::Mercury
            | CelestialBody::Venus
            | CelestialBody::Mars
            | CelestialBody::Jupiter
            | CelestialBody::Saturn
            | CelestialBody::Uranus
            | CelestialBody::Neptune
            | CelestialBody::Pluto
    ) || packaged_asteroids().contains(body)
}

pub(crate) fn body_segment_span_limit(body: &CelestialBody) -> f64 {
    match packaged_artifact_body_cadence(body) {
        PackagedArtifactBodyCadence::Luminaries => 256.0,
        PackagedArtifactBodyCadence::InnerPlanets => 384.0,
        PackagedArtifactBodyCadence::OuterPlanets => 768.0,
        PackagedArtifactBodyCadence::Pluto => 1_536.0,
        PackagedArtifactBodyCadence::LunarPoints => 256.0,
        PackagedArtifactBodyCadence::SelectedAsteroids => 256.0,
        PackagedArtifactBodyCadence::CustomBodies => 512.0,
    }
}

pub(crate) fn packaged_artifact_segment_validation_fractions_for_body(
    body: &CelestialBody,
) -> &'static [f64] {
    if packaged_artifact_body_cadence(body).uses_dense_validation_sampling() {
        PACKAGED_ARTIFACT_DENSE_VALIDATION_SAMPLE_FRACTIONS
    } else {
        PACKAGED_ARTIFACT_MEDIUM_VALIDATION_SAMPLE_FRACTIONS
    }
}

fn packaged_artifact_body_class_span_cap_entries() -> Vec<(&'static str, f64)> {
    vec![
        ("luminaries", body_segment_span_limit(&CelestialBody::Sun)),
        (
            "inner planets",
            body_segment_span_limit(&CelestialBody::Mercury),
        ),
        (
            "outer planets",
            body_segment_span_limit(&CelestialBody::Jupiter),
        ),
        ("pluto", body_segment_span_limit(&CelestialBody::Pluto)),
        (
            "lunar points",
            body_segment_span_limit(&CelestialBody::MeanNode),
        ),
        (
            "selected asteroids",
            body_segment_span_limit(&CelestialBody::Ceres),
        ),
        (
            "custom bodies",
            body_segment_span_limit(&CelestialBody::Custom(CustomBodyId::new(
                "catalog",
                "designation",
            ))),
        ),
    ]
}

/// Structured summary for the packaged-artifact body-class span caps.
#[derive(Clone, Debug, PartialEq)]
pub struct PackagedArtifactBodyClassSpanCapSummary {
    /// Body-class span cap entries in release-facing order.
    pub entries: Vec<(&'static str, f64)>,
}

/// Validation error for a packaged-artifact body-class span cap summary that drifted from the current posture.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PackagedArtifactBodyClassSpanCapSummaryValidationError {
    /// A summary field is out of sync with the current packaged-artifact posture.
    FieldOutOfSync { field: &'static str },
}

impl PackagedArtifactBodyClassSpanCapSummaryValidationError {
    /// Returns the compact release-facing summary for the validation error.
    pub fn summary_line(&self) -> String {
        match self {
            Self::FieldOutOfSync { field } => format!(
                "the packaged artifact body-class span cap summary field `{field}` is out of sync with the current posture"
            ),
        }
    }
}

impl fmt::Display for PackagedArtifactBodyClassSpanCapSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl std::error::Error for PackagedArtifactBodyClassSpanCapSummaryValidationError {}

impl PackagedArtifactBodyClassSpanCapSummary {
    /// Returns the body-class span cap summary as a compact human-readable line.
    pub fn summary_line(&self) -> String {
        format!("body-class span caps: {}", self.entries_summary_line())
    }

    fn entries_summary_line(&self) -> String {
        let entries = self
            .entries
            .iter()
            .map(|(label, days)| format!("{label}={days:.0} days"))
            .collect::<Vec<_>>();

        join_display(&entries)
    }

    /// Returns the validated body-class span cap summary as a compact human-readable line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, PackagedArtifactBodyClassSpanCapSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Returns `Ok(())` when the summary still matches the current packaged-artifact posture.
    pub fn validate(&self) -> Result<(), PackagedArtifactBodyClassSpanCapSummaryValidationError> {
        if self.entries != packaged_artifact_body_class_span_cap_entries() {
            return Err(
                PackagedArtifactBodyClassSpanCapSummaryValidationError::FieldOutOfSync {
                    field: "entries",
                },
            );
        }

        Ok(())
    }
}

impl fmt::Display for PackagedArtifactBodyClassSpanCapSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the current packaged-artifact body-class span caps summary record.
pub fn packaged_artifact_body_class_span_cap_summary_details(
) -> PackagedArtifactBodyClassSpanCapSummary {
    let summary = PackagedArtifactBodyClassSpanCapSummary {
        entries: packaged_artifact_body_class_span_cap_entries(),
    };
    debug_assert!(summary.validate().is_ok());
    summary
}

fn packaged_artifact_body_cadence_counts() -> [(&'static str, usize); 7] {
    let mut counts = [0usize; 7];

    for body in packaged_bodies() {
        match packaged_artifact_body_cadence(body) {
            PackagedArtifactBodyCadence::Luminaries => counts[0] += 1,
            PackagedArtifactBodyCadence::InnerPlanets => counts[1] += 1,
            PackagedArtifactBodyCadence::OuterPlanets => counts[2] += 1,
            PackagedArtifactBodyCadence::Pluto => counts[3] += 1,
            PackagedArtifactBodyCadence::LunarPoints => counts[4] += 1,
            PackagedArtifactBodyCadence::SelectedAsteroids => counts[5] += 1,
            PackagedArtifactBodyCadence::CustomBodies => counts[6] += 1,
        }
    }

    [
        ("luminaries", counts[0]),
        ("inner planets", counts[1]),
        ("outer planets", counts[2]),
        ("pluto", counts[3]),
        ("lunar points", counts[4]),
        ("selected asteroids", counts[5]),
        ("custom bodies", counts[6]),
    ]
}

/// Structured summary for the packaged-artifact body cadence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackagedArtifactBodyCadenceSummary {
    /// Body-cadence entries in release-facing order.
    pub entries: Vec<(&'static str, usize)>,
}

/// Validation error for a packaged-artifact body cadence summary that drifted from the current posture.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PackagedArtifactBodyCadenceSummaryValidationError {
    /// A summary field is out of sync with the current packaged-artifact posture.
    FieldOutOfSync { field: &'static str },
}

impl PackagedArtifactBodyCadenceSummaryValidationError {
    /// Returns the compact release-facing summary for the validation error.
    pub fn summary_line(&self) -> String {
        match self {
            Self::FieldOutOfSync { field } => format!(
                "the packaged artifact body cadence summary field `{field}` is out of sync with the current posture"
            ),
        }
    }
}

impl fmt::Display for PackagedArtifactBodyCadenceSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl std::error::Error for PackagedArtifactBodyCadenceSummaryValidationError {}

impl PackagedArtifactBodyCadenceSummary {
    /// Returns the body cadence summary as a compact human-readable line.
    pub fn summary_line(&self) -> String {
        let entries = self
            .entries
            .iter()
            .map(|(label, count)| {
                format!(
                    "{label}={count} {}",
                    if *count == 1 { "body" } else { "bodies" }
                )
            })
            .collect::<Vec<_>>();

        format!("body cadence: {}", join_display(&entries))
    }

    /// Returns `Ok(())` when the summary still matches the current packaged-artifact posture.
    pub fn validate(&self) -> Result<(), PackagedArtifactBodyCadenceSummaryValidationError> {
        if self.entries != packaged_artifact_body_cadence_counts().to_vec() {
            return Err(
                PackagedArtifactBodyCadenceSummaryValidationError::FieldOutOfSync {
                    field: "entries",
                },
            );
        }

        Ok(())
    }

    /// Returns the summary line after validating the structured posture.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, PackagedArtifactBodyCadenceSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for PackagedArtifactBodyCadenceSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the current packaged-artifact body cadence summary record.
pub fn packaged_artifact_body_cadence_summary_details() -> PackagedArtifactBodyCadenceSummary {
    let summary = PackagedArtifactBodyCadenceSummary {
        entries: packaged_artifact_body_cadence_counts().to_vec(),
    };
    debug_assert!(summary.validate().is_ok());
    summary
}

fn unwrap_longitude_degrees(reference_degrees: f64, candidate_degrees: f64) -> f64 {
    reference_degrees
        + Angle::from_degrees(candidate_degrees - reference_degrees)
            .normalized_signed()
            .degrees()
}

pub(crate) const PACKAGED_ARTIFACT_MEDIUM_VALIDATION_SAMPLE_FRACTIONS: &[f64] =
    &[0.125, 0.25, 0.5, 0.75, 0.875];
pub(crate) const PACKAGED_ARTIFACT_DENSE_VALIDATION_SAMPLE_FRACTIONS: &[f64] =
    &[0.125, 0.25, 0.375, 0.5, 0.625, 0.75, 0.875];
fn unwrap_longitude_samples(samples: &[f64]) -> Vec<f64> {
    let mut unwrapped = Vec::with_capacity(samples.len());

    for &sample in samples {
        if let Some(&previous) = unwrapped.last() {
            unwrapped.push(unwrap_longitude_degrees(previous, sample));
        } else {
            unwrapped.push(sample);
        }
    }

    unwrapped
}

pub(crate) fn coordinates(entry: &SnapshotEntry) -> EclipticCoordinates {
    let radius_km =
        (entry.x_km * entry.x_km + entry.y_km * entry.y_km + entry.z_km * entry.z_km).sqrt();
    let longitude = entry.y_km.atan2(entry.x_km).to_degrees();
    let latitude = (entry.z_km / radius_km)
        .clamp(-1.0, 1.0)
        .asin()
        .to_degrees();
    EclipticCoordinates::new(
        pleiades_backend::Longitude::from_degrees(longitude),
        pleiades_backend::Latitude::from_degrees(latitude),
        Some(radius_km / AU_IN_KM),
    )
}

pub(crate) fn artifact_time_range(artifact: &CompressedArtifact) -> TimeRange {
    let mut start: Option<Instant> = None;
    let mut end: Option<Instant> = None;
    for body in &artifact.bodies {
        for segment in &body.segments {
            start = Some(match start {
                Some(current) => {
                    if segment.start.julian_day.days() < current.julian_day.days() {
                        segment.start
                    } else {
                        current
                    }
                }
                None => segment.start,
            });
            end = Some(match end {
                Some(current) => {
                    if segment.end.julian_day.days() > current.julian_day.days() {
                        segment.end
                    } else {
                        current
                    }
                }
                None => segment.end,
            });
        }
    }
    TimeRange::new(start, end)
}

pub(crate) fn normalize_lookup_instant(instant: Instant) -> Instant {
    match instant.scale {
        TimeScale::Tt => instant,
        TimeScale::Tdb => Instant::new(instant.julian_day, TimeScale::Tt),
        _ => instant,
    }
}

pub(crate) fn map_artifact_error(error: pleiades_compression::CompressionError) -> EphemerisError {
    let kind = match error.kind {
        pleiades_compression::CompressionErrorKind::MissingBody => {
            EphemerisErrorKind::UnsupportedBody
        }
        pleiades_compression::CompressionErrorKind::OutOfRangeInstant => {
            EphemerisErrorKind::OutOfRangeInstant
        }
        pleiades_compression::CompressionErrorKind::UnsupportedTimeScale => {
            EphemerisErrorKind::UnsupportedTimeScale
        }
        pleiades_compression::CompressionErrorKind::MissingChannel => {
            EphemerisErrorKind::MissingDataset
        }
        pleiades_compression::CompressionErrorKind::QuantizationOverflow
        | pleiades_compression::CompressionErrorKind::InvalidFormat
        | pleiades_compression::CompressionErrorKind::UnsupportedEndianPolicy
        | pleiades_compression::CompressionErrorKind::InvalidMagic
        | pleiades_compression::CompressionErrorKind::UnsupportedVersion
        | pleiades_compression::CompressionErrorKind::ChecksumMismatch
        | pleiades_compression::CompressionErrorKind::Truncated
        | _ => EphemerisErrorKind::NumericalFailure,
    };

    EphemerisError::new(kind, error.message)
}

/// Fits one segment over `[t0_jd, t1_jd]` by sampling `reference` (de440 or a
/// test backend) at the body's within-span sample count and least-squares
/// fitting Longitude/Latitude/DistanceAu channels over the normalized interval.
///
/// The x-domain for the polynomial fit matches the decoder: `x = (t - t0) / span`
/// in `[0, 1]`, consistent with `CompressedArtifact::lookup_ecliptic` (artifact.rs).
///
/// Scale exponents match the existing generation pipeline: Longitude=9, Latitude=9,
/// DistanceAu=10 (see `regenerate.rs` segment_from_single_entry and threshold.rs).
///
/// # Errors
///
/// Returns why the segment could not be fit: an empty span, a sample the
/// reference refuses or returns without an ecliptic position or distance, or a
/// fit that is singular or not finite. The message does not name the body or
/// span; [`fit_dense_body_artifact`] adds them.
pub(crate) fn fit_segment_within_span(
    body: &CelestialBody,
    t0_jd: f64,
    t1_jd: f64,
    reference: &dyn EphemerisBackend,
) -> Result<Segment, String> {
    use crate::coverage::{fit_polynomial_lsq, fitting_degree, fitting_within_span_sample_count};

    let n = fitting_within_span_sample_count(body).max(fitting_degree(body) + 1);
    let span = t1_jd - t0_jd;
    if span <= 0.0 {
        return Err(format!("empty span of {span} days"));
    }
    if n < 2 {
        return Err(format!("{n} samples are too few to fit"));
    }

    let mut xs = Vec::with_capacity(n);
    let mut lon_deg = Vec::with_capacity(n);
    let mut lat = Vec::with_capacity(n);
    let mut dist = Vec::with_capacity(n);
    for i in 0..n {
        let frac = i as f64 / (n as f64 - 1.0);
        let jd = t0_jd + frac * span;
        let inst = Instant::new(JulianDay::from_days(jd), TimeScale::Tdb);
        let ec = sample_ecliptic(reference, body, inst)?;
        let ec = if body_uses_heliocentric_frame(body) {
            let sun = sample_ecliptic(reference, &CelestialBody::Sun, inst)?;
            pleiades_compression::heliocentric_from_geocentric(&ec, &sun)
                .ok_or_else(|| format!("heliocentric conversion failed at JD {jd} (TDB)"))?
        } else {
            ec
        };
        xs.push(frac);
        lon_deg.push(ec.longitude.degrees());
        lat.push(ec.latitude.degrees());
        dist.push(
            ec.distance_au
                .ok_or_else(|| format!("the reference gave no distance at JD {jd} (TDB)"))?,
        );
    }

    // Unwrap longitude to a continuous series before fitting (reuse existing helper).
    let lon_unwrapped = unwrap_longitude_samples(&lon_deg);

    let degree = fitting_degree(body);
    let to_samples =
        |ys: &[f64]| -> Vec<(f64, f64)> { xs.iter().copied().zip(ys.iter().copied()).collect() };

    let fit = |channel: &str, ys: &[f64]| {
        fit_polynomial_lsq(&to_samples(ys), degree)
            .ok_or_else(|| format!("the degree-{degree} {channel} fit is singular"))
    };
    let lon_coeffs = fit("longitude", &lon_unwrapped)?;
    let lat_coeffs = fit("latitude", &lat)?;
    let dist_coeffs = fit("distance", &dist)?;

    // Channels must be ordered by ChannelKind discriminant: Longitude=0, Latitude=1, DistanceAu=2.
    // Scale exponents match the existing generation pipeline (Longitude=9, Latitude=9, DistanceAu=10).
    let channels = vec![
        PolynomialChannel::new(ChannelKind::Longitude, 9, lon_coeffs),
        PolynomialChannel::new(ChannelKind::Latitude, 9, lat_coeffs),
        PolynomialChannel::new(ChannelKind::DistanceAu, 10, dist_coeffs),
    ];

    // Validate each channel's coefficients are finite (fail-closed).
    for channel in &channels {
        channel.validate().map_err(|error| error.to_string())?;
    }

    // Segment boundaries are tagged Tt to match the packaged-lookup convention:
    // normalize_lookup_instant re-tags every query to Tt, and Segment::contains
    // requires matching scales. The sampling instants above remain Tdb (the
    // physical ephemeris query scale); the ~2 ms TT/TDB difference is immaterial
    // because the stored artifact only carries the boundary tag, not a converted value.
    let seg = Segment::new(
        Instant::new(JulianDay::from_days(t0_jd), TimeScale::Tt),
        Instant::new(JulianDay::from_days(t1_jd), TimeScale::Tt),
        channels,
    );
    Ok(seg)
}

/// The geocentric ecliptic position of `body` at `instant`, or why `reference`
/// cannot give one.
fn sample_ecliptic(
    reference: &dyn EphemerisBackend,
    body: &CelestialBody,
    instant: Instant,
) -> Result<EclipticCoordinates, String> {
    let jd = instant.julian_day.days();
    reference
        .position(&EphemerisRequest::new(body.clone(), instant))
        .map_err(|error| format!("the reference refused {body} at JD {jd} (TDB): {error}"))?
        .ecliptic
        .ok_or_else(|| {
            format!("the reference gave no ecliptic position for {body} at JD {jd} (TDB)")
        })
}

/// Fits `body` densely from `reference` over `window`, one segment per
/// [`fitting_segment_boundaries`] span, in the body's stored frame.
///
/// Gives up before the next segment once `cancelled` is set, so one body's
/// failure stops the other bodies' fits.
///
/// # Errors
///
/// When `reference` cannot serve `body` somewhere in `window`, naming the body
/// and the segment span that failed, or when `cancelled` is set.
pub(crate) fn fit_dense_body_artifact(
    body: &CelestialBody,
    window: (f64, f64),
    reference: &dyn EphemerisBackend,
    cancelled: &AtomicBool,
) -> Result<BodyArtifact, String> {
    use crate::coverage::fitting_segment_boundaries;
    let (start_jd, end_jd) = window;
    let mut segments = Vec::new();
    for (t0, t1) in fitting_segment_boundaries(body, start_jd, end_jd) {
        if cancelled.load(Ordering::Relaxed) {
            return Err(format!("the fit of {body} was cancelled"));
        }
        let segment = fit_segment_within_span(body, t0, t1, reference)
            .map_err(|reason| format!("cannot fit {body} over JD [{t0}, {t1}]: {reason}"))?;
        segments.push(segment);
    }
    let frame = if body_uses_heliocentric_frame(body) {
        pleiades_compression::StoredFrame::Heliocentric
    } else {
        pleiades_compression::StoredFrame::Geocentric
    };
    Ok(BodyArtifact::with_frame(body.clone(), segments, frame))
}

/// Core artifact builder parameterised by an explicit coverage window.
///
/// Every packaged body (the Sun, Moon, Mercury through Pluto and the five
/// asteroids) is fit densely from `reference` over `window` using
/// [`fitting_segment_boundaries`] + [`fit_segment_within_span`]. A tiny
/// synthetic window lets the kernel-free unit tests run in milliseconds instead
/// of the minutes a full 1900-2100 build takes.
///
/// # Errors
///
/// When `reference` cannot serve a packaged body somewhere in `window`. The
/// first body to fail cancels the others, and its error is returned.
pub(crate) fn build_packaged_artifact_from_reference_over(
    reference: &dyn EphemerisBackend,
    window: (f64, f64),
) -> Result<CompressedArtifact, String> {
    let cancelled = AtomicBool::new(false);
    let mut bodies = Vec::with_capacity(packaged_bodies().len());
    let mut first_failure = None;

    std::thread::scope(|scope| {
        let cancelled = &cancelled;
        let handles: Vec<_> = packaged_bodies()
            .iter()
            .map(|body| {
                scope.spawn(move || {
                    let fitted = fit_dense_body_artifact(body, window, reference, cancelled);
                    // Only the failure that raises the flag is the cause; the
                    // bodies it cancels fail with a cancellation message.
                    let is_first_failure =
                        fitted.is_err() && !cancelled.swap(true, Ordering::Relaxed);
                    (fitted, is_first_failure)
                })
            })
            .collect();

        // Joined in packaged-body order, so `bodies` keeps that order.
        for handle in handles {
            match handle.join() {
                Ok((Ok(body_artifact), _)) => bodies.push(body_artifact),
                Ok((Err(error), true)) => first_failure = Some(error),
                Ok((Err(_), false)) => {}
                Err(panic) => std::panic::resume_unwind(panic),
            }
        }
    });

    if let Some(error) = first_failure {
        return Err(error);
    }

    let mut artifact = CompressedArtifact::new(
        ArtifactHeader::new(ARTIFACT_LABEL, packaged_artifact_source_text()),
        bodies,
    );
    artifact.checksum = artifact
        .checksum()
        .map_err(|error| format!("packaged artifact checksum: {error}"))?;
    artifact
        .validate()
        .map_err(|error| format!("packaged artifact validation: {error}"))?;
    Ok(artifact)
}

/// Regenerates the packaged artifact from the de440 planetary kernel and the
/// JPL `sb441-n373s` small-body kernel over an explicit coverage window.
/// Every body, asteroids included, is fit densely across `window`.
///
/// # Errors
///
/// When a kernel fails to load, or when the kernels cannot serve a packaged
/// body somewhere in `window` (for example an asteroid kernel without the
/// packaged asteroids, or a window outside the kernels' coverage). The message
/// names the body and the segment span that failed.
pub fn regenerate_packaged_artifact_from_kernels_over(
    de_kernel: &str,
    asteroid_kernel: &str,
    window: pleiades_jpl::spk::corpus_spec::CoverageWindow,
) -> Result<CompressedArtifact, String> {
    let backend = pleiades_jpl::SpkBackend::builder()
        .add_kernel(de_kernel)
        .map_err(|error| error.message)?
        .add_kernel(asteroid_kernel)
        .map_err(|error| error.message)?
        .build();
    build_packaged_artifact_from_reference_over(&backend, window.as_tuple())
}

/// [`regenerate_packaged_artifact_from_kernels_over`] over the shipped default
/// window (1900-2100).
pub fn regenerate_packaged_artifact_from_kernels(
    de_kernel: &str,
    asteroid_kernel: &str,
) -> Result<CompressedArtifact, String> {
    regenerate_packaged_artifact_from_kernels_over(
        de_kernel,
        asteroid_kernel,
        pleiades_jpl::spk::corpus_spec::CoverageWindow::default(),
    )
}
