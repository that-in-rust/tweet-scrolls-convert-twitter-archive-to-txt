use std::path::PathBuf;

use tempfile::tempdir;
use tweet_scrolls::{
    cli::CliConfig,
    processing::{
        data_structures::{Tweet, TweetEntities},
        reply_threads::process_reply_threads,
        tweets::process_tweets,
    },
};

fn create_characterized_test_tweet(
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
        in_reply_to_screen_name: reply_to.map(|_| "user".to_owned()),
        edit_info: None,
        entities: TweetEntities::default(),
        possibly_sensitive: None,
    }
}

fn canonicalize_thread_identifier_groups(threads: &[Vec<Tweet>]) -> Vec<Vec<String>> {
    let mut groups = threads
        .iter()
        .map(|thread| {
            thread
                .iter()
                .map(|tweet| tweet.id_str.clone())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    groups.sort();
    groups
}

// TEST-CHAR-CORE-002 / REQ-CORE-002.0
#[test]
fn legacy_thread_groups_match() {
    let tweets = vec![
        create_characterized_test_tweet("1", "Mon Jan 01 12:00:00 +0000 2024", None, false),
        create_characterized_test_tweet("2", "Mon Jan 01 12:01:00 +0000 2024", Some("1"), false),
        create_characterized_test_tweet("3", "Mon Jan 01 12:02:00 +0000 2024", None, false),
    ];

    let threads = process_reply_threads(&tweets, "user");

    assert_eq!(
        canonicalize_thread_identifier_groups(&threads),
        vec![vec!["1".to_owned(), "2".to_owned()], vec!["3".to_owned()]]
    );
}

// TEST-CHAR-CLI-001 / REQ-CLI-001.0
#[test]
fn cli_invocation_contract_remains() {
    let archive_folder = PathBuf::from("/tmp/archive");
    let explicit_output = PathBuf::from("/tmp/output");

    let default_config = CliConfig {
        archive_folder: archive_folder.clone(),
        output_dir: None,
        non_interactive: true,
    };
    let explicit_config = CliConfig {
        archive_folder,
        output_dir: Some(explicit_output.clone()),
        non_interactive: true,
    };

    assert_eq!(
        default_config.get_output_dir("user", 42),
        PathBuf::from("/tmp/archive/output_user_42")
    );
    assert_eq!(explicit_config.get_output_dir("user", 42), explicit_output);
}

// TEST-CHAR-CLI-002 / REQ-CLI-002.0
#[tokio::test]
async fn cli_artifact_contract_remains() -> anyhow::Result<()> {
    let temporary = tempdir()?;
    let tweets_path = temporary.path().join("tweets.js");
    let output_path = temporary.path().join("output");
    tokio::fs::create_dir(&output_path).await?;

    let tweets = vec![
        create_characterized_test_tweet("1", "Mon Jan 01 12:00:00 +0000 2024", None, false),
        create_characterized_test_tweet("2", "Mon Jan 01 12:01:00 +0000 2024", None, true),
    ];
    let wrapped = tweets
        .into_iter()
        .map(|tweet| serde_json::json!({ "tweet": tweet }))
        .collect::<Vec<_>>();
    let script = format!(
        "window.YTD.tweets.part0 = {}",
        serde_json::to_string(&wrapped)?
    );
    tokio::fs::write(&tweets_path, script).await?;

    process_tweets(
        tweets_path.to_string_lossy().as_ref(),
        "user",
        &output_path,
        42,
    )
    .await?;

    let mut entries = std::fs::read_dir(&output_path)?
        .map(|entry| entry.map(|value| value.file_name().to_string_lossy().into_owned()))
        .collect::<Result<Vec<_>, _>>()?;
    entries.sort();

    assert_eq!(entries.len(), 3);
    assert!(entries
        .iter()
        .any(|name| name.starts_with("threads_user_") && name.ends_with(".txt")));
    assert!(entries
        .iter()
        .any(|name| name.starts_with("threads_user_") && name.ends_with(".csv")));
    assert!(entries
        .iter()
        .any(|name| name.starts_with("results_user_") && name.ends_with(".txt")));

    let thread_file = entries
        .iter()
        .find(|name| name.starts_with("threads_user_") && name.ends_with(".txt"))
        .expect("characterization fixture must create a thread text file");
    let rendered = tokio::fs::read_to_string(output_path.join(thread_file)).await?;
    assert!(rendered.contains("tweet-1"));
    assert!(!rendered.contains("tweet-2"));

    Ok(())
}
