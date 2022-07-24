use async_trait::async_trait;
use futures::{future, StreamExt, TryFutureExt};
use rss::{Channel, Item};

use crate::{Config, RssClient};

#[async_trait]
pub trait Target {
    async fn publish<'a>(&self, posts: &[Item]) -> Result<(), Box<dyn std::error::Error + 'a>>;
}

/// Orchestrates syndication
pub async fn syndicate<'rss_client>(
    config: &Config,
    rss_client: Box<dyn RssClient + 'rss_client>,
    targets: &[Box<dyn Target>],
) -> Result<(), Box<dyn std::error::Error + 'rss_client>> {
    log::debug!("Received config: {:?}", config);
    let results = futures::stream::iter(config.rss.urls.iter())
        .map(|url| {
            rss_client
                .get_channel(url)
                .and_then(|channel| syndycate_channel(channel, targets))
        })
        .buffer_unordered(10)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<(), Box<dyn std::error::Error>>>();

    results
}

/// Syndicates a single channel
async fn syndycate_channel<'a>(
    channel: Channel,
    targets: &[Box<dyn Target>],
) -> Result<(), Box<dyn std::error::Error + 'a>> {
    let results = targets.iter().map(|target| target.publish(&channel.items));

    // TODO, this compiles
    future::try_join_all(results).map_ok(|_| ()).await
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use async_mutex::Mutex;
    use async_trait::async_trait;
    use rss::Item;

    use crate::stubs::rss::{default_items, StubRssClient};
    use crate::{config::RSSConfig, Config};

    use super::{syndicate, Target};

    #[derive(Default)]
    struct StubTarget {
        calls: Arc<Mutex<Vec<Vec<Item>>>>,
    }

    #[async_trait]
    impl Target for StubTarget {
        async fn publish<'a>(&self, posts: &[Item]) -> Result<(), Box<dyn std::error::Error + 'a>> {
            let mut calls = self.calls.lock().await;
            calls.push(posts.to_vec());
            Ok(())
        }
    }

    impl From<StubTarget> for Box<dyn Target> {
        fn from(stub_target: StubTarget) -> Self {
            Box::new(stub_target)
        }
    }

    #[tokio::test]
    async fn test_syndycate_fetches_a_feed() {
        let feed = "http://example.com/rss.xml";
        let config = Config {
            rss: RSSConfig {
                urls: vec![feed.to_string()],
            },
        };

        let client = StubRssClient::default();
        let client_calls = Arc::clone(&client.urls);
        let stub_target = StubTarget::default();
        let targets = vec![stub_target.into()];

        syndicate(&config, Box::new(client), &targets)
            .await
            .expect("Should be Ok()");

        let calls = (*client_calls).lock().await;

        assert_eq!(*calls, vec![feed])
    }

    #[tokio::test]
    async fn test_syndycate_fetches_multiple_feeds() {
        let feed1 = "http://example.com/rss.xml";
        let feed2 = "https://blog.example.com/rss.xml";
        let config = Config {
            rss: RSSConfig {
                urls: vec![feed1.to_string(), feed2.to_string()],
            },
        };

        let client = StubRssClient::default();
        let client_calls = Arc::clone(&client.urls);
        let stub_target = StubTarget::default();
        let targets = vec![stub_target.into()];

        syndicate(&config, Box::new(client), &targets)
            .await
            .expect("Should be Ok()");

        let calls = (*client_calls).lock().await;

        assert_eq!(*calls, vec![feed1, feed2])
    }

    #[tokio::test]
    async fn test_syndycate_publishes_posts_to_targets() {
        let feed = "http://example.com/rss.xml";
        let config = Config {
            rss: RSSConfig {
                urls: vec![feed.to_string()],
            },
        };

        let client = StubRssClient::default();
        let stub_target = StubTarget::default();
        let target_calls = Arc::clone(&stub_target.calls);
        let targets = vec![stub_target.into()];

        syndicate(&config, Box::new(client), &targets)
            .await
            .expect("Should be Ok()");

        let calls = (*target_calls).lock().await;

        assert_eq!(*calls, vec![default_items(feed)]);
    }

    #[tokio::test]
    async fn test_syndycate_publishes_from_multiple_feeds_to_multiple_targets() {
        let feed1 = "http://example.com/rss.xml";
        let feed2 = "https://blog.example.com/rss.xml";
        let config = Config {
            rss: RSSConfig {
                urls: vec![feed1.to_string(), feed2.to_string()],
            },
        };

        let client = StubRssClient::default();
        let stub_target1 = StubTarget::default();
        let target_calls1 = Arc::clone(&stub_target1.calls);
        let stub_target2 = StubTarget::default();
        let target_calls2 = Arc::clone(&stub_target2.calls);

        let targets = vec![stub_target1.into(), stub_target2.into()];

        syndicate(&config, Box::new(client), &targets)
            .await
            .expect("Should be Ok()");

        let calls1 = (*target_calls1).lock().await;
        let calls2 = (*target_calls2).lock().await;

        assert_eq!(*calls1, vec![default_items(feed1), default_items(feed2)]);
        assert_eq!(*calls2, vec![default_items(feed1), default_items(feed2)]);
    }

    // TOOD: test the following scenarions
    // - a failure in fetching a feed shouldn't stop syndycating the rest of the feeds
    // - a failure in publishing to a target shouldn stop syndycating
}
