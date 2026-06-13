//! Error rendering helpers.

use pleiades_core::EphemerisError;

pub(crate) fn render_error(error: EphemerisError) -> String {
    error.summary_line()
}
