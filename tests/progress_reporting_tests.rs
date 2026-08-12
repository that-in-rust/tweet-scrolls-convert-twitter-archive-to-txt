#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    struct MockProgressBar {
        pub messages: Arc<Mutex<Vec<String>>>,
    }
    impl MockProgressBar {
        pub fn new() -> Self {
            Self { messages: Arc::new(Mutex::new(Vec::new())) }
        }
        pub fn set_message(&self, msg: String) {
            self.messages.lock().unwrap().push(msg);
        }
        pub fn inc(&self, _n: u64) {}
        pub fn finish_with_message(&self, msg: String) {
            self.messages.lock().unwrap().push(msg);
        }
    }

    fn simulate_progress_reporting(num_tweets: usize) -> Vec<String> {
        let pb = MockProgressBar::new();
        let mut original_tweet_count = 0;
        let mut reply_count = 0;
        let mut retweet_count = 0;
        let mut total_tweets = 0;
        let mut logs = Vec::new();
        for i in 0..num_tweets {
            total_tweets += 1;
            // Simulate tweet type
            if i % 3 == 0 {
                retweet_count += 1;
            } else if i % 3 == 1 {
                reply_count += 1;
            } else {
                original_tweet_count += 1;
            }
            if total_tweets % 10 == 0 {
                pb.set_message(format!("{} originals, {} replies, {} retweets", original_tweet_count, reply_count, retweet_count));
                logs.push(format!("[PROGRESS] Processed {} tweets: {} originals, {} replies, {} retweets", total_tweets, original_tweet_count, reply_count, retweet_count));
            }
            pb.inc(1);
        }
        pb.finish_with_message(format!("{} originals, {} replies, {} retweets", original_tweet_count, reply_count, retweet_count));
        logs
    }

    #[test]
    fn test_progress_reporting_every_10_tweets() {
    let logs = simulate_progress_reporting(25);
    assert_eq!(logs.len(), 2);
    // The simulated logic assigns retweet for i%3==0, reply for i%3==1, original for i%3==2
    // So after 10 tweets: 4 retweets, 3 replies, 3 originals
    assert_eq!(logs[0], "[PROGRESS] Processed 10 tweets: 3 originals, 3 replies, 4 retweets");
    // After 20 tweets: 7 originals, 7 replies, 6 retweets
    assert_eq!(logs[1], "[PROGRESS] Processed 20 tweets: 6 originals, 7 replies, 7 retweets");
    }

    #[test]
    fn test_progress_reporting_with_less_than_10_tweets() {
        let logs = simulate_progress_reporting(9);
        assert_eq!(logs.len(), 0);
    }

    #[test]
    fn test_progress_reporting_with_exact_multiple() {
        let logs = simulate_progress_reporting(30);
        assert_eq!(logs.len(), 3);
        assert_eq!(logs[2], "[PROGRESS] Processed 30 tweets: 10 originals, 10 replies, 10 retweets");
    }
}
