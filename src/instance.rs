//! Same-desktop file-open handoff. The session lock still has exactly one owner.
use crate::{document::MAX_TABS, storage};
use std::{
    cell::RefCell,
    collections::VecDeque,
    ffi::OsString,
    os::windows::ffi::{OsStrExt, OsStringExt},
    path::{Path, PathBuf},
    ptr::null_mut,
    time::{Duration, Instant},
};
use windows_sys::Win32::{Foundation::*, UI::WindowsAndMessaging::*};

const PROTOCOL: usize = 0x4246_4F31;
const MAX_PACKET: usize = 1024 * 1024;
const MAX_QUEUED: usize = 128;
const CLASS: &str = "ByteForgeRustWindow";
thread_local! {
    static PROPERTY:RefCell<Vec<u16>>=const{RefCell::new(Vec::new())};
    static REQUESTS:RefCell<VecDeque<Request>>=const{RefCell::new(VecDeque::new())};
}
#[derive(Debug)]
pub struct Request {
    pub paths: Vec<PathBuf>,
    pub activate: bool,
}
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}
fn property(dir: &Path) -> Result<Vec<u16>, String> {
    let dir =
        std::fs::canonicalize(dir).map_err(|e| format!("Cannot resolve session directory: {e}"))?;
    let mut name: Vec<u16> = dir.as_os_str().encode_wide().collect();
    unsafe {
        CharUpperBuffW(name.as_mut_ptr(), name.len() as u32);
    }
    let bytes: Vec<u8> = name.into_iter().flat_map(u16::to_le_bytes).collect();
    Ok(wide(&format!(
        "ByteForgeRust.Session.{}",
        storage::hashes(&bytes)?.1
    )))
}
/// # Safety
/// `hwnd` must be this UI thread's live ByteForge window; install only once.
pub unsafe fn install(hwnd: HWND, dir: &Path) -> Result<(), String> {
    let name = property(dir)?;
    if SetPropW(hwnd, name.as_ptr(), PROTOCOL as HANDLE) == 0 {
        return Err("Cannot register the file-open endpoint".into());
    }
    PROPERTY.with(|p| *p.borrow_mut() = name);
    Ok(())
}
/// # Safety
/// Call on the window's owning thread before the registered HWND is destroyed.
pub unsafe fn uninstall(hwnd: HWND) {
    PROPERTY.with(|p| {
        let name = std::mem::take(&mut *p.borrow_mut());
        if !name.is_empty() {
            RemovePropW(hwnd, name.as_ptr());
        }
    });
}
/// Resolve relative paths in the sender, never in the receiver's working folder.
pub fn command_paths(args: &[OsString]) -> Result<Vec<PathBuf>, String> {
    let mut out = Vec::new();
    let mut skip = false;
    let mut literal = false;
    for arg in args {
        if skip {
            skip = false;
            continue;
        }
        if !literal && arg == "--" {
            literal = true;
            continue;
        }
        if !literal
            && [
                "--session-dir",
                "--ui-self-test",
                "--verify-recovery",
                "--verify-explorer",
            ]
            .iter()
            .any(|flag| arg == *flag)
        {
            skip = true;
            continue;
        }
        if !literal && arg.to_string_lossy().starts_with("--") {
            continue;
        }
        out.push(std::path::absolute(Path::new(arg)).map_err(|e| e.to_string())?);
    }
    Ok(out)
}
fn encode(paths: &[PathBuf], activate: bool) -> Result<Vec<u8>, String> {
    if paths.len() > MAX_TABS {
        return Err("Too many files in one open request (maximum 128)".into());
    }
    let mut out = Vec::new();
    out.extend((paths.len() as u32).to_le_bytes());
    out.extend((activate as u32).to_le_bytes());
    for path in paths {
        let p: Vec<u16> = path.as_os_str().encode_wide().collect();
        if !path.is_absolute() || p.is_empty() || p.len() > 32767 || p.contains(&0) {
            return Err("Invalid file-open path".into());
        }
        out.extend((p.len() as u32).to_le_bytes());
        out.extend(p.into_iter().flat_map(u16::to_le_bytes));
        if out.len() > MAX_PACKET {
            return Err("File-open request exceeds its size limit".into());
        }
    }
    Ok(out)
}
fn decode(bytes: &[u8]) -> Option<Request> {
    if bytes.len() < 8 || bytes.len() > MAX_PACKET {
        return None;
    }
    let count = u32::from_le_bytes(bytes[..4].try_into().ok()?) as usize;
    let flags = u32::from_le_bytes(bytes[4..8].try_into().ok()?);
    if count > MAX_TABS || flags > 1 {
        return None;
    }
    let mut at = 8usize;
    let mut paths = Vec::new();
    for _ in 0..count {
        let n = u32::from_le_bytes(bytes.get(at..at + 4)?.try_into().ok()?) as usize;
        at += 4;
        if n == 0 || n > 32767 {
            return None;
        }
        let end = at.checked_add(n * 2)?;
        let p: Vec<u16> = bytes
            .get(at..end)?
            .chunks_exact(2)
            .map(|b| u16::from_le_bytes([b[0], b[1]]))
            .collect();
        at = end;
        if p.contains(&0) {
            return None;
        }
        let path = PathBuf::from(OsString::from_wide(&p));
        if !path.is_absolute() {
            return None;
        }
        paths.push(path);
    }
    (at == bytes.len()).then_some(Request {
        paths,
        activate: flags != 0,
    })
}
/// WM_COPYDATA memory expires when SendMessage returns: validate and copy here.
/// Do not open files or enter a modal dialog while the sender is waiting.
/// # Safety
/// `lp` must be the valid COPYDATASTRUCT and buffer supplied by WM_COPYDATA,
/// and `hwnd` must be its receiving window on the current UI thread.
pub unsafe fn receive(hwnd: HWND, lp: isize, wake: u32) -> isize {
    use windows_sys::Win32::System::DataExchange::COPYDATASTRUCT;
    if lp == 0 {
        return 0;
    }
    let packet = &*(lp as *const COPYDATASTRUCT);
    if packet.dwData != PROTOCOL || packet.lpData.is_null() || packet.cbData as usize > MAX_PACKET {
        return 0;
    }
    let bytes = std::slice::from_raw_parts(packet.lpData as *const u8, packet.cbData as usize);
    let Some(request) = decode(bytes) else {
        return 0;
    };
    REQUESTS.with(|q| {
        let mut q = q.borrow_mut();
        if q.len() >= MAX_QUEUED {
            return 0;
        }
        q.push_back(request);
        if PostMessageW(hwnd, wake, 0, 0) == 0 {
            q.pop_back();
            return 0;
        }
        1
    })
}
pub fn take() -> Option<Request> {
    REQUESTS.with(|q| q.borrow_mut().pop_front())
}
struct Search {
    name: *const u16,
    found: HWND,
}
unsafe extern "system" fn find(hwnd: HWND, lp: isize) -> i32 {
    let search = &mut *(lp as *mut Search);
    let mut class = [0u16; 64];
    let n = GetClassNameW(hwnd, class.as_mut_ptr(), 64);
    if n > 0
        && String::from_utf16_lossy(&class[..n as usize]) == CLASS
        && GetPropW(hwnd, search.name) == PROTOCOL as HANDLE
    {
        search.found = hwnd;
        return 0;
    }
    1
}
/// # Safety
/// Call from a Windows desktop thread where synchronous window messaging is
/// permitted. This can reenter a window procedure on the same thread.
pub unsafe fn forward(dir: &Path, paths: &[PathBuf], activate: bool) -> Result<(), String> {
    use windows_sys::Win32::System::DataExchange::COPYDATASTRUCT;
    let name = property(dir)?;
    let bytes = encode(paths, activate)?;
    let mut search = Search {
        name: name.as_ptr(),
        found: null_mut(),
    };
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        EnumWindows(Some(find), &mut search as *mut _ as isize);
        if !search.found.is_null() {
            break;
        }
        if Instant::now() >= deadline {
            return Err("The running ByteForge could not accept this open request. Close any older ByteForge Rust windows, then reopen the files. Existing tabs have not been changed.".into());
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    if activate {
        let mut pid = 0;
        GetWindowThreadProcessId(search.found, &mut pid);
        if pid != 0 {
            AllowSetForegroundWindow(pid);
        }
    }
    let packet = COPYDATASTRUCT {
        dwData: PROTOCOL,
        cbData: bytes.len() as u32,
        lpData: bytes.as_ptr() as *mut _,
    };
    let mut ack = 0;
    let sent = SendMessageTimeoutW(
        search.found,
        WM_COPYDATA,
        0,
        &packet as *const _ as isize,
        SMTO_ABORTIFHUNG | SMTO_ERRORONEXIT,
        5000,
        &mut ack,
    );
    if sent == 0 || ack != 1 {
        return Err("ByteForge did not confirm the file-open request. It may be busy or running at a different permission level. Return to the existing window and try opening the files again.".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_protocol() {
        let paths = vec![
            PathBuf::from("C:\\a folder\\café 😀.json"),
            PathBuf::from("C:\\other.ps1"),
        ];
        let data = encode(&paths, true).unwrap();
        let r = decode(&data).unwrap();
        assert_eq!(r.paths, paths);
        assert!(r.activate);
        for n in 0..data.len() {
            assert!(decode(&data[..n]).is_none());
        }
        let mut trailing = data.clone();
        trailing.push(0);
        assert!(decode(&trailing).is_none());
        let mut bad = data;
        bad[4] = 2;
        assert!(decode(&bad).is_none());
        assert!(encode(&[PathBuf::from("relative.txt")], true).is_err());
        assert!(decode(&[255; 16]).is_none());
        assert!(decode(&vec![0; MAX_PACKET + 1]).is_none());
    }
    #[test]
    fn arguments_and_utf16() {
        let args = [
            "--session-dir",
            "ignored",
            "--hidden",
            "one.txt",
            "two files.txt",
        ]
        .map(OsString::from);
        let paths = command_paths(&args).unwrap();
        assert_eq!(paths.len(), 2);
        assert!(paths.iter().all(|p| p.is_absolute()));
        let mut p: Vec<u16> = "C:\\unicode-".encode_utf16().collect();
        p.push(0xd800);
        p.extend(".txt".encode_utf16());
        let paths = vec![PathBuf::from(OsString::from_wide(&p))];
        assert_eq!(
            decode(&encode(&paths, false).unwrap()).unwrap().paths,
            paths
        );
        assert!(decode(&encode(&[], true).unwrap())
            .unwrap()
            .paths
            .is_empty());
    }
}
