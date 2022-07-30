use crate::target::Target;
use async_trait::async_trait;
use rss::Item;

pub struct Mastodon {}

#[async_trait]
impl Target for Mastodon {
    async fn publish<'a>(&self, posts: &[Item]) -> Result<(), Box<dyn std::error::Error + 'a>> {
        todo!()
    }
}
