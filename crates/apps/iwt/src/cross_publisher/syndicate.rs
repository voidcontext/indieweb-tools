use super::rss;
use ::rss::Channel;
use futures::{Future, FutureExt, StreamExt, TryFutureExt};

use super::syndicated_post;
use super::target::Target;
use crate::Config;

/// Orchestrates syndication
pub async fn syndicate<R, S>(
    config: &Config,
    rss_client: &R,
    targets: &[Box<dyn Target>],
    storage: &S,
) -> Result<(), Box<dyn std::error::Error>>
where
    R: rss::Client,
    S: syndicated_post::Storage,
{
    log::debug!("Received config: {:?}", config);
    run_and_collect(config.rss.urls.iter(), |url| {
        rss_client
            .get_channel(url)
            .and_then(|channel| syndycate_channel(channel, targets, storage))
    })
    .await
}

/// Syndicates a single channel
async fn syndycate_channel<S: syndicated_post::Storage>(
    channel: Channel,
    targets: &[Box<dyn Target>],
    storage: &S,
) -> Result<(), Box<dyn std::error::Error>> {
    run_and_collect(targets.iter(), |target| {
        run_and_collect(channel.items.iter(), |post| {
            log::info!(
                "Syndicating post found at {} to {}",
                post.link().unwrap(),
                target.network().to_string()
            );
            let stored = storage.find(&post.guid.as_ref().unwrap().value, &target.network());

            async {
                match stored {
                    Ok(None) => {
                        log::info!(
                            " -> Post not found in DB, syndycating to {}",
                            target.network().to_string()
                        );
                        target
                            .publish(post)
                            .map(|result| {
                                result.and_then(|syndicated| {
                                    storage
                                        .store(syndicated)
                                        .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)
                                })
                            })
                            .await
                    }
                    Ok(Some(_)) => {
                        log::info!(
                            " -> Post has been already syndicated to {}",
                            target.network().to_string()
                        );
                        Ok(())
                    }
                    Err(e) => Err(Box::new(e) as Box<dyn std::error::Error>),
                }
            }
        })
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

    use super::syndicated_post::{Storage, SyndicatedPost};
    use crate::config::{Config, Mastodon, Rss, Twitter, UrlShortener, DB};
    use crate::cross_publisher::stubs::rss::{default_items, StubRssClient};
    use crate::cross_publisher::stubs::syndycated_post::SyndicatedPostStorageStub;
    use crate::cross_publisher::stubs::target::FailingStubTarget;
    use crate::cross_publisher::stubs::target::StubTarget;
    use crate::social::Network;

    use super::syndicate;

    fn config(urls: Vec<String>) -> Config {
        Config {
            rss: Rss { urls },
            db: DB {
                path: String::from("some/path"),
            },
            twitter: Twitter {
                client_id: ClientId::new(String::from("some_client_id")),
            },
            mastodon: Mastodon {
                base_uri: String::from("https://example.com/mastodon"),
                access_token: AccessToken::new(String::from("some-access-token")),
            },
            url_shortener: UrlShortener {
                protocol: String::from("http"),
                domain: String::from("shortly"),
                put_base_uri: Some(String::from("http://localhost:9000")),
            },
        }
    }

    #[tokio::test]
    async fn test_syndycate_fetches_a_feed() {
        let feed = "http://example.com/rss.xml";
        let config = config(vec![feed.to_string()]);

        let client = StubRssClient::default();
        let client_calls = Arc::clone(&client.urls);
        let stub_target = StubTarget::new(Network::Mastodon);
        let targets = vec![stub_target.into()];

        syndicate(
            &config,
            &client,
            &targets,
            &SyndicatedPostStorageStub::default(),
        )
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
        let stub_target = StubTarget::new(Network::Mastodon);
        let targets = vec![stub_target.into()];

        syndicate(
            &config,
            &client,
            &targets,
            &SyndicatedPostStorageStub::default(),
        )
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
        let stub_target = StubTarget::new(Network::Mastodon);
        let target_calls = Arc::clone(&stub_target.calls);
        let targets = vec![stub_target.into()];

        syndicate(
            &config,
            &client,
            &targets,
            &SyndicatedPostStorageStub::default(),
        )
        .await
        .expect("Should be Ok()");

        let calls = (*target_calls).lock().await;

        assert_eq!(*calls, default_items(feed));
    }

    #[tokio::test]
    async fn test_syndycate_should_skip_published_posts() {
        let feed = "http://example.com/rss.xml";
        let config = config(vec![feed.to_string()]);

        let client = StubRssClient::default();
        let stub_target = StubTarget::new(Network::Mastodon);
        let target_calls = Arc::clone(&stub_target.calls);
        let targets = vec![stub_target.into()];

        let items = default_items(feed);
        let storage = SyndicatedPostStorageStub::default();

        for item in items {
            storage
                .store(SyndicatedPost::new(
                    Network::Mastodon,
                    &String::from("id"),
                    &item,
                ))
                .unwrap();
        }

        syndicate(&config, &client, &targets, &storage)
            .await
            .expect("Should be Ok()");

        let calls = (*target_calls).lock().await;

        assert_eq!(*calls, []);
    }

    #[tokio::test]
    async fn test_syndycate_publishes_from_multiple_feeds_to_multiple_targets() {
        let feed1 = "http://example.com/rss.xml";
        let feed2 = "https://blog.example.com/rss.xml";
        let config = config(vec![feed1.to_string(), feed2.to_string()]);

        let client = StubRssClient::default();
        let stub_target1 = StubTarget::new(Network::Mastodon);
        let target_calls1 = Arc::clone(&stub_target1.calls);
        let stub_target2 = StubTarget::new(Network::Twitter);
        let target_calls2 = Arc::clone(&stub_target2.calls);

        let targets = vec![stub_target1.into(), stub_target2.into()];

        syndicate(
            &config,
            &client,
            &targets,
            &SyndicatedPostStorageStub::default(),
        )
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
        let stub_target1 = StubTarget::new(Network::Mastodon);
        let target_calls1 = Arc::clone(&stub_target1.calls);
        let stub_target2 = StubTarget::new(Network::Twitter);
        let target_calls2 = Arc::clone(&stub_target2.calls);

        let targets = vec![stub_target1.into(), stub_target2.into()];

        let result = syndicate(
            &config,
            &client,
            &targets,
            &SyndicatedPostStorageStub::default(),
        )
        .await;

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
        let stub_target2 = StubTarget::new(Network::Mastodon);
        let target_calls2 = Arc::clone(&stub_target2.calls);

        let targets = vec![stub_target1.into(), stub_target2.into()];

        let result = syndicate(
            &config,
            &client,
            &targets,
            &SyndicatedPostStorageStub::default(),
        )
        .await;

        assert!(result.is_err());

        let calls2 = (*target_calls2).lock().await;
        let mut expected = default_items(feed1);
        expected.extend(default_items(feed2));

        assert_eq!(*calls2, expected);
    }

    #[tokio::test]
    async fn test_syndycate_should_store_the_syndicated_posts() {
        let feed1 = "http://example.com/rss.xml";
        let feed2 = "https://blog.example.com/rss.xml";
        let config = config(vec![feed1.to_string(), feed2.to_string()]);

        let client = StubRssClient::default();
        let stub_target1 = StubTarget::new(Network::Mastodon);
        let stub_target2 = StubTarget::new(Network::Twitter);

        let targets = vec![stub_target1.into(), stub_target2.into()];
        let storage = SyndicatedPostStorageStub::default();

        syndicate(&config, &client, &targets, &storage)
            .await
            .expect("Should be Ok()");

        let mut items = default_items(feed1);
        items.extend(default_items(feed2));

        let mut expected = items
            .iter()
            .enumerate()
            .map(|(i, item)| SyndicatedPost {
                social_network: Network::Mastodon,
                id: i.to_string(),
                original_guid: String::from(item.guid().unwrap().value()),
                original_uri: String::from(item.link().unwrap()),
            })
            .collect::<Vec<_>>();

        expected.extend(
            items
                .iter()
                .enumerate()
                .map(|(i, item)| SyndicatedPost {
                    social_network: Network::Twitter,
                    id: i.to_string(),
                    original_guid: String::from(item.guid().unwrap().value()),
                    original_uri: String::from(item.link().unwrap()),
                })
                .collect::<Vec<_>>(),
        );

        let mut posts = storage.posts.lock().unwrap();
        // Sort vecs as the order doesn't matter
        // TODO: maybe use HashSet?
        expected.sort_by_key(|i| {
            (
                i.social_network.clone(),
                i.id.clone(),
                i.original_uri.clone(),
            )
        });
        posts.sort_by_key(|i| {
            (
                i.social_network.clone(),
                i.id.clone(),
                i.original_uri.clone(),
            )
        });

        assert_eq!(posts.len(), expected.len());
        assert_eq!(*posts, expected);
    }
}
