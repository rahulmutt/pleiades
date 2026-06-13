//! `SpkBackend`: a runtime `EphemerisBackend` backed by SPK kernels.

use pleiades_backend::{
    validate_request_policy, AccuracyClass, BackendCapabilities, BackendFamily, BackendId,
    BackendMetadata, BackendProvenance, CelestialBody, EphemerisBackend, EphemerisError,
    EphemerisErrorKind, EphemerisRequest, EphemerisResult, QualityAnnotation,
};
use pleiades_types::{CoordinateFrame, Motion, TimeRange, TimeScale};

use super::chain::{ecliptic_for_body, naif_ids};
use super::pool::KernelPool;
use super::{SpkError, SpkErrorKind};

/// Builder for [`SpkBackend`].
pub struct SpkBackendBuilder {
    pool: KernelPool,
    labels: Vec<String>,
}

impl SpkBackendBuilder {
    /// Starts an empty builder.
    pub fn new() -> Self {
        Self { pool: KernelPool::new(), labels: Vec::new() }
    }

    /// Adds a kernel from a path.
    pub fn add_kernel(mut self, path: impl AsRef<std::path::Path>) -> Result<Self, SpkError> {
        let p = path.as_ref().display().to_string();
        self.pool.add_path(path)?;
        self.labels.push(p);
        Ok(self)
    }

    /// Adds a kernel from raw bytes (used in tests and embedded generation).
    pub fn add_kernel_bytes(
        mut self,
        bytes: Vec<u8>,
        label: impl Into<String>,
    ) -> Result<Self, SpkError> {
        let label = label.into();
        self.pool.add_bytes(bytes, label.clone())?;
        self.labels.push(label);
        Ok(self)
    }

    /// Finalises the backend.
    pub fn build(self) -> SpkBackend {
        SpkBackend { pool: self.pool, labels: self.labels }
    }
}

impl Default for SpkBackendBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// A runtime backend reading user-supplied SPK kernels.
pub struct SpkBackend {
    pool: KernelPool,
    labels: Vec<String>,
}

impl SpkBackend {
    /// Starts a builder.
    pub fn builder() -> SpkBackendBuilder {
        SpkBackendBuilder::new()
    }

    /// The bodies in [`CelestialBody`] whose NAIF id is present in the pool.
    fn covered_bodies(&self) -> Vec<CelestialBody> {
        let present = self.pool.targets();
        let all = [
            CelestialBody::Sun,
            CelestialBody::Moon,
            CelestialBody::Mercury,
            CelestialBody::Venus,
            CelestialBody::Mars,
            CelestialBody::Jupiter,
            CelestialBody::Saturn,
            CelestialBody::Uranus,
            CelestialBody::Neptune,
            CelestialBody::Pluto,
            CelestialBody::Ceres,
            CelestialBody::Pallas,
            CelestialBody::Juno,
            CelestialBody::Vesta,
        ];
        all.into_iter()
            .filter(|b| naif_ids(b).iter().any(|id| present.contains(id)))
            .collect()
    }

    /// Dynamically detected nominal range: intersection of Sun/Earth coverage,
    /// converted from ET seconds to a Julian-Day `TimeRange`.
    fn nominal_range(&self) -> TimeRange {
        use pleiades_types::{Instant, JulianDay};
        let sun = self.pool.coverage_for_target(10);
        let emb = self.pool.coverage_for_target(3);
        match (sun, emb) {
            (Some(s), Some(e)) => {
                let start_et = s.start_et.max(e.start_et);
                let stop_et = s.stop_et.min(e.stop_et);
                let to_jd = |et: f64| JulianDay::from_days(2_451_545.0 + et / 86_400.0);
                TimeRange::new(
                    Some(Instant::new(to_jd(start_et), TimeScale::Tdb)),
                    Some(Instant::new(to_jd(stop_et), TimeScale::Tdb)),
                )
            }
            _ => TimeRange::new(None, None),
        }
    }
}

