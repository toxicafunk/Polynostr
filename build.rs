use std::env;
use std::process::Command;

fn main() {
    // Skip aws-lc-sys compiler check for compatibility with older GCC versions.
    // This is needed for Ubuntu 20.04 LTS and similar systems with GCC < 11.
    // The transitive dependency polymarket-client-sdk -> reqwest v0.13 -> aws-lc-sys
    // requires this override since we can't control its crypto provider choice.
    // SAFETY: This is safe to call during build.rs as it runs in a single-threaded
    // context before any other code executes.
    unsafe {
        env::set_var("AWS_LC_SYS_NO_COMPILER_CHECKS", "1");
    }

    // Re-run build script when git metadata changes.
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs");
    println!("cargo:rerun-if-changed=.git/index");

    let hash = Command::new("git")
        .args(["rev-parse", "--short=9", "HEAD"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|stdout| stdout.trim().to_owned())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_owned());

    println!("cargo:rustc-env=GIT_COMMIT_SHORT_HASH={hash}");
}
