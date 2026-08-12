use std::path::Path;

use tempfile::tempdir;
use tweet_scrolls::{
    processing::data_structures::{Tweet, TweetEntities},
    thread_export::{
        export_archive_threads_text, plan_semantic_thread_parts, validate_archive_tweets_input,
        write_optional_thread_parts, CompleteThreadExportOutcome, CompleteThreadExportRequest,
        SemanticPartExportRequest, ThreadExportFailureKind,
    },
};

fn create_parts_export_tweet(id: &str, text: impl Into<String>, reply_to: Option<&str>) -> Tweet {
    Tweet {
        id_str: id.to_owned(),
        id: id.to_owned(),
        full_text: text.into(),
        created_at: format!("Mon Jan 01 12:0{id}:00 +0000 2024"),
        favorite_count: "0".to_owned(),
        retweet_count: "0".to_owned(),
        retweeted: false,
        favorited: false,
        truncated: false,
        lang: "en".to_owned(),
        source: "test".to_owned(),
        display_text_range: Vec::new(),
        in_reply_to_status_id: reply_to.map(str::to_owned),
        in_reply_to_status_id_str: reply_to.map(str::to_owned),
        in_reply_to_user_id: None,
        in_reply_to_user_id_str: None,
        in_reply_to_screen_name: reply_to.map(|_| "fixture-user".to_owned()),
        edit_info: None,
        entities: TweetEntities::default(),
        possibly_sensitive: None,
    }
}

async fn create_parts_export_outcome(
    archive_folder: &Path,
    tweets: &[Tweet],
    timestamp: i64,
) -> anyhow::Result<CompleteThreadExportOutcome> {
    let wrapped = tweets
        .iter()
        .map(|tweet| serde_json::json!({ "tweet": tweet }))
        .collect::<Vec<_>>();
    let script = format!(
        "window.YTD.tweets.part0 = {}",
        serde_json::to_string(&wrapped)?
    );
    tokio::fs::write(archive_folder.join("tweets.js"), script).await?;
    let archive = validate_archive_tweets_input(archive_folder)?;
    Ok(export_archive_threads_text(CompleteThreadExportRequest::new(archive, timestamp)).await?)
}

async fn read_numbered_part_payloads(
    outcome: &CompleteThreadExportOutcome,
) -> anyhow::Result<Vec<String>> {
    let mut paths = std::fs::read_dir(outcome.output_folder())?
        .filter_map(|entry| entry.ok().map(|value| value.path()))
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| {
                    name.starts_with("tweet-scrolls-part-") && name.ends_with(".txt")
                })
        })
        .collect::<Vec<_>>();
    paths.sort();
    let mut payloads = Vec::with_capacity(paths.len());
    for path in paths {
        payloads.push(tokio::fs::read_to_string(path).await?);
    }
    Ok(payloads)
}

// TEST-PROP-PART-003 / REQ-PART-003.0
#[tokio::test]
async fn semantic_parts_respect_limit() -> anyhow::Result<()> {
    let temporary = tempdir()?;
    let tweets = vec![
        create_parts_export_tweet("1", format!("marker-a {}", "a".repeat(90)), None),
        create_parts_export_tweet("2", format!("marker-b {}", "b".repeat(90)), None),
        create_parts_export_tweet("3", format!("marker-c {}", "c".repeat(90)), None),
    ];
    let outcome = create_parts_export_outcome(temporary.path(), &tweets, 11).await?;
    let limit = 300;
    let request = SemanticPartExportRequest::with_limit(&outcome, limit);

    let result = write_optional_thread_parts(&request).await?;
    let payloads = read_numbered_part_payloads(&outcome).await?;
    let full_payload = tokio::fs::read_to_string(outcome.full_output_path()).await?;

    assert_eq!(result.part_count(), payloads.len());
    assert!(payloads.len() > 1);
    assert!(payloads
        .iter()
        .all(|payload| !payload.is_empty() && payload.len() < limit));
    let combined = payloads.join("");
    let mut full_order = ["marker-a", "marker-b", "marker-c"];
    full_order.sort_by_key(|marker| full_payload.find(marker));
    let mut parts_order = ["marker-a", "marker-b", "marker-c"];
    parts_order.sort_by_key(|marker| combined.find(marker));
    assert_eq!(parts_order, full_order);
    Ok(())
}

// TEST-UNIT-PART-004 / REQ-PART-003.0
#[tokio::test]
async fn small_export_creates_one() -> anyhow::Result<()> {
    let temporary = tempdir()?;
    let tweets = vec![create_parts_export_tweet("1", "small text", None)];
    let outcome = create_parts_export_outcome(temporary.path(), &tweets, 12).await?;
    let full_payload = tokio::fs::read_to_string(outcome.full_output_path()).await?;
    let request = SemanticPartExportRequest::with_limit(&outcome, 10_000);

    let result = write_optional_thread_parts(&request).await?;
    let payloads = read_numbered_part_payloads(&outcome).await?;

    assert_eq!(result.part_count(), 1);
    assert_eq!(payloads, vec![full_payload]);
    assert!(result.part_paths()[0].ends_with("tweet-scrolls-part-001.txt"));
    Ok(())
}

