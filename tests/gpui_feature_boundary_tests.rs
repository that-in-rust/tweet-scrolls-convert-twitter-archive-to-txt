use std::{fs, path::PathBuf};

const ZED_GPUI_REVISION: &str = "6bd93fc3195242834f4999f3b3daab294df6b253";

fn read_cargo_manifest_text() -> String {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    fs::read_to_string(manifest).expect("test must read the project Cargo manifest")
}

// TEST-BUILD-001 / REQ-BUILD-001.0
#[test]
fn gpui_feature_boundary_builds() {
    let manifest = read_cargo_manifest_text();

    assert!(manifest.contains("gpui-app = ["));
    assert!(manifest.contains("name = \"tweet-scrolls-mac\""));
    assert!(manifest.contains("required-features = [\"gpui-app\"]"));
    assert!(manifest.contains("optional = true"));
    assert!(manifest.contains("features = [\"font-kit\", \"runtime_shaders\"]"));
}

// TEST-META-002 / REQ-BUILD-001.0
#[test]
fn gpui_dependencies_revision_match() {
    let manifest = read_cargo_manifest_text();

    assert_eq!(manifest.matches(ZED_GPUI_REVISION).count(), 3);
    assert!(!manifest.contains("/Users/amuldotexe/Desktop/oss-read-only/zed-gpui"));
}

// TEST-META-004 / REQ-BUILD-003.0
#[test]
fn native_text_renderer_enabled() {
    let manifest = read_cargo_manifest_text();
    let platform_dependency = manifest
        .lines()
        .find(|line| line.starts_with("gpui_platform = "))
        .expect("the GPUI app must declare its platform dependency");

    assert!(platform_dependency.contains("features = [\"font-kit\", \"runtime_shaders\"]"));
}

// TEST-BUILD-001 / REQ-BUILD-001.0
#[test]
fn rust_toolchain_supports_gpui() {
    let toolchain = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("rust-toolchain.toml");
    let configuration =
        fs::read_to_string(toolchain).expect("the GPUI build must pin a compatible Rust toolchain");

    assert!(configuration.contains("channel = \"1.95.0\""));
    assert!(configuration.contains("components = [\"rustfmt\", \"clippy\"]"));
}

// TEST-STATIC-PRIV-001 / REQ-PRIV-001.0
#[test]
fn export_modules_network_free() {
    let project = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let source_paths = [
        project.join("src/thread_export/archive.rs"),
        project.join("src/thread_export/full_export.rs"),
        project.join("src/thread_export/semantic_parts.rs"),
        project.join("src/mac_app/mod.rs"),
    ];
    let forbidden = ["reqwest", "TcpStream", "UdpSocket", "open_url("];

    for source_path in source_paths {
        let source =
            fs::read_to_string(&source_path).expect("static check must read export source");
        for token in forbidden {
            assert!(
                !source.contains(token),
                "{} must not contain network token {token}",
                source_path.display()
            );
        }
    }
}
