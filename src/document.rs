use std::path::PathBuf;

pub const MAX_DOCUMENT: usize = 32 * 1024 * 1024;
pub const MAX_TABS: usize = 128;
pub const TAB_COLORS: [(&str, u32); 7] = [
    ("Default", 0xFF982E),
    ("Red", 0xEF7180),
    ("Orange", 0xFFB65C),
    ("Yellow", 0xDCC85A),
    ("Green", 0x62CBA0),
    ("Blue", 0x75B8F5),
    ("Purple", 0xB59AE8),
];
pub const HIGHLIGHTS: [u32; 4] = [0xFFA0A0, 0xFFC800, 0xFFFF00, 0x80FF80];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Encoding {
    Utf8,
    Utf8Bom,
    Utf16Le,
    Utf16Be,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Eol {
    CrLf,
    Lf,
    Cr,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Highlight {
    pub start: i32,
    pub end: i32,
    pub color: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Document {
    pub path: Option<PathBuf>,
    pub title: String,
    /// Internal text uses one LF per paragraph, matching RichEdit CP positions.
    pub text: String,
    pub encoding: Encoding,
    pub eol: Eol,
    pub dirty: bool,
    /// SHA-256 of the last opened/saved disk bytes; never updated by recovery.
    pub disk_hash: Option<String>,
    pub color: u8,
    pub font: String,
    pub points: i32,
    pub zoom: i32,
    pub wrap: bool,
    pub selection: (i32, i32),
    pub scroll: (i32, i32),
    pub highlights: Vec<Highlight>,
    pub hex: u8,
}
impl Default for Document {
    fn default() -> Self {
        Self {
            path: None,
            title: "Untitled".into(),
            text: String::new(),
            encoding: Encoding::Utf8,
            eol: Eol::CrLf,
            dirty: false,
            disk_hash: None,
            color: 0,
            font: "Consolas".into(),
            points: 11,
            zoom: 100,
            wrap: false,
            selection: (0, 0),
            scroll: (0, 0),
            highlights: vec![],
            hex: 0,
        }
    }
}
pub fn normalize(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}
/// Logical line/UTF-16 column, independent of a hidden RichEdit's wrap width.
pub fn line_column(text: &str, position: i32) -> (usize, usize) {
    let (mut line, mut col) = (1, 1);
    for u in text.encode_utf16().take(position.max(0) as usize) {
        if u == 10 {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}
pub fn position(text: &str, line: usize, column: usize) -> Option<i32> {
    if line == 0 || column == 0 {
        return None;
    }
    let mut offset = 0;
    for (i, part) in text.split('\n').enumerate() {
        let len = part.encode_utf16().count();
        if i + 1 == line {
            return Some((offset + (column - 1).min(len)) as i32);
        }
        offset += len + 1;
    }
    None
}
pub fn decode(bytes: &[u8]) -> Result<(String, Encoding, Eol), String> {
    if bytes.len() > MAX_DOCUMENT {
        return Err("File exceeds the 32 MiB editor limit.".into());
    }
    let (text, enc) = if bytes.starts_with(&[0xFF, 0xFE]) || bytes.starts_with(&[0xFE, 0xFF]) {
        if bytes.len() % 2 != 0 {
            return Err("Truncated UTF-16 input.".into());
        }
        let le = bytes[0] == 0xFF;
        let units: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|b| {
                if le {
                    u16::from_le_bytes([b[0], b[1]])
                } else {
                    u16::from_be_bytes([b[0], b[1]])
                }
            })
            .collect();
        (
            String::from_utf16(&units).map_err(|_| "Invalid UTF-16 input")?,
            if le {
                Encoding::Utf16Le
            } else {
                Encoding::Utf16Be
            },
        )
    } else {
        let bom = bytes.starts_with(&[0xEF, 0xBB, 0xBF]);
        (
            String::from_utf8(bytes[if bom { 3 } else { 0 }..].to_vec())
                .map_err(|_| "Not valid UTF-8 or BOM-marked UTF-16. File was not opened.")?,
            if bom {
                Encoding::Utf8Bom
            } else {
                Encoding::Utf8
            },
        )
    };
    if text.contains('\0') {
        return Err("NUL-containing/binary input is not supported by this text editor.".into());
    }
    let eol = if text.contains("\r\n") {
        Eol::CrLf
    } else if text.contains('\r') {
        Eol::Cr
    } else {
        Eol::Lf
    };
    Ok((normalize(&text), enc, eol))
}
impl Document {
    pub fn bytes(&self) -> Vec<u8> {
        let normalized = normalize(&self.text);
        let text = match self.eol {
            Eol::CrLf => normalized.replace('\n', "\r\n"),
            Eol::Cr => normalized.replace('\n', "\r"),
            Eol::Lf => normalized,
        };
        match self.encoding {
            Encoding::Utf8 => text.into_bytes(),
            Encoding::Utf8Bom => [vec![0xEF, 0xBB, 0xBF], text.into_bytes()].concat(),
            Encoding::Utf16Le | Encoding::Utf16Be => {
                let le = self.encoding == Encoding::Utf16Le;
                let mut out = if le {
                    vec![0xFF, 0xFE]
                } else {
                    vec![0xFE, 0xFF]
                };
                for u in text.encode_utf16() {
                    out.extend(if le { u.to_le_bytes() } else { u.to_be_bytes() });
                }
                out
            }
        }
    }
    pub fn label(&self) -> String {
        format!("{}{}", if self.dirty { "* " } else { "" }, self.title)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn encoding_roundtrips() {
        for encoding in [
            Encoding::Utf8,
            Encoding::Utf8Bom,
            Encoding::Utf16Le,
            Encoding::Utf16Be,
        ] {
            for eol in [Eol::CrLf, Eol::Lf, Eol::Cr] {
                let d = Document {
                    text: "hé 😀\nnext".into(),
                    encoding,
                    eol,
                    ..Document::default()
                };
                let (s, e, l) = decode(&d.bytes()).unwrap();
                assert_eq!((s, e, l), (d.text, encoding, eol));
            }
        }
    }
    #[test]
    fn rejects_lossy_binary() {
        assert!(decode(&[0xFF, 0]).is_err());
        assert!(decode(b"a\0b").is_err());
        assert!(decode(&[0xFF, 0xFE, 1]).is_err());
    }
}
