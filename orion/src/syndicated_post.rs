use rss::Item;

use crate::provider::Provider;

#[derive(Debug, PartialEq)]
pub struct SyndicatedPost {
    pub provider: Provider,
    pub id: String,
    pub original_guid: String,
    pub original_uri: String,
}

impl SyndicatedPost {
    pub fn new(provider: Provider, id: &String, item: &Item) -> Self {
        Self {
            provider,
            id: String::from(id),
            original_guid: String::from(item.guid().unwrap().value()),
            original_uri: String::from(item.link().unwrap()),
        }
    }
}

#[derive(Debug)]
pub enum StorageError {}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StorageError")
    }
}

impl std::error::Error for StorageError {}

pub trait SyndicatedPostStorage {
    fn store(&self, syndicated_post: SyndicatedPost) -> Result<(), StorageError>;
}

pub struct SledSyndycatedPostStorage {}

impl SledSyndycatedPostStorage {
    pub fn new() -> Self {
        Self {}
    }
}

impl SyndicatedPostStorage for SledSyndycatedPostStorage {
    fn store(&self, _syndicated_post: SyndicatedPost) -> Result<(), StorageError> {
        todo!()
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
