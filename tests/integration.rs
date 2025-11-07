//! Integration tests for wlgen-rs CLI.

use std::process::Command;

#[test]
fn test_cli_simple_wordlist() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "-1", "ab", "-2", "12", "?1?2"])
        .output()
        .expect("failed to execute wlgen-rs");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let lines: Vec<&str> = stdout.trim().lines().collect();

    assert_eq!(lines, vec!["a1", "a2", "b1", "b2"]);
}

#[test]
fn test_cli_three_positions() {
    let output = Command::new("cargo")
        .args([
            "run", "--quiet", "--", "-1", "ab", "-2", "12", "-3", "xy", "?1?2?3",
        ])
        .output()
        .expect("failed to execute wlgen-rs");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let lines: Vec<&str> = stdout.trim().lines().collect();

    assert_eq!(
        lines,
        vec!["a1x", "a1y", "a2x", "a2y", "b1x", "b1y", "b2x", "b2y"]
    );
}

#[test]
fn test_cli_repeated_charset() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "-1", "ab", "?1?1"])
        .output()
        .expect("failed to execute wlgen-rs");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let lines: Vec<&str> = stdout.trim().lines().collect();

    assert_eq!(lines, vec!["aa", "ab", "ba", "bb"]);
}

#[test]
fn test_cli_literal_characters() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "-1", "ab", "x?1y"])
        .output()
        .expect("failed to execute wlgen-rs");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let lines: Vec<&str> = stdout.trim().lines().collect();

    assert_eq!(lines, vec!["xay", "xby"]);
}

#[test]
fn test_cli_undefined_charset_error() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "-1", "ab", "?1?2"])
        .output()
        .expect("failed to execute wlgen-rs");

    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("invalid UTF-8");
    assert!(stderr.contains("charset ?2 not defined"));
}

#[test]
fn test_cli_help() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "--help"])
        .output()
        .expect("failed to execute wlgen-rs");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    assert!(stdout.contains("High-performance wordlist generator"));
    assert!(stdout.contains("Usage:"));
}

#[test]
fn test_cli_version() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "--version"])
        .output()
        .expect("failed to execute wlgen-rs");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    assert!(stdout.contains("wlgen-rs"));
}

#[test]
fn test_cli_all_charsets() {
    let output = Command::new("cargo")
        .args([
            "run", "--quiet", "--", "-1", "a", "-2", "b", "-3", "c", "?1?2?3",
        ])
        .output()
        .expect("failed to execute wlgen-rs");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let lines: Vec<&str> = stdout.trim().lines().collect();

    assert_eq!(lines, vec!["abc"]);
}

#[test]
fn test_cli_large_wordlist() {
    // Test with a larger wordlist to verify performance
    let output = Command::new("cargo")
        .args([
            "run",
            "--release",
            "--quiet",
            "--",
            "-1",
            "abc",
            "-2",
            "123",
            "?1?1?2?2",
        ])
        .output()
        .expect("failed to execute wlgen-rs");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let lines: Vec<&str> = stdout.trim().lines().collect();

    // 3^2 * 3^2 = 9 * 9 = 81 combinations
    assert_eq!(lines.len(), 81);

    // Check first and last
    assert_eq!(lines[0], "aa11");
    assert_eq!(lines[80], "cc33");
}

#[test]
fn test_cli_builtin_lowercase() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "?l"])
        .output()
        .expect("failed to execute wlgen-rs");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let lines: Vec<&str> = stdout.trim().lines().collect();

    assert_eq!(lines.len(), 26);
    assert_eq!(lines[0], "a");
    assert_eq!(lines[25], "z");
}

#[test]
fn test_cli_builtin_digits() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "?d?d"])
        .output()
        .expect("failed to execute wlgen-rs");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let lines: Vec<&str> = stdout.trim().lines().collect();

    assert_eq!(lines.len(), 100); // 10 * 10
    assert_eq!(lines[0], "00");
    assert_eq!(lines[99], "99");
}

#[test]
fn test_cli_builtin_mixed() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "?l?d"])
        .output()
        .expect("failed to execute wlgen-rs");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let lines: Vec<&str> = stdout.trim().lines().collect();

    assert_eq!(lines.len(), 260); // 26 * 10
    assert_eq!(lines[0], "a0");
    assert_eq!(lines[259], "z9");
}
