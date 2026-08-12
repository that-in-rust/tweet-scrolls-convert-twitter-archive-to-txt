//! Tweet-Scrolls: Twitter Archive JSON to CSV/TXT Processor
//!
//! This library provides functionality to process Twitter archive data, analyze interactions,
//! and generate meaningful insights from tweets and direct messages.

#![warn(missing_docs)]

pub mod cli;
pub mod main_integration;
pub mod main_process;
pub mod models;
pub mod processing;
pub mod relationship;
pub mod services;
pub mod thread_export;
pub mod utils;

#[cfg(feature = "gpui-app")]
/// Native GPUI Mac application adapter.
pub mod mac_app;

// Re-exports for common types
pub use models::interaction::*;
pub use services::timeline::*;
