use std::process::Command;

fn git(args: &[&str]) -> String {
    Command::new("git")
        .args(args)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|stdout| stdout.trim().to_string())
        .filter(|stdout| !stdout.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

fn main() {
    println!(
        "cargo:rustc-env=PROJECT_GIT_COMMIT={}",
        git(&["rev-parse", "--short", "HEAD"])
    );
    println!(
        "cargo:rustc-env=PROJECT_GIT_BRANCH={}",
        git(&["rev-parse", "--abbrev-ref", "HEAD"])
    );
    println!("cargo:rerun-if-changed=../../.git/HEAD");
}
