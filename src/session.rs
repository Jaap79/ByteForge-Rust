//! Versioned, bounded, checksummed recovery snapshots. No saved file is modified here.
use crate::{document::*, storage::atomic_write};
use std::{
    fs,
    io::Read,
    os::windows::ffi::{OsStrExt, OsStringExt},
    path::{Path, PathBuf},
};
const MAGIC: &[u8; 8] = b"BFRSESS1";
const LIMIT: usize = 128 * 1024 * 1024;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Session {
    pub documents: Vec<Document>,
    pub active: usize,
    pub dark: bool,
    pub enabled: bool,
}
impl Default for Session {
    fn default() -> Self {
        Self {
            documents: vec![Document::default()],
            active: 0,
            dark: true,
            enabled: true,
        }
    }
}
pub fn directory() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("ByteForge-Rust")
}
fn crc(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for b in data {
        crc ^= *b as u32;
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xEDB8_8320u32.wrapping_mul(crc & 1));
        }
    }
    !crc
}
fn u32out(out: &mut Vec<u8>, n: u32) {
    out.extend(n.to_le_bytes());
}
fn int(out: &mut Vec<u8>, n: i32) {
    u32out(out, n as u32);
}
fn string(out: &mut Vec<u8>, s: &str) {
    u32out(out, s.len() as u32);
    out.extend(s.as_bytes());
}
pub fn encode(s: &Session) -> Result<Vec<u8>, String> {
    if s.documents.is_empty() || s.documents.len() > MAX_TABS {
        return Err("Invalid tab count".into());
    }
    let mut out = MAGIC.to_vec();
    u32out(&mut out, s.documents.len() as u32);
    u32out(&mut out, s.active as u32);
    out.extend([s.dark as u8, s.enabled as u8]);
    for d in &s.documents {
        if d.text.len() > MAX_DOCUMENT {
            return Err(
                "A tab exceeds the 32 MiB UTF-8 recovery limit. Save the document to disk.".into(),
            );
        }
        let path: Vec<u16> = d
            .path
            .as_ref()
            .map(|p| p.as_os_str().encode_wide().collect())
            .unwrap_or_default();
        u32out(&mut out, path.len() as u32);
        for u in path {
            out.extend(u.to_le_bytes());
        }
        string(&mut out, &d.title);
        string(&mut out, &d.text);
        string(&mut out, &d.font);
        string(&mut out, d.disk_hash.as_deref().unwrap_or(""));
        out.extend([
            d.encoding as u8,
            d.eol as u8,
            d.dirty as u8,
            d.color,
            d.wrap as u8,
            d.hex,
        ]);
        for n in [
            d.points,
            d.zoom,
            d.selection.0,
            d.selection.1,
            d.scroll.0,
            d.scroll.1,
        ] {
            int(&mut out, n);
        }
        u32out(&mut out, d.highlights.len() as u32);
        for h in &d.highlights {
            int(&mut out, h.start);
            int(&mut out, h.end);
            u32out(&mut out, h.color);
        }
        if out.len() > LIMIT {
            return Err("Session exceeds 128 MiB recovery limit. Save documents to disk.".into());
        }
    }
    let sum = crc(&out);
    u32out(&mut out, sum);
    Ok(out)
}
struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
        let end = self.at.checked_add(n).ok_or("Length overflow")?;
        let b = self.bytes.get(self.at..end).ok_or("Truncated session")?;
        self.at = end;
        Ok(b)
    }
    fn number(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn signed(&mut self) -> Result<i32, String> {
        Ok(self.number()? as i32)
    }
    fn byte(&mut self) -> Result<u8, String> {
        Ok(self.take(1)?[0])
    }
    fn string(&mut self, max: usize) -> Result<String, String> {
        let n = self.number()? as usize;
        if n > max {
            return Err("Session string exceeds limit".into());
        }
        let s = std::str::from_utf8(self.take(n)?).map_err(|_| "Invalid session UTF-8")?;
        if s.contains('\0') {
            return Err("NUL in session text".into());
        }
        Ok(s.into())
    }
}
pub fn decode(data: &[u8]) -> Result<Session, String> {
    if data.len() < 22 || data.len() > LIMIT + 4 || &data[..8] != MAGIC {
        return Err("Unsupported session format/size".into());
    }
    let end = data.len() - 4;
    if crc(&data[..end]) != u32::from_le_bytes(data[end..].try_into().unwrap()) {
        return Err("Session checksum mismatch".into());
    }
    let mut r = Reader {
        bytes: &data[..end],
        at: 8,
    };
    let count = r.number()? as usize;
    let active = r.number()? as usize;
    if count == 0 || count > MAX_TABS || active >= count {
        return Err("Invalid session tab index".into());
    }
    let dark = r.byte()? != 0;
    let enabled = r.byte()? != 0;
    let mut documents = vec![];
    for _ in 0..count {
        let plen = r.number()? as usize;
        if plen > 32767 {
            return Err("Session path exceeds limit".into());
        }
        let p: Vec<u16> = r
            .take(plen * 2)?
            .chunks_exact(2)
            .map(|b| u16::from_le_bytes([b[0], b[1]]))
            .collect();
        if p.contains(&0) {
            return Err("NUL in session path".into());
        }
        let path = if p.is_empty() {
            None
        } else {
            Some(PathBuf::from(std::ffi::OsString::from_wide(&p)))
        };
        let title = r.string(32768)?;
        let text = r.string(MAX_DOCUMENT)?;
        let font = r.string(256)?;
        let hash = r.string(64)?;
        let encoding = match r.byte()? {
            0 => Encoding::Utf8,
            1 => Encoding::Utf8Bom,
            2 => Encoding::Utf16Le,
            3 => Encoding::Utf16Be,
            _ => return Err("Invalid encoding".into()),
        };
        let eol = match r.byte()? {
            0 => Eol::CrLf,
            1 => Eol::Lf,
            2 => Eol::Cr,
            _ => return Err("Invalid line ending".into()),
        };
        let dirty = r.byte()? != 0;
        let color = r.byte()?;
        let wrap = r.byte()? != 0;
        let hex = r.byte()?;
        let points = r.signed()?;
        let zoom = r.signed()?;
        let selection = (r.signed()?, r.signed()?);
        let scroll = (r.signed()?, r.signed()?);
        if color as usize >= TAB_COLORS.len()
            || !(6..=72).contains(&points)
            || !(25..=500).contains(&zoom)
            || hex > 3
        {
            return Err("Invalid tab settings".into());
        }
        let n = r.number()? as usize;
        let len = text.encode_utf16().count() as i32;
        if n > len as usize + 1 {
            return Err("Too many highlight ranges".into());
        }
        let mut highlights = vec![];
        let mut last = 0;
        for _ in 0..n {
            let start = r.signed()?;
            let end = r.signed()?;
            let color = r.number()?;
            if start < last || end <= start || end > len {
                return Err("Invalid highlight range".into());
            }
            last = end;
            highlights.push(Highlight { start, end, color });
        }
        documents.push(Document {
            path,
            title,
            text,
            encoding,
            eol,
            dirty,
            disk_hash: if hash.is_empty() { None } else { Some(hash) },
            color,
            font,
            points,
            zoom,
            wrap,
            selection,
            scroll,
            highlights,
            hex,
        });
    }
    if r.at != end {
        return Err("Trailing session data".into());
    }
    Ok(Session {
        documents,
        active,
        dark,
        enabled,
    })
}
pub fn load(dir: &Path) -> Result<Option<(Session, bool)>, String> {
    let path = dir.join("session.bfs");
    if !path.exists() && !dir.join("session.previous.bfs").exists() {
        return Ok(None);
    }
    fn read(path: &Path) -> Result<Session, String> {
        let file = fs::File::open(path).map_err(|e| e.to_string())?;
        if file.metadata().map_err(|e| e.to_string())?.len() > LIMIT as u64 + 4 {
            return Err("Session too large".into());
        }
        let mut b = vec![];
        file.take(LIMIT as u64 + 5)
            .read_to_end(&mut b)
            .map_err(|e| e.to_string())?;
        decode(&b)
    }
    match read(&path) {
        Ok(s) => Ok(Some((s, false))),
        Err(first) => match read(&dir.join("session.previous.bfs")) {
            Ok(s) => Ok(Some((s, true))),
            Err(_) => Err(format!(
                "Recovery data could not be read: {first}. The existing files have been preserved."
            )),
        },
    }
}
pub fn save(dir: &Path, s: &Session) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let bytes = encode(s)?;
    let path = dir.join("session.bfs");
    if let Ok(old) = fs::read(&path) {
        if decode(&old).is_ok() {
            atomic_write(&dir.join("session.previous.bfs"), &old)?;
        }
    }
    atomic_write(&path, &bytes)
}
pub fn clear(dir: &Path) -> Result<(), String> {
    for name in ["session.bfs", "session.previous.bfs"] {
        let p = dir.join(name);
        if p.exists() {
            fs::remove_file(p).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_settings_roundtrip() {
        let d = Document {
            text: "abc 😀\nxyz".into(),
            dirty: true,
            color: 4,
            font: "Georgia".into(),
            zoom: 160,
            wrap: true,
            selection: (2, 5),
            scroll: (0, 72),
            highlights: vec![Highlight {
                start: 0,
                end: 3,
                color: 0xFFFF00,
            }],
            ..Document::default()
        };
        let s = Session {
            documents: vec![Document::default(), d],
            active: 1,
            dark: false,
            enabled: true,
        };
        assert_eq!(decode(&encode(&s).unwrap()).unwrap(), s);
    }
    #[test]
    fn malformed_rejected() {
        let data = encode(&Session::default()).unwrap();
        for n in 0..data.len() {
            assert!(decode(&data[..n]).is_err());
        }
        let mut bad = data.clone();
        bad[20] ^= 64;
        assert!(decode(&bad).is_err());
        let mut trailing = data;
        trailing.push(1);
        assert!(decode(&trailing).is_err());
    }
    #[test]
    fn backup_recovery() {
        let dir = std::env::temp_dir().join(format!("bf-session-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let mut s = Session::default();
        s.documents[0].text = "original".into();
        save(&dir, &s).unwrap();
        s.documents[0].text = "new".into();
        save(&dir, &s).unwrap();
        atomic_write(&dir.join("session.bfs"), b"corrupt").unwrap();
        let (recovered, backup) = load(&dir).unwrap().unwrap();
        assert!(backup);
        assert_eq!(recovered.documents[0].text, "original");
        clear(&dir).unwrap();
        fs::remove_dir(dir).unwrap();
    }
}
