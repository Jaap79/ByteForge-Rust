//! Generate the small icon, manifest and COFF resource object without an SDK/resource compiler.
use std::{env, fs, path::PathBuf};
fn u16le(v: &mut Vec<u8>, n: u16) {
    v.extend(n.to_le_bytes());
}
fn u32le(v: &mut Vec<u8>, n: u32) {
    v.extend(n.to_le_bytes());
}
fn put(v: &mut [u8], at: usize, n: u32) {
    v[at..at + 4].copy_from_slice(&n.to_le_bytes());
}
fn aligned(v: &mut Vec<u8>) {
    while v.len() % 4 != 0 {
        v.push(0);
    }
}
fn icon(size: usize) -> Vec<u8> {
    let mut v = Vec::new();
    for n in [40, size as u32, size as u32 * 2] {
        u32le(&mut v, n);
    }
    u16le(&mut v, 1);
    u16le(&mut v, 32);
    for n in [0, (size * size * 4) as u32, 0, 0, 0, 0] {
        u32le(&mut v, n);
    }
    for y in (0..size).rev() {
        for x in 0..size {
            let (x, y) = (x * 32 / size, y * 32 / size);
            let page = (7..=23).contains(&x) && (6..=25).contains(&y);
            let line = matches!(x,7..=9|21..=23) || matches!(y,6..=8|23..=25);
            let pixel = if (18..=27).contains(&x) && (4..=10).contains(&y) {
                0xffff982eu32
            } else if page && line || ((12..=18).contains(&x) && matches!(y,13..=14|18..=19)) {
                0xfff1f4f6
            } else {
                0xff1b1e24
            };
            v.extend(pixel.to_le_bytes());
        }
    }
    v.resize(v.len() + size.div_ceil(32) * 4 * size, 0);
    v
}
#[derive(Clone)]
enum Node {
    Dir(Vec<(u32, Node)>),
    Data(Vec<u8>),
}
fn version_block(
    key: &str,
    value: &[u8],
    value_len: u16,
    kind: u16,
    children: &[Vec<u8>],
) -> Vec<u8> {
    let mut b = vec![];
    u16le(&mut b, 0);
    u16le(&mut b, value_len);
    u16le(&mut b, kind);
    for u in key.encode_utf16().chain(Some(0)) {
        u16le(&mut b, u);
    }
    aligned(&mut b);
    b.extend(value);
    for child in children {
        aligned(&mut b);
        b.extend(child);
    }
    let n = b.len() as u16;
    b[..2].copy_from_slice(&n.to_le_bytes());
    b
}
fn version_resource() -> Vec<u8> {
    let mut fixed = vec![];
    for n in [
        0xFEEF04BD, 0x00010000, 0x00010002, 0x00000000, 0x00010002, 0x00000000, 0x3F, 0,
        0x00040004, 1, 0, 0, 0,
    ] {
        u32le(&mut fixed, n);
    }
    let fields = [
        ("FileDescription", "ByteForge Rust native tabbed editor"),
        ("FileVersion", "1.2.0.0"),
        ("ProductName", "ByteForge Rust"),
        ("ProductVersion", "1.2.0.0"),
        ("OriginalFilename", "ByteForge-Rust.exe"),
        ("InternalName", "ByteForge-Rust"),
        ("LegalCopyright", "Copyright 2026 Jaap"),
    ]
    .into_iter()
    .map(|(k, v)| {
        let units: Vec<u16> = v.encode_utf16().chain(Some(0)).collect();
        let mut b = vec![];
        for u in &units {
            u16le(&mut b, *u);
        }
        version_block(k, &b, units.len() as u16, 1, &[])
    })
    .collect::<Vec<_>>();
    let table = version_block("040904B0", &[], 0, 1, &fields);
    let strings = version_block("StringFileInfo", &[], 0, 1, &[table]);
    let translation = version_block("Translation", &[9, 4, 0xB0, 4], 4, 0, &[]);
    let vars = version_block("VarFileInfo", &[], 0, 1, &[translation]);
    version_block("VS_VERSION_INFO", &fixed, 52, 0, &[strings, vars])
}
fn emit(n: &Node, bytes: &mut Vec<u8>, reloc: &mut Vec<u32>) -> u32 {
    aligned(bytes);
    let at = bytes.len();
    match n {
        Node::Dir(children) => {
            bytes.resize(at + 16 + children.len() * 8, 0);
            bytes[at + 14..at + 16].copy_from_slice(&(children.len() as u16).to_le_bytes());
            for (i, (id, child)) in children.iter().enumerate() {
                let off = emit(child, bytes, reloc);
                put(bytes, at + 16 + i * 8, *id);
                put(
                    bytes,
                    at + 20 + i * 8,
                    off | if matches!(child, Node::Dir(_)) {
                        0x80000000
                    } else {
                        0
                    },
                );
            }
        }
        Node::Data(data) => {
            bytes.resize(at + 16, 0);
            let start = bytes.len() as u32;
            put(bytes, at, start);
            put(bytes, at + 4, data.len() as u32);
            reloc.push(at as u32);
            bytes.extend(data);
        }
    }
    at as u32
}
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let sizes = [16usize, 32, 48];
    let mut group = vec![0, 0, 1, 0, 3, 0];
    let mut ico = group.clone();
    let icons: Vec<Vec<u8>> = sizes.iter().map(|&s| icon(s)).collect();
    let mut pos = 6 + 16 * 3;
    for (i, &s) in sizes.iter().enumerate() {
        let mut entry = vec![s as u8, s as u8, 0, 0];
        u16le(&mut entry, 1);
        u16le(&mut entry, 32);
        u32le(&mut entry, icons[i].len() as u32);
        group.extend(&entry);
        u16le(&mut group, (i + 1) as u16);
        ico.extend(entry);
        u32le(&mut ico, pos);
        pos += icons[i].len() as u32;
    }
    for bytes in &icons {
        ico.extend(bytes);
    }
    fs::write(out.join("app.ico"), ico).unwrap();
    let manifest=br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
