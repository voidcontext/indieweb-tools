use std::rc::Rc;

use rss::Item;
use rusqlite::Connection;

use crate::social::Network;

#[derive(Debug, PartialEq)]
pub struct SyndicatedPost {
    pub social_network: Network,
    pub id: String,
    pub original_guid: String,
    pub original_uri: String,
}

impl SyndicatedPost {
    pub fn new(social_network: Network, id: &String, item: &Item) -> Self {
        Self {
            social_network,
            id: String::from(id),
            original_guid: String::from(item.guid().unwrap().value()),
            original_uri: String::from(item.link().unwrap()),
        }
    }
}

#[derive(Debug)]
pub enum StorageError {
    PersistenceError(String),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StorageError")
    }
}

impl std::error::Error for StorageError {}

pub trait SyndicatedPostStorage {
    fn store(&self, syndicated_post: SyndicatedPost) -> Result<(), StorageError>;
}

pub struct SqliteSyndycatedPostStorage {
    conn: Rc<Connection>,
}

impl SqliteSyndycatedPostStorage {
    pub fn new(conn: Rc<Connection>) -> Self {
        Self { conn }
    }

    pub fn init_table(&self) -> Result<(), StorageError> {
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS post (
              id VARCHAR(64) NOT NULL,
              social_network VARCHAR(20) NOT NULL,
              original_guid TEXT NOT NULL,
              original_uri TEXT NOT NULL,
            
              PRIMARY KEY (id, social_network)
            )",
                (),
            )
            .map(|_| ())
            .map_err(|err| StorageError::PersistenceError(format!("{:?}", err)))
    }
}

impl SyndicatedPostStorage for SqliteSyndycatedPostStorage {
    fn store(&self, syndicated_post: SyndicatedPost) -> Result<(), StorageError> {
        self.conn
            .execute(
                "INSERT INTO post (id, social_network, original_guid, original_uri) 
             VALUES (:id, :social_network, :original_guid, :original_url)",
                &[
                    (":id", &syndicated_post.id),
                    (
                        ":social_network",
                        &syndicated_post.social_network.to_string(),
                    ),
                    (":original_guid", &syndicated_post.original_guid),
                    (":original_url", &syndicated_post.original_uri),
                ],
            )
            .map(|_| ())
            .map_err(|err| StorageError::PersistenceError(format!("{:?}", err)))
    }
}

#[cfg(test)]
pub mod stubs {
    use std::sync::Mutex;

    use super::{SyndicatedPost, SyndicatedPostStorage};

    pub struct SyndicatedPostStorageStub {
        pub posts: Mutex<Vec<SyndicatedPost>>,
    }

    impl Default for SyndicatedPostStorageStub {
        fn default() -> Self {
            Self {
                posts: Default::default(),
            }
        }
    }

    impl SyndicatedPostStorage for SyndicatedPostStorageStub {
        fn store(&self, syndicated_post: SyndicatedPost) -> Result<(), super::StorageError> {
            let mut posts = self.posts.lock().unwrap();
            posts.push(syndicated_post);

            Ok(())
        }
    }
}
