use std::fmt;
use std::path::PathBuf;
use std::sync::OnceLock;

#[cfg(feature = "packaged-artifact-path")]
use std::path::Path;

use pleiades_compression::CompressedArtifact;

pub(crate) const PACKAGED_ARTIFACT_FIXTURE: &[u8] =
    include_bytes!("../../tests/fixtures/packaged-artifact.bin");

/// Returns the checked-in packaged artifact bytes.
pub fn packaged_artifact_bytes() -> &'static [u8] {
    PACKAGED_ARTIFACT_FIXTURE
}

/// Returns the bundled packed artifact.
pub fn packaged_artifact() -> &'static CompressedArtifact {
    static ARTIFACT: OnceLock<CompressedArtifact> = OnceLock::new();
    ARTIFACT.get_or_init(crate::regenerate::build_packaged_artifact)
}

/// Decodes a packaged artifact from raw bytes.
pub fn packaged_artifact_from_bytes(
    bytes: &[u8],
) -> Result<CompressedArtifact, pleiades_compression::CompressionError> {
    let artifact = CompressedArtifact::decode(bytes)?;
    artifact.validate()?;
    Ok(artifact)
}

/// Errors that can occur while loading an external packaged artifact.
#[derive(Debug)]
pub enum PackagedArtifactLoadError {
    /// The artifact could not be read from disk.
    Io {
        /// Path that was attempted.
        path: PathBuf,
        /// The underlying I/O error.
        error: std::io::Error,
    },
    /// The artifact decoded but failed validation.
    Decode(pleiades_compression::CompressionError),
}

impl fmt::Display for PackagedArtifactLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, error } => write!(
                f,
                "failed to read packaged artifact at {}: {}",
                path.display(),
                error
            ),
            Self::Decode(error) => write!(f, "failed to decode packaged artifact: {}", error),
        }
    }
}

impl std::error::Error for PackagedArtifactLoadError {}

#[cfg(feature = "packaged-artifact-path")]
/// Loads a packaged artifact from a file path.
pub fn packaged_artifact_from_path(
    path: impl AsRef<Path>,
) -> Result<CompressedArtifact, PackagedArtifactLoadError> {
    let path = path.as_ref();
    let bytes = std::fs::read(path).map_err(|error| PackagedArtifactLoadError::Io {
        path: path.to_path_buf(),
        error,
    })?;
    packaged_artifact_from_bytes(&bytes).map_err(PackagedArtifactLoadError::Decode)
}
