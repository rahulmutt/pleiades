//! A pool of loaded SPK kernels with `(target, center)` routing and coverage.

use std::sync::Arc;

use super::bytes::Endian;
use super::daf::{DafFile, SegmentDescriptor};
use super::segment::{evaluate, StateVector};
use super::{SpkError, SpkErrorKind};

/// Owns a kernel's bytes and parsed descriptors.
pub struct LoadedKernel {
    pub source: Arc<Vec<u8>>,
    pub endian: Endian,
    pub segments: Vec<SegmentDescriptor>,
    pub label: String,
}

/// A pool of kernels queried as one ephemeris set.
pub struct KernelPool {
    kernels: Vec<LoadedKernel>,
}

/// Inclusive time coverage `[start, stop]` in TDB seconds past J2000.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Coverage {
    pub start_et: f64,
    pub stop_et: f64,
}

impl KernelPool {
    /// Creates an empty pool.
    pub fn new() -> Self {
        Self { kernels: Vec::new() }
    }

    /// Parses and adds a kernel from raw bytes (label is for provenance/errors).
    pub fn add_bytes(&mut self, bytes: Vec<u8>, label: impl Into<String>) -> Result<(), SpkError> {
        let arc = Arc::new(bytes);
        let daf = {
            let slice: &[u8] = &arc;
            DafFile::parse(slice)?
        };
        self.kernels.push(LoadedKernel {
            source: arc,
            endian: daf.endian,
            segments: daf.segments,
            label: label.into(),
        });
        Ok(())
    }

    /// Loads a kernel from a filesystem path.
    pub fn add_path(&mut self, path: impl AsRef<std::path::Path>) -> Result<(), SpkError> {
        let path = path.as_ref();
        let bytes = std::fs::read(path)
            .map_err(|e| SpkError::new(SpkErrorKind::Io, format!("reading {}: {e}", path.display())))?;
        self.add_bytes(bytes, path.display().to_string())
    }

    /// Finds the segment covering `et` for `(target, center)`, if any.
    fn find_segment(&self, target: i32, center: i32, et: f64)
        -> Option<(&LoadedKernel, &SegmentDescriptor)> {
        for k in &self.kernels {
            for seg in &k.segments {
                if seg.target == target && seg.center == center
                    && et >= seg.start_et && et <= seg.stop_et {
                    return Some((k, seg));
                }
            }
        }
        None
    }

    /// Evaluates `(target, center)` at `et` directly (one segment, no chaining).
    pub fn state(&self, target: i32, center: i32, et: f64) -> Result<StateVector, SpkError> {
        let (k, seg) = self.find_segment(target, center, et).ok_or_else(|| {
            SpkError::new(
                SpkErrorKind::OutOfCoverage,
                format!("no segment for target {target} center {center} at et {et}"),
            )
        })?;
        let slice: &[u8] = &k.source;
        evaluate(slice, k.endian, seg, et)
    }

    /// Union coverage across all segments for `target` (any center).
    pub fn coverage_for_target(&self, target: i32) -> Option<Coverage> {
        let mut start = f64::INFINITY;
        let mut stop = f64::NEG_INFINITY;
        let mut found = false;
        for k in &self.kernels {
            for seg in &k.segments {
                if seg.target == target {
                    found = true;
                    start = start.min(seg.start_et);
                    stop = stop.max(seg.stop_et);
                }
            }
        }
        found.then_some(Coverage { start_et: start, stop_et: stop })
    }

    /// All distinct target ids present across loaded kernels.
    pub fn targets(&self) -> Vec<i32> {
        let mut ids: Vec<i32> = self.kernels.iter()
            .flat_map(|k| k.segments.iter().map(|s| s.target))
            .collect();
        ids.sort_unstable();
        ids.dedup();
        ids
    }
}

impl Default for KernelPool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spk::test_support::{build_daf, type2_record, type2_segment_data, SegmentSpec};

    fn one_segment_kernel(target: i32, start: f64, stop: f64) -> Vec<u8> {
        let rec = type2_record((start + stop) / 2.0, (stop - start) / 2.0,
            &[5.0, 0.0], &[0.0, 0.0], &[0.0, 0.0]);
        let data = type2_segment_data(start, stop - start, rec.len(), &[rec]);
        build_daf(&[SegmentSpec {
            start_et: start, stop_et: stop, target, center: 0,
            frame: 1, data_type: 2, data, name: "SEG".to_string(),
        }])
    }

    #[test]
    fn pool_routes_state_and_reports_coverage() {
        let mut pool = KernelPool::new();
        pool.add_bytes(one_segment_kernel(499, -100.0, 100.0), "k1").unwrap();
        let st = pool.state(499, 0, 0.0).unwrap();
        assert!((st.position_km[0] - 5.0).abs() < 1e-9);
        let cov = pool.coverage_for_target(499).unwrap();
        assert_eq!(cov.start_et, -100.0);
        assert_eq!(cov.stop_et, 100.0);
        assert_eq!(pool.targets(), vec![499]);
        assert_eq!(pool.state(499, 0, 999.0).unwrap_err().kind, SpkErrorKind::OutOfCoverage);
    }
}
