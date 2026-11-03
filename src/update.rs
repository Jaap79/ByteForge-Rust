//! Version comparison for the explicit, read-only GitHub update check.
use std::cmp::Ordering;

pub const CURRENT: &str = env!("CARGO_PKG_VERSION");
pub const REPOSITORY: &str = "https://github.com/Jaap79/ByteForge-Rust";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Version([u32; 4]);
impl Version {
    fn parse(value: &str) -> Result<Self, String> {
        let parts: Vec<_> = value.split('.').collect();
        if !(3..=4).contains(&parts.len()) || value.len() > 40 {
            return Err("Invalid version number in GitHub response.".into());
        }
        let mut numbers = [0; 4];
        for (number, part) in numbers.iter_mut().zip(parts) {
            if part.is_empty()
                || !part.bytes().all(|c| c.is_ascii_digit())
                || part.len() > 1 && part.starts_with('0')
            {
                return Err("Invalid version number in GitHub response.".into());
            }
            *number = part
                .parse()
                .map_err(|_| "Version number is out of range.")?;
        }
        Ok(Self(numbers))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Current,
    Available,
    Ahead,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Check {
    pub latest: String,
    pub status: Status,
}
impl Check {
    pub fn message(&self) -> String {
        let heading = match self.status {
            Status::Current => "ByteForge Rust is up to date.",
            Status::Available => "A newer ByteForge Rust version is available.",
            Status::Ahead => "This local build is newer than the published GitHub version.",
        };
        format!("{heading}\n\nInstalled: {CURRENT}\nGitHub (main): {}\n\nOpen the repository below to view the current executable in dist/.\nNothing is downloaded or installed automatically.", self.latest)
    }
}

/// Read the package version only, never dependency versions or arbitrary URLs.
/// This intentionally accepts the repository's simple quoted Cargo version,
/// not general TOML expressions, escaped strings or prerelease ordering.
pub fn check_manifest(body: &str, current: &str) -> Result<Check, String> {
    if body.len() > 32768 {
        return Err("GitHub version response is too large.".into());
    }
    let mut package = false;
    let mut found = None;
    for line in body.trim_start_matches('\u{feff}').lines() {
        let line = line.trim();
        if line.starts_with('[') {
            package = line == "[package]";
            continue;
        }
        if !package {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.trim() != "version" {
            continue;
        }
        if found.is_some() {
            return Err("Duplicate package version in GitHub response.".into());
        }
        let quoted = value
            .trim()
            .strip_prefix('"')
            .ok_or("Invalid package version in GitHub response.")?;
        let (version, tail) = quoted
            .split_once('"')
            .ok_or("Incomplete package version in GitHub response.")?;
        if !tail.trim().is_empty() && !tail.trim().starts_with('#') {
            return Err("Unexpected package version data in GitHub response.".into());
        }
        Version::parse(version)?;
        found = Some(version);
    }
    let latest = found.ok_or("No package version in GitHub response.")?;
    let status = match Version::parse(latest)?.cmp(&Version::parse(current)?) {
        Ordering::Equal => Status::Current,
        Ordering::Greater => Status::Available,
        Ordering::Less => Status::Ahead,
    };
    Ok(Check {
        latest: latest.into(),
        status,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn manifest(version: &str) -> String {
        format!("[package]\nname = \"byteforge-rust\"\nversion = \"{version}\"\n[dependencies]\nversion = \"99.9.9\"\n")
    }
    #[test]
    fn numeric_version_comparison() {
        for (remote, local, expected) in [
            ("1.2.3", "1.2.3", Status::Current),
            ("1.2.3.0", "1.2.3", Status::Current),
            ("1.2.10", "1.2.9", Status::Available),
            ("1.10.0", "1.9.99", Status::Available),
            ("2.0.0", "1.99.99", Status::Available),
            ("1.2.2", "1.2.3", Status::Ahead),
        ] {
            assert_eq!(
                check_manifest(&manifest(remote), local).unwrap().status,
                expected
            );
        }
    }
    #[test]
    fn rejects_untrusted_or_ambiguous_versions() {
        for value in [
            "",
            "1.2",
            "1.2.3.4.5",
            "v1.2.3",
            "1.02.3",
            "1.2.3-beta",
            "1.2.4294967296",
            "1.2.3\" trailing",
            "https://evil.test/",
        ] {
            assert!(
                check_manifest(&manifest(value), "1.2.3").is_err(),
                "{value}"
            );
        }
        for body in [
            "<html>404</html>",
            "[dependencies]\nversion=\"1.2.3\"",
            "[package]\nversion=\"1.2.3\"\nversion=\"9.9.9\"",
        ] {
            assert!(check_manifest(body, "1.2.3").is_err());
        }
        assert!(check_manifest(&"x".repeat(32769), "1.2.3").is_err());
    }
    #[test]
    fn reads_real_package_manifest() {
        assert_eq!(
            check_manifest(include_str!("../Cargo.toml"), CURRENT)
                .unwrap()
                .status,
            Status::Current
        );
        assert_eq!(
            check_manifest(
                "\u{feff}[package]\n version = \"1.2.3\" # stable\n",
                "1.2.3"
            )
            .unwrap()
            .latest,
            "1.2.3"
        );
    }
}
