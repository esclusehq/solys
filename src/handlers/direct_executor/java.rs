//! Java version detection and validation for Direct Executor

use anyhow::{bail, Result};

/// Detect Java on PATH and return (major_version, version_string).
///
/// Runs `java --version`, parses the first line, extracts the major
/// version number. Returns `None` if Java is not found or version
/// cannot be parsed.
///
/// # Format support
/// - OpenJDK (standard): `openjdk version "21.0.1" 2026-01-20` → 21
/// - OpenJDK (Termux):   `openjdk 21.0.10 2026-01-20`         → 21
/// - Oracle:    `java version "21.0.1"`     → 21
/// - GraalVM:   `java version "21.0.1"`     → 21
/// - IBM JDK:   `java version "1.8.0"`      → 8
///
/// Uses std::process::Command (sync, called once at startup).
/// Tries multiple detection strategies for broad platform coverage.
pub fn detect_java_version() -> Option<(u32, String)> {
    // Collect all candidate paths to try
    let candidates = java_candidates();

    for candidate in &candidates {
        if let Some(res) = run_java_version(&std::path::PathBuf::from(candidate)) {
            return Some(res);
        }
    }

    tracing::warn!(
        "Java detection exhausted {:?} candidates — none produced version output",
        candidates.iter().map(|s| s.as_str()).collect::<Vec<_>>()
    );
    None
}

/// Return a list of candidate Java paths to try, ordered by preference.
/// Includes direct paths, shell resolution, and bare command name.
fn java_candidates() -> Vec<String> {
    let mut candidates = Vec::new();

    // Strategy 1: which crate (Rust-native PATH search)
    if let Ok(p) = which::which("java") {
        candidates.push(p.to_string_lossy().to_string());
    }

    // Strategy 2: known Termux paths (direct binary, no symlink chain)
    candidates.push("/data/data/com.termux/files/usr/lib/jvm/java-21-openjdk/bin/java".into());
    candidates.push("/data/data/com.termux/files/usr/lib/jvm/java-17-openjdk/bin/java".into());

    // Strategy 3: shell's `command -v` (works on Termux when which::which fails)
    let output = std::process::Command::new("sh")
        .args(["-c", "command -v java"])
        .output()
        .ok();
    if let Some(out) = output {
        if out.status.success() {
            let trimmed = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !trimmed.is_empty() {
                candidates.push(trimmed);
            }
        }
    }

    // Strategy 4: bare `java` (let the OS resolve via inherited PATH)
    candidates.push("java".into());

    candidates
}

fn run_java_version(java_path: &std::path::Path) -> Option<(u32, String)> {
    let output = match std::process::Command::new(java_path)
        .arg("--version")
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            tracing::warn!("java detection: failed to run {:?}: {}", java_path, e);
            return None;
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let first_line_raw = stdout.lines().next()?;
    let first_line = first_line_raw.trim();
    tracing::debug!("java detection: {:?} first_line={:?}", java_path, first_line);

    parse_java_version(first_line)
}

/// Parse the first line of `java --version` output into (major, version_string).
///
/// Supported formats:
/// - `openjdk version "21.0.1" 2026-01-20` → (21, "openjdk version...")
/// - `openjdk 21.0.10 2026-01-20`          → (21, "openjdk 21.0.10...")
/// - `java version "1.8.0_202"`            → (8, "java version...")
fn parse_java_version(first_line: &str) -> Option<(u32, String)> {
    let version_str = first_line
        .split('"')
        .nth(1)
        .or_else(|| first_line.split_whitespace().nth(1))?;

    let raw_major = version_str.split('.').next()?;
    let major_str = if raw_major == "1" {
        version_str.split('.').nth(1).unwrap_or("8")
    } else {
        raw_major
    };

    let major: u32 = major_str.parse().ok()?;
    Some((major, first_line.trim().to_string()))
}

/// Verify Java version is sufficient for the target Minecraft version.
///
/// - Minecraft 1.21+ requires Java 21+
/// - Minecraft < 1.21 requires Java 17+
pub fn validate_java_for_version(mc_version: &str, java_major: u32) -> Result<()> {
    // Parse MC major version: "1.21.4" → 21, "1.20.4" → 20
    let mc_major: u32 = mc_version
        .split('.')
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let required = if mc_major >= 21 { 21 } else { 17 };

    if java_major >= required {
        Ok(())
    } else {
        bail!(
            "Java {} is required for Minecraft {} (found Java {})",
            required, mc_version, java_major
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_java_21_sufficient_for_1_21() {
        assert!(validate_java_for_version("1.21.4", 21).is_ok());
        assert!(validate_java_for_version("1.21.4", 22).is_ok());
    }

    #[test]
    fn test_java_17_insufficient_for_1_21() {
        assert!(validate_java_for_version("1.21.4", 17).is_err());
        assert!(validate_java_for_version("1.21", 17).is_err());
    }

    #[test]
    fn test_java_17_sufficient_for_1_20() {
        assert!(validate_java_for_version("1.20.4", 17).is_ok());
        assert!(validate_java_for_version("1.20.4", 21).is_ok());
    }

    #[test]
    fn test_java_8_insufficient_for_1_20() {
        assert!(validate_java_for_version("1.20.4", 8).is_err());
    }

    #[test]
    fn test_version_parsing_edge_cases() {
        // Java 11 works for < 1.21 but not >= 1.21
        assert!(validate_java_for_version("1.20.4", 11).is_err());
        assert!(validate_java_for_version("1.21", 11).is_err());
    }

    #[test]
    fn test_parse_standard_quoted() {
        let r = parse_java_version(r#"openjdk version "21.0.1" 2026-01-20"#).unwrap();
        assert_eq!(r.0, 21);
    }

    #[test]
    fn test_parse_termux_no_quotes() {
        let r = parse_java_version("openjdk 21.0.10 2026-01-20").unwrap();
        assert_eq!(r.0, 21);
    }

    #[test]
    fn test_parse_java_8() {
        let r = parse_java_version(r#"java version "1.8.0_202""#).unwrap();
        assert_eq!(r.0, 8);
    }

    #[test]
    fn test_parse_ga_version_1_8() {
        let r = parse_java_version(r#"java version "1.8.0_202" 2026-01-01"#).unwrap();
        assert_eq!(r.0, 8);
    }
}
