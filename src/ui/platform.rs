use super::native::*;
pub use byteforge_rust::update::REPOSITORY;
use std::ptr::{null, null_mut};
use windows_sys::Win32::Networking::WinHttp::*;
pub unsafe fn open_repository(owner: HWND) {
    let result = windows_sys::Win32::UI::Shell::ShellExecuteW(
        owner,
        wide("open").as_ptr(),
        wide(REPOSITORY).as_ptr(),
        null(),
        null(),
        SW_SHOWNORMAL,
    ) as isize;
    if result <= 32 {
        error(owner, "Could not open GitHub in your default browser.");
    }
}
struct HttpHandle(*mut std::ffi::c_void);
impl Drop for HttpHandle {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                WinHttpCloseHandle(self.0);
            }
        }
    }
}
pub fn latest_version() -> Result<byteforge_rust::update::Check, String> {
    unsafe {
        let started = std::time::Instant::now();
        let session = HttpHandle(WinHttpOpen(
            wide(concat!(
                "ByteForge-Rust/",
                env!("CARGO_PKG_VERSION"),
                " manual update check"
            ))
            .as_ptr(),
            WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY,
            null(),
            null(),
            0,
        ));
        if session.0.is_null() {
            return Err("Could not open WinHTTP".into());
        }
        if WinHttpSetTimeouts(session.0, 5000, 5000, 5000, 5000) == 0 {
            return Err("Could not set update-check timeouts.".into());
        }
        let conn = HttpHandle(WinHttpConnect(
            session.0,
            wide("raw.githubusercontent.com").as_ptr(),
            443,
            0,
        ));
        if conn.0.is_null() {
            return Err("Could not connect".into());
        }
        let req = HttpHandle(WinHttpOpenRequest(
            conn.0,
            wide("GET").as_ptr(),
            wide("/Jaap79/ByteForge-Rust/main/Cargo.toml").as_ptr(),
            null(),
            null(),
            null(),
            WINHTTP_FLAG_SECURE,
        ));
        if req.0.is_null() {
            return Err("Could not create request".into());
        }
        let redirect = WINHTTP_OPTION_REDIRECT_POLICY_NEVER;
        if WinHttpSetOption(
            req.0,
            WINHTTP_OPTION_REDIRECT_POLICY,
            &redirect as *const _ as _,
            4,
        ) == 0
        {
            return Err("Could not restrict update-check redirects.".into());
        }
        let headers = wide("Cache-Control: no-cache\r\nAccept: text/plain\r\n");
        if WinHttpSendRequest(
            req.0,
            headers.as_ptr(),
            (headers.len() - 1) as u32,
            null(),
            0,
            0,
            0,
        ) == 0
            || WinHttpReceiveResponse(req.0, null_mut()) == 0
        {
            return Err(
                "Could not reach GitHub. Check your connection or proxy and try again.".into(),
            );
        }
        let mut status = 0u32;
        let mut size = 4;
        if WinHttpQueryHeaders(
            req.0,
            WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
            null(),
            &mut status as *mut _ as _,
            &mut size,
            null_mut(),
        ) == 0
            || status != 200
        {
            return Err(format!(
                "GitHub returned HTTP {status}; the latest version could not be verified."
            ));
        }
        let mut body = vec![];
        loop {
            if started.elapsed() > std::time::Duration::from_secs(15) {
                return Err("GitHub update check timed out.".into());
            }
            let mut b = [0u8; 1024];
            let mut n = 0;
            if WinHttpReadData(req.0, b.as_mut_ptr() as _, 1024, &mut n) == 0 {
                return Err("Response read failed".into());
            }
            if n == 0 {
                break;
            }
            body.extend_from_slice(&b[..n as usize]);
            if body.len() > 32768 {
                return Err("Response too large".into());
            }
        }
        let body = String::from_utf8(body).map_err(|_| "Invalid response")?;
        byteforge_rust::update::check_manifest(&body, byteforge_rust::update::CURRENT)
    }
}
