use std::path::Path;

/// Set file permissions to owner read/write/execute (0o755).
/// No-op on Windows (executability is determined by file extension).
pub fn set_permissions_executable(path: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o755);
        std::fs::set_permissions(path, perms)
    }
    #[cfg(windows)]
    {
        let _ = path;
        Ok(())
    }
}

/// Atomically swap a symlink by creating a temp symlink and renaming.
///
/// On Unix: creates temp symlink, then renames over target (atomic).
/// On Windows: removes target, copies source (not atomic, but best effort).
pub fn atomic_symlink_swap(src: &Path, dst: &Path, temp: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        let _ = std::fs::remove_file(temp);
        std::os::unix::fs::symlink(src, temp)?;
        std::fs::rename(temp, dst)?;
    }
    #[cfg(windows)]
    {
        let _ = std::fs::remove_file(temp);
        // On Windows, a running .exe cannot be deleted but CAN be renamed.
        // If remove_file fails (file locked by running process), rename it away
        // first, then copy the new binary into place.
        if std::fs::remove_file(dst).is_err() {
            let stale = dst.with_extension(format!(
                "{}.old-{}",
                dst.extension().and_then(|e| e.to_str()).unwrap_or("exe"),
                std::process::id()
            ));
            let _ = std::fs::remove_file(&stale);
            if std::fs::rename(dst, &stale).is_err() {
                // Neither delete nor rename worked; propagate the copy error.
                std::fs::copy(src, dst).map(|_| ())?;
                return Ok(());
            }
        }
        std::fs::copy(src, dst).map(|_| ())?;
    }
    Ok(())
}
