//! small filesystem helpers shared across the app

use std::fs::metadata;
use std::io::Write;
use std::path::Path;

pub fn write_atomic(path: &Path, contents: impl AsRef<[u8]>) -> std::io::Result<()> {
    write_atomic_inner(path, contents.as_ref(), false)
}

/// ['write_atomic'] but for secrets such as api keys
pub fn write_private(path: &Path, contents: impl AsRef<[u8]>) -> std::io::Result<()> {
    write_atomic_inner(path, contents.as_ref(), true)
}

pub fn restrict_to_owner(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(path) {
            if metadata.permissions().mode() & 0o077 != 0 {
                let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
            }
        }
    }
    #[cfg(not(unix))]
    let _ = path;
}

fn write_atomic_inner(path: &Path, contents: &[u8], private: bool) -> std::io::Result<()> {
    // resolve symlinks so we replace the real file and not the link
    let target = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());

    let dir = match target.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    };

    let mut temp = tempfile::NamedTempFile::new_in(dir)?;
    temp.write_all(contents)?;

    if private {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            temp.as_file().set_permissions(std::fs::Permissions::from_mode(0o600))?;
        }
    } else if let Ok(metadata) = std::fs::metadata(&target) {
        // keep the mode of the file being replaced
        let _ = temp.as_file().set_permissions(metadata.permissions());
    }

    // make sure that the bytes are on disk before the rename makes them visible
    temp.as_file().sync_all()?;
    temp.persist(&target).map_err(|e| e.error)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_atomic_creates_a_new_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("new.txt");

        write_atomic(&path, "hello").unwrap();

        assert_eq!(std::fs::read_to_string(&path).unwrap(), "hello");
    }

    #[test]
    fn write_atomic_replaces_existing_content_fully() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.txt");
        std::fs::write(&path, "a much longer original body").unwrap();

        write_atomic(&path, "short").unwrap();

        assert_eq!(std::fs::read_to_string(&path).unwrap(), "short");
    }

    #[test]
    fn write_atomic_leaves_no_temp_files_behind() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.txt");

        write_atomic(&path, "x").unwrap();
        write_atomic(&path, "y").unwrap();

        let names: Vec<_> =
            std::fs::read_dir(dir.path()).unwrap().map(|e| e.unwrap().file_name()).collect();
        assert_eq!(names, vec![std::ffi::OsString::from("a.txt")]);
    }

    #[test]
    fn write_atomic_keeps_original_when_the_directory_is_missing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("missing").join("a.txt");

        assert!(write_atomic(&path, "x").is_err());
        assert!(!path.exists());
    }

    #[cfg(unix)]
    #[test]
    fn write_atomic_preserves_permissions() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("script.sh");
        std::fs::write(&path, "old").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();

        write_atomic(&path, "new").unwrap();

        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o755);
    }

    #[cfg(unix)]
    #[test]
    fn write_atomic_writes_through_a_symlink() {
        let dir = tempfile::tempdir().unwrap();
        let real = dir.path().join("real.txt");
        let link = dir.path().join("link.txt");
        std::fs::write(&real, "old").unwrap();
        std::os::unix::fs::symlink(&real, &link).unwrap();

        write_atomic(&link, "new").unwrap();

        assert!(std::fs::symlink_metadata(&link).unwrap().file_type().is_symlink());
        assert_eq!(std::fs::read_to_string(&real).unwrap(), "new");
    }

    #[cfg(unix)]
    #[test]
    fn write_private_creates_an_owner_only_file() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("providers.json");

        write_private(&path, "secret").unwrap();

        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "secret");
    }

    #[cfg(unix)]
    #[test]
    fn write_private_tightens_a_world_readable_file() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("providers.json");
        std::fs::write(&path, "old").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();

        write_private(&path, "new").unwrap();

        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }

    #[cfg(unix)]
    #[test]
    fn restrict_to_owner_fixes_loose_permissions_and_ignores_missing_files() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("wakatime.lua");
        std::fs::write(&path, "key").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();

        restrict_to_owner(&path);
        restrict_to_owner(&dir.path().join("does-not-exist"));

        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }
}
