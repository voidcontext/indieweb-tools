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

        let channel = Channel::read_from(&feed[..])?;
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
    use std::sync::Arc;

    use async_mutex::Mutex;
    use async_trait::async_trait;
    use rss::{Channel, Item};

    use super::RssClient;

    #[derive(Default)]
    pub struct StubRssClient {
        pub urls: Arc<Mutex<Vec<String>>>,
    }

    pub fn default_items(url: &str) -> Vec<Item> {
        (1..5)
            .map(|i| Item {
                title: Some(format!("This is pos #{} at {}", i, url)),
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

            let channel = Channel {
                items: default_items(url),
                link: url.to_owned(),
                ..Default::default()
            };

            Ok(channel)
        }
    }
}
