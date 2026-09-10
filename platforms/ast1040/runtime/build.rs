/*++

Licensed under the Apache-2.0 license.

File Name:

    build.rs

Abstract:

    Cargo build file

--*/

use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=layout.ld");
    println!("cargo:rerun-if-changed=kernel_layout.ld");

    // Force this script to run on every build (not just when the files above
    // change) so the boot banner's git version and timestamp stay current.
    println!("cargo:rerun-if-changed=build.rs.never-generated");

    println!("cargo:rustc-env=MCU_GIT_VERSION={}", git_version());
    println!("cargo:rustc-env=MCU_BUILD_TIMESTAMP={}", build_timestamp());
}

/// `<branch>-<short-hash>[-dirty]`, e.g. `aspeed-dev-2.1-rt-8eacc41c-dirty`,
/// or "unknown" if git isn't available (e.g. building from a source tarball).
fn git_version() -> String {
    match (git_branch(), git_describe()) {
        (Some(branch), Some(rev)) => format!("{branch}-{rev}"),
        (Some(branch), None) => branch,
        (None, Some(rev)) => rev,
        (None, None) => "unknown".to_string(),
    }
}

fn git_branch() -> Option<String> {
    run_cmd("git", &["rev-parse", "--abbrev-ref", "HEAD"])
}

fn git_describe() -> Option<String> {
    run_cmd("git", &["describe", "--always", "--dirty"])
}

/// U-Boot-style build timestamp, e.g. "Sep  8 2026 - 09:15:32 +0000". Falls
/// back to "unknown" if the host `date` command is unavailable.
fn build_timestamp() -> String {
    Command::new("date")
        .env("LC_ALL", "C")
        .arg("+%b %e %Y - %H:%M:%S %z")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

fn run_cmd(cmd: &str, args: &[&str]) -> Option<String> {
    Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}
