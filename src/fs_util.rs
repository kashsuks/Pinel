//! small filesystem helpers shared across the app

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

/// Reads `path` as UTF-8 text, returning the error message on failure
/// (missing file, no permission, or bytes that are not valid UTF-8) so the
/// caller can refuse to open it instead of showing placeholder text.
pub fn read_text_file(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| e.to_string())
}

/// Renames `from` to `to`, failing with [`std::io::ErrorKind::AlreadyExists`]
/// instead of replacing an existing file, folder or symlink at `to`.
pub fn rename_no_clobber(from: &Path, to: &Path) -> std::io::Result<()> {
    if std::fs::symlink_metadata(to).is_ok() && !is_case_only_rename(from, to)? {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("{} already exists", to.display()),
        ));
    }
    std::fs::rename(from, to)
}

/// On a case-insensitive filesystem `a.txt -> A.txt` makes `to` resolve to
/// `from` itself, so the "target exists" check would wrongly block it. It is
/// only that case when the names differ by case alone and no entry in the
/// directory is spelled exactly like `to`.
fn is_case_only_rename(from: &Path, to: &Path) -> std::io::Result<bool> {
    let (Some(from_name), Some(to_name)) = (from.file_name(), to.file_name()) else {
        return Ok(false);
    };
    if from_name == to_name
        || from_name.to_string_lossy().to_lowercase() != to_name.to_string_lossy().to_lowercase()
    {
        return Ok(false);
    }
    let dir = match to.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    };
    for entry in std::fs::read_dir(dir)? {
        if entry?.file_name() == to_name {
            return Ok(false);
        }
    }
    Ok(true)
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

    fn dir_names(dir: &Path) -> std::io::Result<Vec<String>> {
        let mut names = Vec::new();
        for entry in std::fs::read_dir(dir)? {
            names.push(entry?.file_name().to_string_lossy().into_owned());
        }
        names.sort();
        Ok(names)
    }

    #[test]
    fn test_rename_no_clobber_existing_target() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let a = dir.path().join("a.txt");
        let b = dir.path().join("b.txt");
        std::fs::write(&a, "a")?;
        std::fs::write(&b, "b")?;

        let err = rename_no_clobber(&a, &b).err().map(|e| e.kind());

        assert_eq!(err, Some(std::io::ErrorKind::AlreadyExists));
        assert_eq!(std::fs::read_to_string(&a)?, "a");
        assert_eq!(std::fs::read_to_string(&b)?, "b");
        Ok(())
    }

    #[test]
    fn test_rename_no_clobber_new_name() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let a = dir.path().join("a.txt");
        let c = dir.path().join("c.txt");
        std::fs::write(&a, "a")?;

        rename_no_clobber(&a, &c)?;

        assert!(std::fs::symlink_metadata(&a).is_err());
        assert_eq!(std::fs::read_to_string(&c)?, "a");
        Ok(())
    }

    #[test]
    fn test_rename_no_clobber_existing_directory() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let x = dir.path().join("x");
        let y = dir.path().join("y");
        std::fs::create_dir(&x)?;
        std::fs::create_dir(&y)?;

        let err = rename_no_clobber(&x, &y).err().map(|e| e.kind());

        assert_eq!(err, Some(std::io::ErrorKind::AlreadyExists));
        assert!(x.is_dir());
        assert!(y.is_dir());
        Ok(())
    }

    #[test]
    fn test_rename_no_clobber_case_only_change() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let from = dir.path().join("a.txt");
        std::fs::write(&from, "a")?;

        rename_no_clobber(&from, &dir.path().join("A.txt"))?;

        assert_eq!(dir_names(dir.path())?, vec!["A.txt".to_string()]);
        Ok(())
    }

    #[test]
    fn test_rename_no_clobber_case_change_onto_other_file() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let a = dir.path().join("a.txt");
        let b = dir.path().join("b.txt");
        let upper_b = dir.path().join("B.txt");
        std::fs::write(&a, "a")?;
        std::fs::write(&b, "b")?;
        let case_insensitive = std::fs::symlink_metadata(&upper_b).is_ok();

        let result = rename_no_clobber(&a, &upper_b);

        if case_insensitive {
            assert_eq!(
                result.err().map(|e| e.kind()),
                Some(std::io::ErrorKind::AlreadyExists)
            );
            assert_eq!(std::fs::read_to_string(&a)?, "a");
        } else {
            result?;
            assert_eq!(std::fs::read_to_string(&upper_b)?, "a");
        }
        assert_eq!(std::fs::read_to_string(&b)?, "b");
        Ok(())
    }

    #[test]
    fn test_read_text_file_rejects_invalid_utf8() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("binary.pdf");
        let bytes = [0x50, 0x44, 0xFF, 0xFE];
        std::fs::write(&path, bytes)?;

        assert!(read_text_file(&path).is_err());
        assert_eq!(std::fs::read(&path)?, bytes);
        Ok(())
    }

    #[test]
    fn test_read_text_file_reads_utf8() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("hello.txt");
        std::fs::write(&path, "héllo\n")?;

        assert_eq!(read_text_file(&path), Ok("héllo\n".to_string()));
        Ok(())
    }
}
