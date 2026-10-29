use std::{
    fs,
    io::Write,
    os::windows::ffi::OsStrExt,
    path::Path,
    ptr::{null, null_mut},
};
use windows_sys::Win32::{Security::Cryptography::*, Storage::FileSystem::*};
pub fn hashes(bytes: &[u8]) -> Result<(String, String), String> {
    unsafe fn digest(bytes: &[u8], alg: &str, size: usize) -> Result<String, String> {
        let mut handle = null_mut();
        let name: Vec<u16> = alg.encode_utf16().chain(Some(0)).collect();
        let status = BCryptOpenAlgorithmProvider(&mut handle, name.as_ptr(), null(), 0);
        if status < 0 {
            return Err(format!("Hash initialization failed: {status:#x}"));
        }
        let mut out = vec![0; size];
        let status = BCryptHash(
            handle,
            null(),
            0,
            bytes.as_ptr(),
            bytes.len() as u32,
            out.as_mut_ptr(),
            size as u32,
        );
        BCryptCloseAlgorithmProvider(handle, 0);
        if status < 0 {
            Err(format!("Hash failed: {status:#x}"))
        } else {
            Ok(out.iter().map(|b| format!("{b:02x}")).collect())
        }
    }
    unsafe { Ok((digest(bytes, "MD5", 16)?, digest(bytes, "SHA256", 32)?)) }
}
pub fn atomic_write(path: &Path, data: &[u8]) -> Result<(), String> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let name = path
        .file_name()
        .ok_or("Invalid filename")?
        .to_string_lossy();
    let mut temp = None;
    for n in 0..100 {
        let candidate = parent.join(format!(".{name}.byteforge-{}-{n}.tmp", std::process::id()));
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
        {
            Ok(mut file) => {
                let r = file.write_all(data).and_then(|_| file.sync_all());
                drop(file);
                if let Err(e) = r {
                    let _ = fs::remove_file(&candidate);
                    return Err(e.to_string());
                }
                temp = Some(candidate);
                break;
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(e.to_string()),
        }
    }
    let temp = temp.ok_or("Could not create temporary save file")?;
    let w = |p: &Path| {
        p.as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect::<Vec<_>>()
    };
    let ok = unsafe {
        if path.exists() {
            ReplaceFileW(
                w(path).as_ptr(),
                w(&temp).as_ptr(),
                null(),
                0,
                null(),
                null(),
            )
        } else {
            MoveFileExW(w(&temp).as_ptr(), w(path).as_ptr(), MOVEFILE_WRITE_THROUGH)
        }
    };
    if ok == 0 {
        let error = std::io::Error::last_os_error();
        let _ = fs::remove_file(temp);
        Err(error.to_string())
    } else {
        Ok(())
    }
}
pub fn disk_changed(path: &Path, expected: Option<&str>) -> bool {
    match fs::read(path).ok().and_then(|b| hashes(&b).ok()) {
        Some((_, hash)) => Some(hash.as_str()) != expected,
        None => true,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn known_hashes() {
        assert_eq!(
            hashes(b"abc").unwrap(),
            (
                "900150983cd24fb0d6963f7d28e17f72".into(),
                "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad".into()
            )
        );
    }
    #[test]
    fn replacement_and_conflict() {
        let dir = std::env::temp_dir().join(format!("bf-storage-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let p = dir.join("test.txt");
        atomic_write(&p, b"one").unwrap();
        let hash = hashes(b"one").unwrap().1;
        assert!(!disk_changed(&p, Some(&hash)));
        atomic_write(&p, b"two").unwrap();
        assert!(disk_changed(&p, Some(&hash)));
        assert_eq!(fs::read(&p).unwrap(), b"two");
        use std::os::windows::fs::OpenOptionsExt;
        let lock = fs::OpenOptions::new()
            .read(true)
            .share_mode(0)
            .open(&p)
            .unwrap();
        assert!(atomic_write(&p, b"must not replace").is_err());
        drop(lock);
        assert_eq!(fs::read(&p).unwrap(), b"two");
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
        fs::remove_file(p).unwrap();
        fs::remove_dir(dir).unwrap();
    }
}
