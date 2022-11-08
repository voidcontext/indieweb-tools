use std::rc::Rc;

pub use crate::target::Target;
use crate::{
    auth::token_db::SqliteTokenDB, mastodon::Mastodon,
    syndicated_post::SqliteSyndycatedPostStorage, twitter::Twitter,
};
use iwt_commons::url_shortener::ReqwestClient;
pub use iwt_commons::*;
use iwt_config::Config;
use rusqlite::Connection;

mod mastodon;
mod rss;
mod syndicate;
mod syndicated_post;
mod target;
mod twitter;

pub async fn execute(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let conn = Rc::new(Connection::open(&config.db.path).expect("Couldn't open DB"));

    let token_db = Rc::new(SqliteTokenDB::new(Rc::clone(&conn)));

    let url_shortener_client = Rc::new(ReqwestClient::new(
        &config.url_shortener.protocol,
        &config.url_shortener.domain,
        config.url_shortener.put_base_uri.as_ref(),
    ));

    let targets: Vec<Box<dyn Target>> = vec![
        Box::new(Twitter::new(
            config.twitter.client_id.clone(),
            token_db,
            Rc::clone(&url_shortener_client),
        )),
        Box::new(Mastodon::new(
            config.mastodon.base_uri.clone(),
            config.mastodon.access_token.clone(),
            Rc::clone(&url_shortener_client),
        )),
    ];

    let storage = SqliteSyndycatedPostStorage::new(Rc::clone(&conn));
    storage
        .init_table()
        .expect("Couldn't initialise post storage");

    syndicate::syndicate(config, &rss::ReqwestClient, &targets, &storage).await
}

#[cfg(test)]
pub mod stubs {
    pub use crate::rss::stubs as rss;
    pub use crate::syndicated_post::stubs as syndycated_post;
    pub use crate::target::stubs as target;
}
