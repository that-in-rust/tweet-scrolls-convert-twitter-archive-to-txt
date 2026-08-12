#![cfg(feature = "gpui-app")]

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
