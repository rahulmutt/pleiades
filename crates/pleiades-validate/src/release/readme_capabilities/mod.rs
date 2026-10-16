//! README capability blocks (issue #166): each release backend's README
//! carries a block, between `<!-- audit:capabilities <backend-id> -->` and
//! `<!-- /audit:capabilities -->`, rendered from that backend's
//! `BackendMetadata`. `workspace-audit` checks the blocks, and
//! `pleiades-validate readme-capabilities` regenerates them.

mod registry;
mod render;

pub(crate) use registry::{capability_blocks, rendered_blocks, SPK_BODIES};
pub(crate) use render::{render_block_body, render_capability_lines, BodySource};

#[cfg(test)]
mod tests;