// TEST-UNIT-PART-005 / REQ-PART-004.0
#[tokio::test]
async fn oversized_thread_splits_tweets() -> anyhow::Result<()> {
    let temporary = tempdir()?;
    let tweets = vec![
        create_parts_export_tweet("1", "a".repeat(220), None),
        create_parts_export_tweet("2", "b".repeat(220), Some("1")),
    ];
    let outcome = create_parts_export_outcome(temporary.path(), &tweets, 13).await?;
    let request = SemanticPartExportRequest::with_limit(&outcome, 450);

    write_optional_thread_parts(&request).await?;
    let payloads = read_numbered_part_payloads(&outcome).await?;

    assert!(payloads.len() >= 2);
    assert!(payloads.iter().all(|payload| {
        payload.contains("Thread ID: 1") && payload.contains("Thread continuation:")
    }));
    assert!(payloads.join("").find(&"a".repeat(220)).is_some());
    assert!(payloads.join("").find(&"b".repeat(220)).is_some());
    Ok(())
}

// TEST-PROP-PART-006 / REQ-PART-004.0
#[tokio::test]
async fn oversized_tweet_splits_utf8() -> anyhow::Result<()> {
    let temporary = tempdir()?;
    let unicode_text = "🙂".repeat(300);
    let tweets = vec![create_parts_export_tweet("1", &unicode_text, None)];
    let outcome = create_parts_export_outcome(temporary.path(), &tweets, 14).await?;
    let request = SemanticPartExportRequest::with_limit(&outcome, 350);

    write_optional_thread_parts(&request).await?;
    let payloads = read_numbered_part_payloads(&outcome).await?;

    assert!(payloads.len() > 1);
    assert!(payloads
        .iter()
        .all(|payload| payload.len() < 350 && payload.contains("Tweet text continuation:")));
    assert_eq!(payloads.join("").matches('🙂').count(), 300);
    Ok(())
}

// TEST-NEG-PART-007 / REQ-PART-004.0
#[tokio::test]
async fn impossible_part_limit_fails() -> anyhow::Result<()> {
    let temporary = tempdir()?;
    let tweets = vec![create_parts_export_tweet("1", "cannot fit", None)];
    let outcome = create_parts_export_outcome(temporary.path(), &tweets, 15).await?;

    let failure = plan_semantic_thread_parts(outcome.prepared_data(), 40)
        .expect_err("a limit below required metadata must fail without looping");

    assert_eq!(failure.kind(), ThreadExportFailureKind::PartLimitTooSmall);
    Ok(())
}

// TEST-INTEG-PART-008 / REQ-PART-005.0
#[tokio::test]
async fn failed_parts_preserve_full() -> anyhow::Result<()> {
    let temporary = tempdir()?;
    let tweets = vec![create_parts_export_tweet("1", "full survives", None)];
    let outcome = create_parts_export_outcome(temporary.path(), &tweets, 16).await?;
    let full_before = tokio::fs::read(outcome.full_output_path()).await?;
    tokio::fs::create_dir(outcome.output_folder().join("tweet-scrolls-part-001.txt")).await?;
    let request = SemanticPartExportRequest::with_limit(&outcome, 10_000);

    let failure = write_optional_thread_parts(&request)
        .await
        .expect_err("the forced commit conflict must fail the parts attempt");

    assert_eq!(failure.kind(), ThreadExportFailureKind::PartCommit);
    assert_eq!(
        tokio::fs::read(outcome.full_output_path()).await?,
        full_before
    );
    assert!(!outcome
        .output_folder()
        .join("tweet-scrolls-part-001.txt.partial")
        .exists());
    assert!(!outcome
        .output_folder()
        .join("tweet-scrolls-part-001.txt")
        .is_file());
    Ok(())
}

// TEST-INTEG-PART-009 / REQ-PART-005.0
#[tokio::test]
async fn retried_parts_cleanup_stale() -> anyhow::Result<()> {
    let temporary = tempdir()?;
    let tweets = vec![create_parts_export_tweet("1", "retry succeeds", None)];
    let outcome = create_parts_export_outcome(temporary.path(), &tweets, 17).await?;
    let stale_final = outcome.output_folder().join("tweet-scrolls-part-009.txt");
    let stale_partial = outcome
        .output_folder()
        .join("tweet-scrolls-part-009.txt.partial");
    tokio::fs::write(&stale_final, b"stale-final").await?;
    tokio::fs::write(&stale_partial, b"stale-partial").await?;
    let request = SemanticPartExportRequest::with_limit(&outcome, 10_000);

    let result = write_optional_thread_parts(&request).await?;

    assert_eq!(result.part_count(), 1);
    assert!(!stale_final.exists());
    assert!(!stale_partial.exists());
    assert!(outcome
        .output_folder()
        .join("tweet-scrolls-part-001.txt")
        .is_file());
    Ok(())
}
