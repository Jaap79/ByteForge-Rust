use super::native::*;
use std::ptr::{null, null_mut};
use windows_sys::Win32::Networking::WinHttp::*;
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
pub fn upstream_version() -> Result<String, String> {
    unsafe {
        let session = HttpHandle(WinHttpOpen(
            wide("ByteForge-Rust/1.2.0 manual upstream check").as_ptr(),
            WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY,
            null(),
            null(),
            0,
        ));
        if session.0.is_null() {
            return Err("Could not open WinHTTP".into());
        }
        WinHttpSetTimeouts(session.0, 5000, 5000, 5000, 5000);
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
            wide("/Jaap79/ByteForge/main/version.json").as_ptr(),
            null(),
            null(),
            null(),
            WINHTTP_FLAG_SECURE,
        ));
        if req.0.is_null() {
            return Err("Could not create request".into());
        }
        let redirect = WINHTTP_OPTION_REDIRECT_POLICY_NEVER;
        WinHttpSetOption(
            req.0,
            WINHTTP_OPTION_REDIRECT_POLICY,
            &redirect as *const _ as _,
            4,
        );
        if WinHttpSendRequest(req.0, null(), 0, null(), 0, 0, 0) == 0
            || WinHttpReceiveResponse(req.0, null_mut()) == 0
        {
            return Err("Upstream check failed".into());
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
            return Err(format!("Upstream returned HTTP {status}"));
        }
        let mut body = vec![];
        loop {
            let mut b = [0u8; 1024];
            let mut n = 0;
            if WinHttpReadData(req.0, b.as_mut_ptr() as _, 1024, &mut n) == 0 {
                return Err("Response read failed".into());
            }
            if n == 0 {
                break;
            }
            body.extend_from_slice(&b[..n as usize]);
            if body.len() > 8192 {
                return Err("Response too large".into());
            }
        }
        let body = String::from_utf8(body).map_err(|_| "Invalid response")?;
        let version = body
            .split("\"version\"")
            .nth(1)
            .and_then(|s| s.split(':').nth(1))
            .and_then(|s| s.split('"').nth(1))
            .ok_or("Version missing")?;
        if version.len() > 32 || !version.chars().all(|c| c.is_ascii_digit() || c == '.') {
            return Err("Invalid version".into());
        }
        Ok(version.into())
    }
}