fn map_spk_error(e: SpkError) -> EphemerisError {
    let kind = match e.kind {
        SpkErrorKind::OutOfCoverage => EphemerisErrorKind::OutOfRangeInstant,
        SpkErrorKind::NoChain | SpkErrorKind::UnsupportedSegmentType => {
            EphemerisErrorKind::UnsupportedBody
        }
        SpkErrorKind::NumericalFailure => EphemerisErrorKind::NumericalFailure,
        SpkErrorKind::Io
        | SpkErrorKind::Truncated
        | SpkErrorKind::BadHeader
        | SpkErrorKind::UnknownEndianness => EphemerisErrorKind::MissingDataset,
    };
    EphemerisError::new(kind, format!("SPK backend: {}", e.message))
}

impl EphemerisBackend for SpkBackend {
    fn metadata(&self) -> BackendMetadata {
        BackendMetadata {
            id: BackendId::new("jpl-spk"),
            version: "0.1.0".to_string(),
            family: BackendFamily::ReferenceData,
            provenance: BackendProvenance {
                summary:
                    "Pure-Rust JPL DE SPK kernel reader (mean geometric, geocentric ecliptic)"
                        .to_string(),
                data_sources: self.labels.clone(),
            },
            nominal_range: self.nominal_range(),
            supported_time_scales: vec![TimeScale::Tt, TimeScale::Tdb],
            body_coverage: self.covered_bodies(),
            supported_frames: vec![CoordinateFrame::Ecliptic, CoordinateFrame::Equatorial],
            capabilities: BackendCapabilities {
                geocentric: true,
                topocentric: false,
                apparent: false,
                mean: true,
                batch: true,
                native_sidereal: false,
            },
            accuracy: AccuracyClass::High,
            deterministic: true,
            offline: true,
        }
    }

    fn supports_body(&self, body: CelestialBody) -> bool {
        let present = self.pool.targets();
        naif_ids(&body).iter().any(|id| present.contains(id))
    }

    fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        validate_request_policy(
            req,
            "the JPL SPK backend",
            &[TimeScale::Tt, TimeScale::Tdb],
            &[CoordinateFrame::Ecliptic, CoordinateFrame::Equatorial],
            true,  // geocentric supported
            false, // topocentric not supported
        )?;

        let ecliptic =
            ecliptic_for_body(&self.pool, &req.body, req.instant).map_err(map_spk_error)?;

        let mut result = EphemerisResult::new(
            BackendId::new("jpl-spk"),
            req.body.clone(),
            req.instant,
            req.frame,
            req.zodiac_mode.clone(),
            req.apparent,
        );
        result.ecliptic = Some(ecliptic);
        result.equatorial = Some(ecliptic.to_equatorial(req.instant.mean_obliquity()));
        result.motion = None::<Motion>;
        result.quality = QualityAnnotation::Exact;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spk::test_support::{build_daf, type2_record, type2_segment_data, SegmentSpec};
    use pleiades_backend::EphemerisRequest;
    use pleiades_types::{Instant, JulianDay};

    fn const_seg(target: i32, center: i32, pos: [f64; 3]) -> SegmentSpec {
        let rec = type2_record(0.0, 1.0e12, &[pos[0], 0.0], &[pos[1], 0.0], &[pos[2], 0.0]);
        let data = type2_segment_data(-1.0e12, 2.0e12, rec.len(), &[rec]);
        SegmentSpec {
            start_et: -1.0e12,
            stop_et: 1.0e12,
            target,
            center,
            frame: 1,
            data_type: 2,
            data,
            name: "C".to_string(),
        }
    }

    #[test]
    fn backend_reports_coverage_and_answers_position() {
        let blob = build_daf(&[
            const_seg(10, 0, [1.0e8, 0.0, 0.0]),
            const_seg(399, 3, [0.0, 0.0, 0.0]),
            const_seg(3, 0, [0.0, 0.0, 0.0]),
        ]);
        let backend = SpkBackend::builder()
            .add_kernel_bytes(blob, "synthetic")
            .unwrap()
            .build();
        assert!(backend.supports_body(CelestialBody::Sun));
        assert!(!backend.supports_body(CelestialBody::Pluto));
        let req = EphemerisRequest::new(
            CelestialBody::Sun,
            Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
        );
        let res = backend.position(&req).unwrap();
        assert!(res.ecliptic.is_some());
    }
}
