//! Which README holds which backend's capability block.

use pleiades_backend::{BackendMetadata, EphemerisBackend};

use super::render::{render_block_body, render_capability_lines, BodySource};

/// `jpl-spk`'s bodies come from the kernels a caller loads, so its block
/// states that instead of listing them.
pub(crate) const SPK_BODIES: &str = "those of the loaded SPK kernels (none without one)";

/// One capability block: the README it lives in, the backend it describes,
/// and how to build that backend's metadata without the environment.
pub(crate) struct CapabilityBlock {
    /// Workspace-relative README path, with `/` separators.
    pub readme: &'static str,
    /// The backend id; also the block's key in its opening marker.
    pub backend_id: &'static str,
    /// Builds the metadata without loading kernels or reading the environment.
    pub metadata: fn() -> BackendMetadata,
    pub bodies: BodySource,
}

/// A block's expected text, rendered from the current metadata.
pub(crate) struct RenderedBlock {
    pub readme: &'static str,
    pub backend_id: &'static str,
    pub body: String,
}

pub(crate) fn capability_blocks() -> Vec<CapabilityBlock> {
    vec![
        CapabilityBlock {
            readme: "crates/pleiades-data/README.md",
            backend_id: "pleiades-data",
            metadata: || pleiades_data::PackagedDataBackend::default().metadata(),
            bodies: BodySource::Metadata,
        },
        CapabilityBlock {
            readme: "crates/pleiades-vsop87/README.md",
            backend_id: "pleiades-vsop87",
            metadata: || pleiades_vsop87::Vsop87Backend.metadata(),
            bodies: BodySource::Metadata,
        },
        CapabilityBlock {
            readme: "crates/pleiades-elp/README.md",
            backend_id: "pleiades-elp",
            metadata: || pleiades_elp::ElpBackend.metadata(),
            bodies: BodySource::Metadata,
        },
        CapabilityBlock {
            readme: "crates/pleiades-jpl/README.md",
            backend_id: "jpl-snapshot",
            metadata: || pleiades_jpl::JplSnapshotBackend.metadata(),
            bodies: BodySource::Metadata,
        },
        CapabilityBlock {
            readme: "crates/pleiades-jpl/README.md",
            backend_id: "jpl-spk",
            // No kernels: what an SPK backend serves depends on the caller's.
            metadata: || pleiades_jpl::SpkBackend::builder().build().metadata(),
            bodies: BodySource::Runtime(SPK_BODIES),
        },
    ]
}

pub(crate) fn rendered_blocks() -> Vec<RenderedBlock> {
    capability_blocks()
        .into_iter()
        .map(|block| RenderedBlock {
            readme: block.readme,
            backend_id: block.backend_id,
            body: render_block_body(&render_capability_lines(&(block.metadata)(), &block.bodies)),
        })
        .collect()
}
