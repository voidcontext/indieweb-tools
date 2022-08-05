use futures::{Future, StreamExt, TryFutureExt};
use rss::Channel;

use crate::target::Target;
use crate::{Config, RssClient};

/// Orchestrates syndication
pub async fn syndicate<'a>(
    config: &Config,
    rss_client: Box<dyn RssClient + 'a>,
    targets: &[Box<dyn Target>],
) -> Result<(), Box<dyn std::error::Error + 'a>> {
    log::debug!("Received config: {:?}", config);
    run_and_collect(config.rss.urls.iter(), |url| {
        rss_client
            .get_channel(url)
            .and_then(|channel| syndycate_channel(channel, targets))
    })
    .await
}

/// Syndicates a single channel
async fn syndycate_channel(
    channel: Channel,
    targets: &[Box<dyn Target>],
) -> Result<(), Box<dyn std::error::Error>> {
    run_and_collect(targets.iter(), |target| {
        run_and_collect(channel.items.iter(), |post| target.publish(post))
    })
    .await
}

async fn run_and_collect<C, I, F, Fu>(items: C, f: F) -> Result<(), Box<dyn std::error::Error>>
where
    C: Iterator<Item = I>,
    // TODO: understand why this didn't work: Fn(I) -> dyn Future<Output = Result<(), Box<dyn std::error::Error>>>
    //       or: Fn(I) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>>>>
    F: Fn(I) -> Fu,
    Fu: Future<Output = Result<(), Box<dyn std::error::Error>>>,
{
    futures::stream::iter(items)
        .map(f)
        .buffer_unordered(10)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect()
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use oauth2::{AccessToken, ClientId};

    use crate::config::{DBConfig, MastodonConfig, TwitterConfig};
    use crate::stubs::rss::{default_items, StubRssClient};
    use crate::stubs::target::StubTarget;
    use crate::target::stubs::FailingStubTarget;
    use crate::{config::RSSConfig, Config};

    use super::syndicate;

    fn config(urls: Vec<String>) -> Config {
        Config {
            rss: RSSConfig { urls },
            db: DBConfig {
                path: String::from("some/path"),
            },
            twitter: TwitterConfig {
                client_id: ClientId::new(String::from("some_client_id")),
            },
            mastodon: MastodonConfig {
                access_token: AccessToken::new(String::from("some-access-token")),
            },
        }
    }

    #[tokio::test]
    async fn test_syndycate_fetches_a_feed() {
        let feed = "http://example.com/rss.xml";
        let config = config(vec![feed.to_string()]);

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
        let config = config(vec![feed1.to_string(), feed2.to_string()]);

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
        let config = config(vec![feed.to_string()]);

        let client = StubRssClient::default();
        let stub_target = StubTarget::default();
        let target_calls = Arc::clone(&stub_target.calls);
        let targets = vec![stub_target.into()];

        syndicate(&config, Box::new(client), &targets)
            .await
            .expect("Should be Ok()");

        let calls = (*target_calls).lock().await;

        assert_eq!(*calls, default_items(feed));
    }

    #[tokio::test]
    async fn test_syndycate_publishes_from_multiple_feeds_to_multiple_targets() {
        let feed1 = "http://example.com/rss.xml";
        let feed2 = "https://blog.example.com/rss.xml";
        let config = config(vec![feed1.to_string(), feed2.to_string()]);

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

        let mut expected = default_items(feed1);
        expected.extend(default_items(feed2));

        assert_eq!(*calls1, expected);
        assert_eq!(*calls2, expected);
    }

    #[tokio::test]
    async fn test_syndycate_publishes_when_single_feed_fails() {
        let feed1 = "http://example.com/rss.xml?failure=1";
        let feed2 = "https://blog.example.com/rss.xml";
        let config = config(vec![feed1.to_string(), feed2.to_string()]);

        let client = StubRssClient::default();
        let stub_target1 = StubTarget::default();
        let target_calls1 = Arc::clone(&stub_target1.calls);
        let stub_target2 = StubTarget::default();
        let target_calls2 = Arc::clone(&stub_target2.calls);

        let targets = vec![stub_target1.into(), stub_target2.into()];

        let result = syndicate(&config, Box::new(client), &targets).await;

        assert!(result.is_err());

        let calls1 = (*target_calls1).lock().await;
        let calls2 = (*target_calls2).lock().await;

        assert_eq!(*calls1, default_items(feed2));
        assert_eq!(*calls2, default_items(feed2));
    }

    #[tokio::test]
    async fn test_syndycate_publishes_when_single_target_fails() {
        let feed1 = "http://example.com/rss.xml";
        let feed2 = "https://blog.example.com/rss.xml";
        let config = config(vec![feed1.to_string(), feed2.to_string()]);

        let client = StubRssClient::default();
        let stub_target1 = FailingStubTarget::default();
        let stub_target2 = StubTarget::default();
        let target_calls2 = Arc::clone(&stub_target2.calls);

        let targets = vec![stub_target1.into(), stub_target2.into()];

        let result = syndicate(&config, Box::new(client), &targets).await;

        assert!(result.is_err());

        let calls2 = (*target_calls2).lock().await;
        let mut expected = default_items(feed1);
        expected.extend(default_items(feed2));

        assert_eq!(*calls2, expected);
    }
}
