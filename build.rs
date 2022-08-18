use chrono::FixedOffset;
use std::process::Command;
fn main() {
    // note: add error checking yourself.
    let output = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .unwrap();
    let git_hash = String::from_utf8(output.stdout).unwrap();
    let date = chrono::offset::Local::now()
        .with_timezone(&FixedOffset::east(7 * 60 * 60))
        .format("%Y:%m:%d-%T");
    //    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
    println!("cargo:rustc-env=BUILD_VERSION={}-{}", date, git_hash);
}
