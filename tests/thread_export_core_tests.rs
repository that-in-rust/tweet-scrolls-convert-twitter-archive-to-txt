use std::path::Path;

use tempfile::tempdir;
use tweet_scrolls::{
    processing::data_structures::{Tweet, TweetEntities},
    thread_export::{
        load_archive_tweet_threads, validate_archive_tweets_input, ThreadExportFailureKind,
    },
};

fn create_archive_fixture_tweet(
    id: &str,
    created_at: &str,
    reply_to: Option<&str>,
    retweeted: bool,
) -> Tweet {
    Tweet {
        id_str: id.to_owned(),
        id: id.to_owned(),
        full_text: format!("tweet-{id}"),
        created_at: created_at.to_owned(),
        favorite_count: id.to_owned(),
        retweet_count: id.to_owned(),
        retweeted,
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

async fn write_archive_fixture_file(path: &Path, tweets: &[Tweet]) -> anyhow::Result<()> {
    let wrapped = tweets
        .iter()
        .map(|tweet| serde_json::json!({ "tweet": tweet }))
        .collect::<Vec<_>>();
    let script = format!(
        "window.YTD.tweets.part0 = {};\n",
        serde_json::to_string(&wrapped)?
    );
    tokio::fs::write(path.join("tweets.js"), script).await?;
    Ok(())
}

fn canonical_thread_identifier_groups(
    prepared: &tweet_scrolls::thread_export::PreparedThreadExportData,
) -> Vec<Vec<String>> {
    let mut groups = prepared
        .threads()
        .iter()
        .map(|thread| {
            thread
                .tweets
                .iter()
                .map(|tweet| tweet.id_str.clone())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    groups.sort();
    groups
}

// TEST-UNIT-ARCH-001 / REQ-ARCH-001.0
#[tokio::test]
async fn archive_folder_validation_succeeds() -> anyhow::Result<()> {
    let temporary = tempdir()?;
    write_archive_fixture_file(temporary.path(), &[]).await?;

    let archive = validate_archive_tweets_input(temporary.path())?;

    assert_eq!(archive.archive_folder(), temporary.path());
    assert_eq!(archive.tweets_file(), temporary.path().join("tweets.js"));
    Ok(())
}

// TEST-NEG-ARCH-002 / REQ-ARCH-001.0
#[test]
fn archive_folder_validation_fails() -> anyhow::Result<()> {
    let temporary = tempdir()?;
    let selected_file = temporary.path().join("not-a-folder");
    std::fs::write(&selected_file, "fixture")?;

    for invalid_path in [selected_file, temporary.path().join("missing-folder")] {
        let failure = validate_archive_tweets_input(&invalid_path)
            .expect_err("invalid archive selections must fail validation");
        assert_eq!(failure.kind(), ThreadExportFailureKind::InvalidArchive);
    }

    assert_eq!(std::fs::read_dir(temporary.path())?.count(), 1);
    Ok(())
}

// TEST-UNIT-ARCH-003 / REQ-ARCH-002.0
#[tokio::test]
async fn javascript_archive_parsing_succeeds() -> anyhow::Result<()> {
    let temporary = tempdir()?;
    let tweets = vec![create_archive_fixture_tweet(
        "1",
        "Mon Jan 01 12:00:00 +0000 2024",
        None,
        false,
    )];
    write_archive_fixture_file(temporary.path(), &tweets).await?;
    let archive = validate_archive_tweets_input(temporary.path())?;

    let prepared = load_archive_tweet_threads(&archive).await?;

    assert_eq!(prepared.source_tweet_count(), 1);
    assert_eq!(prepared.excluded_retweet_count(), 0);
    assert_eq!(
        canonical_thread_identifier_groups(&prepared),
        vec![vec!["1".to_owned()]]
    );
    Ok(())
}

// TEST-UNIT-ARCH-005 / REQ-ARCH-002.0
#[tokio::test]
async fn missing_archive_metadata_parses() -> anyhow::Result<()> {
    let temporary = tempdir()?;
    let mut tweet = serde_json::to_value(create_archive_fixture_tweet(
        "1",
        "Mon Jan 01 12:00:00 +0000 2024",
        None,
        false,
    ))?;
    tweet["entities"]["urls"] = serde_json::json!([{
        "url": "https://t.co/example",
        "indices": ["0", "20"]
    }]);
    tweet
        .as_object_mut()
        .expect("serialized tweet fixture must be an object")
        .remove("lang");
    let script = format!(
        "window.YTD.tweets.part0 = {};\n",
        serde_json::to_string(&vec![serde_json::json!({ "tweet": tweet })])?
    );
    tokio::fs::write(temporary.path().join("tweets.js"), script).await?;
    let archive = validate_archive_tweets_input(temporary.path())?;

    let prepared = load_archive_tweet_threads(&archive).await?;

    assert_eq!(prepared.source_tweet_count(), 1);
    assert_eq!(prepared.threads()[0].tweets[0].entities.urls.len(), 1);
    Ok(())
}

// TEST-NEG-ARCH-004 / REQ-ARCH-002.0
#[tokio::test]
async fn javascript_archive_parsing_fails() -> anyhow::Result<()> {
    for malformed in [
        "window.YTD.tweets.part0 = no-array",
        "window.YTD.tweets.part0 = [{ definitely-not-json }]",
    ] {
        let temporary = tempdir()?;
        tokio::fs::write(temporary.path().join("tweets.js"), malformed).await?;
        let archive = validate_archive_tweets_input(temporary.path())?;

        let failure = load_archive_tweet_threads(&archive)
            .await
            .expect_err("malformed archive content must return a typed error");

        assert_eq!(failure.kind(), ThreadExportFailureKind::ArchiveParse);
    }
    Ok(())
}

// TEST-UNIT-CORE-001 / REQ-CORE-001.0
#[tokio::test]
async fn current_thread_semantics_match() -> anyhow::Result<()> {
    let temporary = tempdir()?;
    let tweets = vec![
        create_archive_fixture_tweet("1", "Mon Jan 01 12:00:00 +0000 2024", None, false),
        create_archive_fixture_tweet("2", "Mon Jan 01 12:01:00 +0000 2024", Some("1"), false),
        create_archive_fixture_tweet("3", "Mon Jan 01 12:02:00 +0000 2024", None, false),
        create_archive_fixture_tweet("4", "Mon Jan 01 12:03:00 +0000 2024", None, true),
    ];
    write_archive_fixture_file(temporary.path(), &tweets).await?;
    let archive = validate_archive_tweets_input(temporary.path())?;

    let prepared = load_archive_tweet_threads(&archive).await?;

    assert_eq!(prepared.source_tweet_count(), 4);
    assert_eq!(prepared.excluded_retweet_count(), 1);
    assert_eq!(
        canonical_thread_identifier_groups(&prepared),
        vec![vec!["1".to_owned(), "2".to_owned()], vec!["3".to_owned()]]
    );
    let connected = prepared
        .threads()
        .iter()
        .find(|thread| thread.id == "1")
        .expect("the connected fixture thread must exist");
    assert_eq!(connected.favorite_count, 3);
    assert_eq!(connected.retweet_count, 3);
    Ok(())
}

// TEST-NEG-CORE-003 / REQ-CORE-003.0
#[tokio::test]
async fn invalid_timestamp_returns_error() -> anyhow::Result<()> {
    let temporary = tempdir()?;
    let tweets = vec![create_archive_fixture_tweet(
        "broken",
        "not-a-date",
        None,
        false,
    )];
    write_archive_fixture_file(temporary.path(), &tweets).await?;
    let archive = validate_archive_tweets_input(temporary.path())?;

    let failure = load_archive_tweet_threads(&archive)
        .await
        .expect_err("invalid timestamps must not panic");

    assert_eq!(failure.kind(), ThreadExportFailureKind::InvalidTimestamp);
    assert_eq!(failure.tweet_id(), Some("broken"));
    Ok(())
}

// TEST-UNIT-CORE-004 / REQ-CORE-004.0
#[tokio::test]
async fn empty_archive_exports_zero() -> anyhow::Result<()> {
    for tweets in [
        Vec::new(),
        vec![create_archive_fixture_tweet(
            "1",
            "Mon Jan 01 12:00:00 +0000 2024",
            None,
            true,
        )],
    ] {
        let temporary = tempdir()?;
        write_archive_fixture_file(temporary.path(), &tweets).await?;
        let archive = validate_archive_tweets_input(temporary.path())?;

        let prepared = load_archive_tweet_threads(&archive).await?;

        assert!(prepared.threads().is_empty());
    }
    Ok(())
}

// TEST-UNIT-CORE-001 / REQ-CORE-001.0
#[tokio::test]
async fn thread_tweets_sort_chronologically() -> anyhow::Result<()> {
    let temporary = tempdir()?;
    let tweets = vec![
        create_archive_fixture_tweet("1", "Wed Jan 31 23:59:00 +0000 2024", None, false),
        create_archive_fixture_tweet("2", "Thu Feb 01 00:01:00 +0000 2024", Some("1"), false),
    ];
    write_archive_fixture_file(temporary.path(), &tweets).await?;
    let archive = validate_archive_tweets_input(temporary.path())?;

    let prepared = load_archive_tweet_threads(&archive).await?;
    let identifiers = prepared.threads()[0]
        .tweets
        .iter()
        .map(|tweet| tweet.id_str.as_str())
        .collect::<Vec<_>>();

    assert_eq!(identifiers, vec!["1", "2"]);
    assert_eq!(prepared.threads()[0].id, "1");
    Ok(())
}
