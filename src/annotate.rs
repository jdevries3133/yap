//! (not yet implemented) annotate a source-code file based on a prompt
//! request.

use log::info;

/// To annotate, we want to take a file + context + prompt as input, and
/// require the LLM to give us a response like;
///
/// ```text
/// <file>:<lineno> "... feedback ..."
/// ```
///
/// Given that response, we can modify the file in-place to insert all of the
/// feedback.
pub fn annotate() {
    info!("annotating...");
}
