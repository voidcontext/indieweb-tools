use async_trait::async_trait;
use rss::Channel;

pub struct RssClientImpl;

#[async_trait]
pub trait RssClient {
    /// Loads RSS feed from the given URL a parse it into a Channel
    async fn get_channel(&self, url: &str)
        -> Result<Channel, Box<dyn std::error::Error + 'static>>;
}

#[async_trait]
impl RssClient for RssClientImpl {
    async fn get_channel(
        &self,
        url: &str,
    ) -> Result<Channel, Box<dyn std::error::Error + 'static>> {
        let feed = reqwest::get(url).await?.bytes().await?;

        log::debug!("Response received from url: {}", url);

        let mut channel = Channel::read_from(&feed[..])?;
        
        for item in channel.items_mut() {
            if let Some(description) = item.description.clone() {
                let str = description.clone();
                let decoded = htmlentity::entity::decode(&str.as_str()).iter().collect::<String>();
                println!("{}", decoded);
                item.set_description(decoded);
            }
        }

        log::debug!(
            "Successfully loaded channel \"{}\", with {} items",
            channel.title(),
            channel.items().len()
        );
        Ok(channel)
    }
}

#[cfg(test)]
pub mod stubs {
    use std::{fmt::Display, ops::Deref, sync::Arc};

    use async_mutex::Mutex;
    use async_trait::async_trait;
    use reqwest::Url;
    use rss::{Channel, GuidBuilder, Item};

    use super::RssClient;

    #[derive(Default)]
    pub struct StubRssClient {
        pub urls: Arc<Mutex<Vec<String>>>,
    }

    pub fn default_items(url: &str) -> Vec<Item> {
        (0..4)
            .map(|i| Item {
                title: Some(format!("This is pos #{} at {}", i, url)),
                link: Some(format!("{}/post-{}", url, i)),
                guid: Some(
                    GuidBuilder::default()
                        .value(format!("{}/post-{}", url, i))
                        .build(),
                ),
                ..Default::default()
            })
            .collect()
    }

    #[async_trait]
    impl RssClient for StubRssClient {
        async fn get_channel(
            &self,
            url: &str,
        ) -> Result<Channel, Box<dyn std::error::Error + 'static>> {
            let mut urls = self.urls.lock().await;
            urls.push(url.to_owned());

            match Url::parse(url) {
                Ok(parsed) => {
                    let should_fail = parsed
                        .query_pairs()
                        .any(|(key, value)| key.deref() == "failure" && value.deref() == "1");

                    if should_fail {
                        Err(Box::new(RssClientError))
                    } else {
                        let channel = Channel {
                            items: default_items(url),
                            link: url.to_owned(),
                            ..Default::default()
                        };

                        Ok(channel)
                    }
                }
                _ => panic!("Invalid url: {}", url),
            }
        }
    }

    #[derive(Debug)]
    pub struct RssClientError;
    impl Display for RssClientError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "RssClientError")
        }
    }
    impl std::error::Error for RssClientError {}
}
