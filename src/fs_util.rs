use std::io::Write;
use std::path::Path;

pub fn write_atomic(path: &Path, contents: impl AsRef<[u8]>) -> std::io::Result<()> {
    // fix symlinks so we can replace the real file and not the link
    let target = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());

    let dir = match target.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    };

    let mut temp = tempfile::NamedTempFile::new_in(dir)?;
    temp.write_all(contents.as_ref())?;

    if let Ok(metadata) = std::fs::metadata(&target) {
        let _ = temp.as_file().set_permissions(metadata.permissions());
    }

    // make sure the bytes are on disk before the rename makes them visible
    temp.as_file().sync_all()?;
    temp.persist(&target).map_err(|e| e.error)?;
    Ok(())
}
