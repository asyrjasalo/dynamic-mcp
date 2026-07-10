//! Cross-platform PATH resolution for spawned stdio subprocesses.
//!
//! Many host processes that launch dynamic-mcp (IDEs, browsers, launchers,
//! minimal service managers) provide a stripped `PATH`. This module
//! reconstructs a complete, de-duplicated PATH from three sources:
//!
//! 1. the inherited `PATH` (from the parent process that started dynamic-mcp),
//! 2. the system + user PATH (platform-specific):
//!      - Windows: `HKLM` + `HKCU` `Environment\Path` registry values
//!        (with `%VAR%` expansion, e.g. `%SystemRoot%`),
//!      - Linux:   `/etc/environment` (`PATH=...`),
//!      - macOS:   `/etc/paths` and files under `/etc/paths.d/`,
//! 3. the `PATH` provided in the server's `env` configuration.
//!
//! The result is joined with the correct platform separator and de-duplicated
//! (case-insensitively on Windows).

use std::collections::HashSet;

#[cfg(windows)]
const PATH_SEP: char = ';';
#[cfg(not(windows))]
const PATH_SEP: char = ':';

/// Build a merged, de-duplicated PATH string.
///
/// `config_path` is the `PATH` value from the server's `env` configuration
/// (may be empty). The returned string uses the platform path separator.
pub fn build_merged_path(config_path: &str) -> String {
    let inherited = std::env::var("PATH").unwrap_or_default();
    merge_paths(&inherited, config_path)
}

/// Core merge logic, separated so tests can inject a synthetic inherited PATH
/// without mutating the global process environment.
fn merge_paths(inherited: &str, config_path: &str) -> String {
    let mut seen: HashSet<String> = HashSet::new();
    let mut ordered: Vec<String> = Vec::new();

    let mut push = |p: &str| {
        let t = p.trim();
        if t.is_empty() {
            return;
        }
        #[cfg(windows)]
        let key = t.to_lowercase();
        #[cfg(not(windows))]
        let key = t.to_string();
        if seen.insert(key) {
            ordered.push(t.to_string());
        }
    };

    // 1. Inherited PATH (from the parent process).
    for part in inherited.split(PATH_SEP) {
        push(part);
    }

    // 2. System + user PATH (platform-specific).
    for part in read_system_path_entries() {
        push(&part);
    }

    // 3. Config-provided PATH (server env).
    for part in config_path.split(PATH_SEP) {
        push(part);
    }

    ordered.join(&PATH_SEP.to_string())
}

#[cfg(windows)]
fn read_system_path_entries() -> Vec<String> {
    let mut out = Vec::new();
    let keys = [
        r"HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\Session Manager\Environment",
        r"HKEY_CURRENT_USER\Environment",
    ];
    for key in keys {
        if let Some(raw) = read_registry_string(key, "Path") {
            let expanded = expand_env_vars(&raw);
            for part in expanded.split(';') {
                let t = part.trim();
                if !t.is_empty() {
                    out.push(t.to_string());
                }
            }
        }
    }
    out
}

#[cfg(windows)]
fn read_registry_string(key: &str, value: &str) -> Option<String> {
    let output = std::process::Command::new("cmd")
        .args(["/c", "reg", "query", key, "/v", value])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        let line = line.trim();
        if line.contains("REG_SZ") || line.contains("REG_EXPAND_SZ") {
            // The value sits after the type token; take the whole remainder
            // (paths may contain spaces, so we must not split on whitespace).
            let idx = line.find("REG_SZ").or_else(|| line.find("REG_EXPAND_SZ"))?;
            let after_type = line[idx..].split_once(char::is_whitespace)?.1;
            let val = after_type.trim();
            if !val.is_empty() {
                return Some(val.to_string());
            }
        }
    }
    None
}

#[cfg(windows)]
fn expand_env_vars(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%' {
            let mut name = String::new();
            let mut closed = false;
            while let Some(&nc) = chars.peek() {
                if nc == '%' {
                    chars.next();
                    closed = true;
                    break;
                }
                name.push(nc);
                chars.next();
            }
            if closed && !name.is_empty() {
                match std::env::var(&name) {
                    Ok(v) => out.push_str(&v),
                    Err(_) => {
                        out.push('%');
                        out.push_str(&name);
                        out.push('%');
                    }
                }
            } else {
                out.push('%');
                out.push_str(&name);
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(target_os = "linux")]
fn read_system_path_entries() -> Vec<String> {
    let mut out = Vec::new();
    if let Ok(content) = std::fs::read_to_string("/etc/environment") {
        for line in content.lines() {
            let line = line.trim();
            if let Some(rest) = line.strip_prefix("PATH=") {
                let val = rest.trim().trim_matches('"');
                for part in val.split(':') {
                    let t = part.trim();
                    if !t.is_empty() {
                        out.push(t.to_string());
                    }
                }
            }
        }
    }
    out
}

#[cfg(target_os = "macos")]
fn read_system_path_entries() -> Vec<String> {
    let mut out = Vec::new();
    if let Ok(content) = std::fs::read_to_string("/etc/paths") {
        for line in content.lines() {
            let t = line.trim();
            if !t.is_empty() && !t.starts_with('#') {
                out.push(t.to_string());
            }
        }
    }
    if let Ok(entries) = std::fs::read_dir("/etc/paths.d") {
        for entry in entries.flatten() {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                for line in content.lines() {
                    let t = line.trim();
                    if !t.is_empty() && !t.starts_with('#') {
                        out.push(t.to_string());
                    }
                }
            }
        }
    }
    out
}

#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
fn read_system_path_entries() -> Vec<String> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merges_inherited_config_and_dedupes() {
        let inherited = if cfg!(windows) { "C:\\a;C:\\b" } else { "/a:/b" };
        let config = if cfg!(windows) { "C:\\b;C:\\c" } else { "/b:/c" };
        let merged = merge_paths(inherited, config);

        assert!(merged.contains(if cfg!(windows) { "C:\\a" } else { "/a" }));
        assert!(merged.contains(if cfg!(windows) { "C:\\c" } else { "/c" }));

        // C:\b / /b appears in both inherited and config -> must be de-duplicated.
        let sep = if cfg!(windows) { ';' } else { ':' };
        let target = if cfg!(windows) { "C:\\b" } else { "/b" };
        let count = merged
            .split(sep)
            .filter(|p| p.trim() == target)
            .count();
        assert_eq!(count, 1, "duplicate PATH entry found: {}", merged);
    }

    #[test]
    fn empty_config_yields_inherited() {
        let inherited = if cfg!(windows) { "C:\\x" } else { "/x" };
        let merged = merge_paths(inherited, "");
        assert!(merged.contains(if cfg!(windows) { "C:\\x" } else { "/x" }));
    }

    #[cfg(windows)]
    #[test]
    fn expands_percent_vars() {
        std::env::set_var("DYNAMIC_MCP_TEST_ROOT", "C:\\winroot");
        let expanded = expand_env_vars("%DYNAMIC_MCP_TEST_ROOT%\\system32");
        assert_eq!(expanded, "C:\\winroot\\system32");
        std::env::remove_var("DYNAMIC_MCP_TEST_ROOT");
    }
}
