#[cfg(not(windows))]
use std::fs::File;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use uuid::Uuid;

pub fn replace_file(path: &Path, data: &[u8]) -> io::Result<()> {
    let temporary = path.with_extension(format!("{}.tmp", Uuid::new_v4()));
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        file.write_all(data)?;
        file.sync_all()?;
        #[cfg(windows)]
        if path.exists() {
            replace_existing(&temporary, path)?;
        } else {
            fs::rename(&temporary, path)?;
        }
        #[cfg(not(windows))]
        fs::rename(&temporary, path)?;
        #[cfg(not(windows))]
        if let Some(parent) = path.parent() {
            File::open(parent)?.sync_all()?;
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(temporary);
    }
    result
}

#[cfg(windows)]
fn replace_existing(source: &Path, target: &Path) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::Storage::FileSystem::MoveFileExW;
    use windows::Win32::Storage::FileSystem::{MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH};
    use windows::core::PCWSTR;

    let src: Vec<u16> = source.as_os_str().encode_wide().chain(Some(0)).collect();
    let dst: Vec<u16> = target.as_os_str().encode_wide().chain(Some(0)).collect();
    unsafe {
        MoveFileExW(
            PCWSTR(src.as_ptr()),
            PCWSTR(dst.as_ptr()),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
        .map_err(io::Error::other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failed_replacement_leaves_target_unchanged_and_removes_temporary() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("config.json");
        fs::create_dir(&target).unwrap();
        assert!(replace_file(&target, b"replacement").is_err());
        assert!(target.is_dir());
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    }
}
