use std::{fs, path::PathBuf};

fn project_root_directory_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

// TEST-SMOKE-003 / REQ-BUILD-002.0
#[test]
fn mac_bundle_metadata_matches() {
    let project = project_root_directory_path();
    let plist = fs::read_to_string(project.join("packaging/macos/Info.plist"))
        .expect("the Mac app must include bundle metadata");

    assert!(plist.contains("<string>tweet-scrolls-mac</string>"));
    assert!(plist.contains("<string>Tweet Scrolls</string>"));
    assert!(plist.contains("<string>com.amuldotexe.tweetscrolls</string>"));
    assert!(plist.contains("<string>0.0.2</string>"));
}

// TEST-SMOKE-003 / REQ-BUILD-002.0
#[test]
fn package_script_contract_matches() {
    let project = project_root_directory_path();
    let script = fs::read_to_string(project.join("scripts/package_mac_app.sh"))
        .expect("the Mac app must include a repeatable packaging script");

    assert!(script.contains("cargo build --release --features gpui-app --bin tweet-scrolls-mac"));
    assert!(script.contains("MACOS_DIR=\"$APP_DIR/Contents/MacOS\""));
    assert!(script.contains("\"$MACOS_DIR/tweet-scrolls-mac\""));
    assert!(script.contains("Contents/Info.plist"));
}

// REQ-BUILD-002.0
#[test]
fn product_version_contract_matches() {
    let project = project_root_directory_path();
    let manifest = fs::read_to_string(project.join("Cargo.toml"))
        .expect("version contract must read the Cargo manifest");

    assert!(manifest.contains("version = \"0.0.2\""));
}
