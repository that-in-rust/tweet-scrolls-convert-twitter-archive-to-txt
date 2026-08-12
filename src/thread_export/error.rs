use std::{io, path::PathBuf};

use thiserror::Error;

/// Stable categories that adapters can map to recovery actions and messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadExportFailureKind {
    /// The selected path is not a readable extracted Twitter archive.
    InvalidArchive,
    /// The archive file could not be read.
    ArchiveRead,
    /// The JavaScript wrapper or JSON payload is malformed.
    ArchiveParse,
    /// A tweet timestamp cannot be parsed safely.
    InvalidTimestamp,
    /// A thread model has no tweet to render.
    InvalidThread,
    /// The timestamp-derived output folder already exists.
    OutputCollision,
    /// An output directory or temporary file could not be written.
    OutputWrite,
    /// A temporary output could not be published under its completed name.
    OutputCommit,
    /// Required semantic continuation metadata cannot fit the requested limit.
    PartLimitTooSmall,
    /// A numbered part temporary file could not be written.
    PartWrite,
    /// A numbered part temporary file could not be committed.
    PartCommit,
    /// A blocking worker failed before returning its typed result.
    BackgroundTask,
}

/// Typed failures from the shared tweet-thread export boundary.
#[derive(Debug, Error)]
pub enum ThreadExportFailure {
    /// The selected archive directory does not contain a readable `tweets.js`.
    #[error("invalid Twitter archive at {path}: {message}")]
    InvalidArchive {
        /// Selected path.
        path: PathBuf,
        /// Recovery-oriented explanation.
        message: String,
    },
    /// The validated archive became unreadable while loading.
    #[error("failed to read Twitter archive file {path}: {source}")]
    ArchiveRead {
        /// Archive file path.
        path: PathBuf,
        /// Underlying filesystem failure.
        #[source]
        source: io::Error,
    },
    /// The JavaScript wrapper or JSON payload could not be decoded.
    #[error("failed to parse Twitter archive file {path}: {message}")]
    ArchiveParse {
        /// Archive file path.
        path: PathBuf,
        /// Decoder explanation without archive content.
        message: String,
    },
    /// One tweet contains an invalid archive timestamp.
    #[error("tweet {tweet_id} has invalid timestamp {value:?}")]
    InvalidTimestamp {
        /// Archive tweet identifier.
        tweet_id: String,
        /// Invalid timestamp value.
        value: String,
    },
    /// A thread has no first tweet and therefore cannot be rendered.
    #[error("thread {thread_id} has no tweets")]
    InvalidThread {
        /// Invalid thread identifier.
        thread_id: String,
    },
    /// A timestamp-derived output folder already belongs to another run.
    #[error("output folder already exists: {path}")]
    OutputCollision {
        /// Existing run path.
        path: PathBuf,
    },
    /// An output directory or temporary file could not be written.
    #[error("failed to write output path {path}: {source}")]
    OutputWrite {
        /// Path affected by the write failure.
        path: PathBuf,
        /// Underlying filesystem failure.
        #[source]
        source: io::Error,
    },
    /// An atomic rename could not publish the completed output.
    #[error("failed to commit output from {temporary_path} to {completed_path}: {source}")]
    OutputCommit {
        /// Temporary path being committed.
        temporary_path: PathBuf,
        /// Completed path that was not published.
        completed_path: PathBuf,
        /// Underlying rename failure.
        #[source]
        source: io::Error,
    },
    /// Required semantic continuation metadata cannot fit the requested limit.
    #[error("exclusive part limit {exclusive_limit} is too small; at least {minimum_required} bytes are required")]
    PartLimitTooSmall {
        /// Configured exclusive byte limit.
        exclusive_limit: usize,
        /// Smallest known exclusive limit that can make progress.
        minimum_required: usize,
    },
    /// A numbered part temporary file could not be written.
    #[error("failed to write semantic part {path}: {source}")]
    PartWrite {
        /// Part path affected by the write failure.
        path: PathBuf,
        /// Underlying filesystem failure.
        #[source]
        source: io::Error,
    },
    /// A numbered part temporary file could not be published.
    #[error("failed to commit semantic part from {temporary_path} to {completed_path}: {source}")]
    PartCommit {
        /// Temporary path being committed.
        temporary_path: PathBuf,
        /// Completed part path that was not published.
        completed_path: PathBuf,
        /// Underlying rename failure.
        #[source]
        source: io::Error,
    },
    /// A background worker failed before completing its operation.
    #[error("background archive task failed: {message}")]
    BackgroundTask {
        /// Join failure explanation.
        message: String,
    },
}

impl ThreadExportFailure {
    /// Returns the stable failure category for UI recovery decisions.
    pub fn kind(&self) -> ThreadExportFailureKind {
        match self {
            Self::InvalidArchive { .. } => ThreadExportFailureKind::InvalidArchive,
            Self::ArchiveRead { .. } => ThreadExportFailureKind::ArchiveRead,
            Self::ArchiveParse { .. } => ThreadExportFailureKind::ArchiveParse,
            Self::InvalidTimestamp { .. } => ThreadExportFailureKind::InvalidTimestamp,
            Self::InvalidThread { .. } => ThreadExportFailureKind::InvalidThread,
            Self::OutputCollision { .. } => ThreadExportFailureKind::OutputCollision,
            Self::OutputWrite { .. } => ThreadExportFailureKind::OutputWrite,
            Self::OutputCommit { .. } => ThreadExportFailureKind::OutputCommit,
            Self::PartLimitTooSmall { .. } => ThreadExportFailureKind::PartLimitTooSmall,
            Self::PartWrite { .. } => ThreadExportFailureKind::PartWrite,
            Self::PartCommit { .. } => ThreadExportFailureKind::PartCommit,
            Self::BackgroundTask { .. } => ThreadExportFailureKind::BackgroundTask,
        }
    }

    /// Returns the affected tweet identifier when the failure is tweet-scoped.
    pub fn tweet_id(&self) -> Option<&str> {
        match self {
            Self::InvalidTimestamp { tweet_id, .. } => Some(tweet_id),
            _ => None,
        }
    }

    /// Returns the affected thread identifier when the failure is thread-scoped.
    pub fn thread_id(&self) -> Option<&str> {
        match self {
            Self::InvalidThread { thread_id } => Some(thread_id),
            _ => None,
        }
    }
}
