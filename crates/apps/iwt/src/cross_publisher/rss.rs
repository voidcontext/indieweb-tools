use async_trait::async_trait;
use regex::Regex;
use rss::Channel;
use scraper::{Html, Selector};

pub struct ReqwestClient;

#[async_trait]
pub trait Client {
    /// Loads RSS feed from the given URL a parse it into a Channel
    async fn get_channel(&self, url: &str)
        -> Result<Channel, Box<dyn std::error::Error + 'static>>;
}

#[async_trait]
impl Client for ReqwestClient {
    async fn get_channel(
        &self,
        url: &str,
    ) -> Result<Channel, Box<dyn std::error::Error + 'static>> {
        let feed = reqwest::get(url).await?.bytes().await?;

        log::debug!("Response received from url: {}", url);

        let mut channel = Channel::read_from(&feed[..])?;

        for item in channel.items_mut() {
            if let Some(description) = item.description.clone() {
                let re = Regex::new(r"<h[1-6] ").unwrap();

                let summary: String = re.split(&description).next().unwrap().to_string();

                let str = summary
                    .replace("<li>", "<li>- ")
                    .replace("<code>", "`")
                    .replace("</code>", "`")
                    .replace("\n", " ")
                    .replace("</p> ", "\n\n");

                log::debug!("original desc:\n{}\n", str);

                let fragment = Html::parse_document(&format!("<html>{}</html>", &str));
                let cleaned = fragment
                    .select(&Selector::parse("html").unwrap())
                    .next()
                    .unwrap()
                    .text()
                    .collect::<Vec<_>>()
                    .join("");

                log::debug!("cleaned desc:\n{}\n", cleaned);

                item.set_description(cleaned);
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
    use std::{collections::HashMap, fmt::Display, ops::Deref, sync::Arc};

    use async_mutex::Mutex;
    use async_trait::async_trait;
    use reqwest::Url;
    use rss::{extension::ExtensionMap, Channel, GuidBuilder, Item};

    use crate::{cross_publisher::rss_item_ext::stubs::create_iwt_extension_map, social};

    use super::Client;

    pub struct StubRssClient {
        pub urls: Arc<Mutex<Vec<String>>>,
        items: HashMap<String, Vec<Item>>,
    }

    impl StubRssClient {
        pub fn new(items: &HashMap<String, Vec<Item>>) -> Self {
            Self {
                items: items.clone(),
                urls: Default::default(),
            }
        }
    }

    pub fn gen_items(urls: &[&str]) -> HashMap<String, Vec<Item>> {
        gen_items_with_extension(
            urls,
            4,
            0,
            create_iwt_extension_map(
                &[social::Network::Mastodon, social::Network::Twitter],
                None,
                &Vec::new(),
            ),
        )
    }

    pub fn gen_items_with_extension(
        urls: &[&str],
        count: usize,
        offset: usize,
        extensions: ExtensionMap,
    ) -> HashMap<String, Vec<Item>> {
        let mut result = HashMap::new();
        for url in urls {
            let mut items = Vec::new();
            for i in (0 + offset)..(count + offset) {
                items.push(Item {
                    title: Some(format!("This is pos #{} at {}", i, url)),
                    link: Some(format!("{}/post-{}", url, i)),
                    guid: Some(
                        GuidBuilder::default()
                            .value(format!("{}/post-{}", url, i))
                            .build(),
                    ),
                    extensions: extensions.clone(),
                    ..Default::default()
                })
            }
            result.insert(url.to_string(), items);
        }
        result
    }

    #[async_trait]
    impl Client for StubRssClient {
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
                            items: self.items.get(&url.to_string()).unwrap().clone(),
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
