//! All-or-nothing file output.

use std::io::{Error, ErrorKind};
use std::path::Path;

/// Run `process` against a temporary file next to `output` and rename it into
/// place only if it succeeds, so that a failed run (for example on a tampered
/// or truncated ciphertext) leaves no partial output and keeps any existing
/// file. The closure receives the path it must write to.
///
/// A newly created output is readable by its owner only (on Unix); an existing
/// output keeps its permissions. An existing output that is not a regular file
/// (a device or pipe) cannot be renamed over: `process` writes to it directly,
/// without the all-or-nothing guarantee.
///
/// # Errors
///
/// Fails if `input` cannot be resolved, if `output` is the same file as
/// `input`, if the temporary file cannot be created or renamed, or with the
/// error returned by `process`.
pub fn write_atomically<T, E: From<Error>>(
    input: &Path,
    output: &Path,
    process: impl FnOnce(&Path) -> Result<T, E>,
) -> Result<T, E> {
    let input = std::fs::canonicalize(input)
        .map_err(|e| Error::new(e.kind(), format!("failed to open input file: {e}")))?;
    let mut target = output.to_path_buf();
    let mut permissions = None;
    if let Ok(metadata) = std::fs::metadata(output) {
        if !metadata.is_file() {
            return process(output);
        }
        // Resolve symlinks so the link's destination is replaced, not the link.
        target = std::fs::canonicalize(output)?;
        if target == input {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "output must not be the same file as input",
            )
            .into());
        }
        permissions = Some(metadata.permissions());
    }

    let directory = match target.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    };
    // Removed automatically if dropped before `persist`.
    let temporary = tempfile::Builder::new()
        .prefix(".kelvin-")
        .suffix(".tmp")
        .tempfile_in(directory)
        .map_err(|e| Error::new(e.kind(), format!("failed to create output file: {e}")))?
        .into_temp_path();

    let value = process(&temporary)?;

    if let Some(permissions) = permissions {
        std::fs::set_permissions(&temporary, permissions)?;
    }
    temporary.persist(&target).map_err(|e| e.error)?;
    Ok(value)
}
