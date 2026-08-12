use std::path::Path;

use tempfile::tempdir;
use tweet_scrolls::{
    processing::data_structures::{Thread, Tweet, TweetEntities},
    thread_export::{
        derive_timestamp_output_folder, export_archive_threads_text, render_single_thread_block,
        validate_archive_tweets_input, write_complete_thread_output, CompleteThreadExportRequest,
        ThreadExportFailureKind,
    },
};

fn create_full_export_tweet(id: &str, text: &str, reply_to: Option<&str>) -> Tweet {
    Tweet {
        id_str: id.to_owned(),
        id: id.to_owned(),
        full_text: text.to_owned(),
        created_at: format!("Mon Jan 01 12:0{id}:00 +0000 2024"),
        favorite_count: id.to_owned(),
        retweet_count: id.to_owned(),
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

fn create_full_export_thread(tweets: Vec<Tweet>) -> Thread {
    Thread {
        id: tweets
            .first()
            .map(|tweet| tweet.id_str.clone())
            .unwrap_or_else(|| "empty-thread".to_owned()),
        tweet_count: tweets.len(),
        favorite_count: tweets
            .iter()
            .filter_map(|tweet| tweet.favorite_count.parse::<u32>().ok())
            .sum(),
        retweet_count: tweets
            .iter()
            .filter_map(|tweet| tweet.retweet_count.parse::<u32>().ok())
            .sum(),
        tweets,
    }
}

async fn write_full_archive_fixture(path: &Path, tweets: &[Tweet]) -> anyhow::Result<()> {
    let wrapped = tweets
        .iter()
        .map(|tweet| serde_json::json!({ "tweet": tweet }))
        .collect::<Vec<_>>();
    let script = format!(
        "window.YTD.tweets.part0 = {}",
        serde_json::to_string(&wrapped)?
    );
    tokio::fs::write(path.join("tweets.js"), script).await?;
    Ok(())
}

// TEST-GOLD-TEXT-001 / REQ-TEXT-001.0
#[test]
fn thread_block_rendering_matches() -> anyhow::Result<()> {
    let thread = create_full_export_thread(vec![
        create_full_export_tweet("1", "First line\nSecond line", None),
        create_full_export_tweet("2", "Reply text", Some("1")),
    ]);

    let rendered = render_single_thread_block(&thread)?;

    assert_eq!(
        rendered,
        "--- Start of Thread ---\n\
Thread ID: 1\n\
Timestamp: Mon Jan 01 12:01:00 +0000 2024\n\
Public Support: 1 retweets, 1 likes\n\
Thread text:\n\
- Tweet 1:\n\
First line\nSecond line\n\
\n\
- Tweet 2:\n\
Reply text\n\
\n\
--- End of Thread ---\n\n"
    );
    Ok(())
}

// TEST-NEG-TEXT-002 / REQ-TEXT-002.0
#[test]
fn empty_thread_rendering_fails() {
    let thread = create_full_export_thread(Vec::new());

    let failure = render_single_thread_block(&thread)
        .expect_err("an empty semantic thread must not be indexed");

    assert_eq!(failure.kind(), ThreadExportFailureKind::InvalidThread);
    assert_eq!(failure.thread_id(), Some("empty-thread"));
}

// TEST-UNIT-PATH-001 / REQ-PATH-001.0
#[test]
fn timestamp_output_folder_derives() {
    let archive = Path::new("/tmp/extracted-twitter-archive");

    let output = derive_timestamp_output_folder(archive, 1_723_456_789_012);

    assert_eq!(output, archive.join("tweet-scrolls-1723456789012"));
}

// TEST-INTEG-PATH-002 / REQ-PATH-002.0
#[tokio::test]
async fn existing_output_folder_conflicts() -> anyhow::Result<()> {
    let temporary = tempdir()?;
    write_full_archive_fixture(temporary.path(), &[]).await?;
    let output = derive_timestamp_output_folder(temporary.path(), 42);
    tokio::fs::create_dir(&output).await?;
    let sentinel = output.join("sentinel.bin");
    tokio::fs::write(&sentinel, b"keep-me").await?;
    let archive = validate_archive_tweets_input(temporary.path())?;

    let failure = export_archive_threads_text(CompleteThreadExportRequest::new(archive, 42))
        .await
        .expect_err("an existing run folder must not be merged or overwritten");

    assert_eq!(failure.kind(), ThreadExportFailureKind::OutputCollision);
    assert_eq!(tokio::fs::read(&sentinel).await?, b"keep-me");
    Ok(())
}

// TEST-INTEG-FULL-001 / REQ-FULL-001.0
#[tokio::test]
async fn complete_output_commit_succeeds() -> anyhow::Result<()> {
    let temporary = tempdir()?;
    let tweets = vec![create_full_export_tweet("1", "Complete text", None)];
    write_full_archive_fixture(temporary.path(), &tweets).await?;
    let archive = validate_archive_tweets_input(temporary.path())?;

    let outcome = export_archive_threads_text(CompleteThreadExportRequest::new(archive, 7)).await?;

    assert_eq!(outcome.thread_count(), 1);
    assert!(outcome.byte_count() > 0);
    assert!(outcome.full_output_path().is_file());
    assert!(!outcome
        .output_folder()
        .join("tweet-scrolls-full.txt.partial")
        .exists());
    assert_eq!(
        tokio::fs::metadata(outcome.full_output_path()).await?.len(),
        outcome.byte_count()
    );
    Ok(())
}

// TEST-NEG-FULL-002 / REQ-FULL-001.0
#[tokio::test]
async fn failed_output_commit_cleans() -> anyhow::Result<()> {
    let temporary = tempdir()?;
    let tweets = vec![create_full_export_tweet("1", "Complete text", None)];
    write_full_archive_fixture(temporary.path(), &tweets).await?;
    let archive = validate_archive_tweets_input(temporary.path())?;
    let prepared = tweet_scrolls::thread_export::load_archive_tweet_threads(&archive).await?;
    let output_folder = temporary.path().join("forced-commit-failure");
    tokio::fs::create_dir(&output_folder).await?;
    tokio::fs::create_dir(output_folder.join("tweet-scrolls-full.txt")).await?;

    let failure = write_complete_thread_output(&prepared, &output_folder)
        .await
        .expect_err("a directory at the final filename must force rename failure");

    assert_eq!(failure.kind(), ThreadExportFailureKind::OutputCommit);
    assert!(!output_folder
        .join("tweet-scrolls-full.txt.partial")
        .exists());
    assert!(!output_folder.join("tweet-scrolls-full.txt").is_file());
    Ok(())
}

// TEST-INTEG-FULL-003 / REQ-FULL-002.0
#[tokio::test]
async fn app_artifact_allowlist_matches() -> anyhow::Result<()> {
    let temporary = tempdir()?;
    let tweets = vec![create_full_export_tweet("1", "Only text", None)];
    write_full_archive_fixture(temporary.path(), &tweets).await?;
    let archive = validate_archive_tweets_input(temporary.path())?;

    let outcome = export_archive_threads_text(CompleteThreadExportRequest::new(archive, 8)).await?;
    let entries = std::fs::read_dir(outcome.output_folder())?
        .map(|entry| entry.map(|value| value.file_name()))
        .collect::<Result<Vec<_>, _>>()?;

    assert_eq!(entries, vec!["tweet-scrolls-full.txt"]);
    Ok(())
}

// TEST-INTEG-FULL-004 / REQ-FULL-003.0
#[tokio::test]
async fn empty_export_completion_succeeds() -> anyhow::Result<()> {
    let temporary = tempdir()?;
    write_full_archive_fixture(temporary.path(), &[]).await?;
    let archive = validate_archive_tweets_input(temporary.path())?;

    let outcome = export_archive_threads_text(CompleteThreadExportRequest::new(archive, 9)).await?;

    assert_eq!(outcome.thread_count(), 0);
    assert_eq!(outcome.byte_count(), 0);
    assert_eq!(tokio::fs::read(outcome.full_output_path()).await?, b"");
    assert!(!outcome.has_thread_content());
    Ok(())
}
