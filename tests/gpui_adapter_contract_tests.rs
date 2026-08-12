#![cfg(feature = "gpui-app")]

use std::{fs, path::PathBuf};

use tweet_scrolls::{
    mac_app::{apply_archive_selection_result, archive_folder_prompt_options},
    thread_export::ThreadExportViewState,
};

// TEST-GPUI-APP-001 / REQ-APP-001.0
#[test]
fn folder_picker_config_matches() {
    let options = archive_folder_prompt_options();

    assert!(!options.files);
    assert!(options.directories);
    assert!(!options.multiple);
}

// TEST-GPUI-APP-002 / REQ-APP-001.0
#[test]
fn folder_picker_cancel_stays() {
    let state = apply_archive_selection_result(ThreadExportViewState::NeedsArchive, None);

    assert!(matches!(state, ThreadExportViewState::NeedsArchive));
}

// TEST-GPUI-APP-009 / REQ-APP-006.0
#[test]
fn primary_actions_are_accessible() {
    let source_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/mac_app/mod.rs");
    let source = fs::read_to_string(source_path).expect("test must read the GPUI adapter source");

    assert!(source.contains(".role(Role::Button)"));
    assert!(source.contains(".aria_label(label)"));
    assert!(source.contains(".tab_index(0)"));
}
