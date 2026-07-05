//! Java version detection and validation for Direct Executor

use anyhow::{bail, Result};

/// Detect Java on PATH and return (major_version, version_string).
///
/// Runs `java --version`, parses the first line, extracts the major
/// version number. Returns `None` if Java is not found or version
/// cannot be parsed.
///
/// # Format support
/// - OpenJDK:   `openjdk version "21.0.1"` → 21
/// - Oracle:    `java version "21.0.1"`     → 21
/// - GraalVM:   `java version "21.0.1"`     → 21
/// - IBM JDK:   `java version "1.8.0"`      → 8
///
/// Uses std::process::Command (sync, called once at startup).
/// Tries multiple detection strategies for broad platform coverage.
pub fn detect_java_version() -> Option<(u32, String)> {
    // Strategy 1: which crate (Rust-native PATH search)
    if let Some(res) = which::which("java").ok()
        .and_then(|p| run_java_version(&p)) {
        return Some(res);
    }

    // Strategy 2: shell's `command -v` (works on Termux where which::which fails)
    if let Some(res) = find_java_via_shell() {
        return Some(res);
    }

    // Strategy 3: bare `java` (let the OS resolve via inherited PATH)
    run_java_version(&std::path::PathBuf::from("java"))
}

/// Find Java using `sh -c "command -v java"` which uses the shell's
/// PATH resolution. This works on platforms like Termux where
/// `which::which` may fail due to symlink/stat quirks.
fn find_java_via_shell() -> Option<(u32, String)> {
    let output = std::process::Command::new("sh")
        .args(["-c", "command -v java"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let path_str = String::from_utf8_lossy(&output.stdout).to_string();
    let trimmed = path_str.trim();
    if trimmed.is_empty() {
        return None;
    }

    let java_path = std::path::PathBuf::from(trimmed);
    run_java_version(&java_path)
}

fn run_java_version(java_path: &std::path::Path) -> Option<(u32, String)> {
    let output = std::process::Command::new(java_path)
        .arg("--version")
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let first_line = stdout.lines().next()?;

    // Parse major version — capture the first number on the line.
    // Pattern: "openjdk version \"21.0.1\"" → "21"
    // Pattern: "java version \"1.8.0_202\"" → "8" (old format)
    let version_str = first_line
        .split('"')
        .nth(1)
        .or_else(|| first_line.split_whitespace().last())?;

    // Extract major: for "21.0.1" → "21", for "1.8.0" → "8"
    let major_str = if version_str.starts_with("1.") {
        // Java 8 or earlier: "1.8.0_202" → "8"
        version_str.split('.').nth(1)?
    } else {
        version_str.split('.').next()?
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
}