<assemblyIdentity version="1.2.0.0" processorArchitecture="amd64" name="ByteForge.Rust" type="win32"/>
<description>ByteForge Rust</description>
<dependency><dependentAssembly><assemblyIdentity type="win32" name="Microsoft.Windows.Common-Controls" version="6.0.0.0" processorArchitecture="*" publicKeyToken="6595b64144ccf1df" language="*"/></dependentAssembly></dependency>
<trustInfo xmlns="urn:schemas-microsoft-com:asm.v3"><security><requestedPrivileges><requestedExecutionLevel level="asInvoker" uiAccess="false"/></requestedPrivileges></security></trustInfo>
<application xmlns="urn:schemas-microsoft-com:asm.v3"><windowsSettings><dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">PerMonitorV2</dpiAwareness><longPathAware xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">true</longPathAware></windowsSettings></application>
</assembly>"#;
    let lang = |data| Node::Dir(vec![(1033, Node::Data(data))]);
    let root = Node::Dir(vec![
        (
            3,
            Node::Dir(
                icons
                    .into_iter()
                    .enumerate()
                    .map(|(i, b)| ((i + 1) as u32, lang(b)))
                    .collect(),
            ),
        ),
        (14, Node::Dir(vec![(1, lang(group))])),
        (16, Node::Dir(vec![(1, lang(version_resource()))])),
        (24, Node::Dir(vec![(1, lang(manifest.to_vec()))])),
    ]);
    let mut data = Vec::new();
    let mut reloc = Vec::new();
    emit(&root, &mut data, &mut reloc);
    aligned(&mut data);
    let reloc_offset = 60 + data.len() as u32;
    let sym_offset = reloc_offset + reloc.len() as u32 * 10;
    let mut obj = Vec::new();
    u16le(&mut obj, 0x8664);
    u16le(&mut obj, 1);
    u32le(&mut obj, 0);
    u32le(&mut obj, sym_offset);
    u32le(&mut obj, 1);
    u16le(&mut obj, 0);
    u16le(&mut obj, 0);
    obj.extend(b".rsrc\0\0\0");
    u32le(&mut obj, 0);
    u32le(&mut obj, 0);
    u32le(&mut obj, data.len() as u32);
    u32le(&mut obj, 60);
    u32le(&mut obj, reloc_offset);
    u32le(&mut obj, 0);
    u16le(&mut obj, reloc.len() as u16);
    u16le(&mut obj, 0);
    u32le(&mut obj, 0x40300040);
    obj.extend(data);
    for at in reloc {
        u32le(&mut obj, at);
        u32le(&mut obj, 0);
        u16le(&mut obj, 3);
    }
    obj.extend(b".rsrc\0\0\0");
    u32le(&mut obj, 0);
    u16le(&mut obj, 1);
    u16le(&mut obj, 0);
    obj.extend([3, 0]);
    u32le(&mut obj, 4);
    let path = out.join("resources.o");
    fs::write(&path, obj).unwrap();
    println!("cargo:rustc-link-arg-bin=ByteForge-Rust={}", path.display());
}
