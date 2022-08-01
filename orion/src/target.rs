use async_trait::async_trait;
use rss::Item;

#[async_trait(?Send)]
pub trait Target {
    async fn publish<'a>(&self, posts: &[Item]) -> Result<(), Box<dyn std::error::Error + 'a>>;
}

#[cfg(test)]
pub mod stubs {
    use async_mutex::Mutex;
    use std::{fmt::Display, sync::Arc};

    use async_trait::async_trait;
    use rss::Item;

    use super::Target;

    #[derive(Default)]
    pub struct StubTarget {
        pub calls: Arc<Mutex<Vec<Vec<Item>>>>,
    }

    #[async_trait(?Send)]
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

    #[derive(Debug)]
    pub struct TargetError;
    impl Display for TargetError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "RssClientError")
        }
    }

    impl std::error::Error for TargetError {}

    #[derive(Default)]
    pub struct FailingStubTarget;

    #[async_trait(?Send)]
    impl Target for FailingStubTarget {
        async fn publish<'a>(
            &self,
            _posts: &[Item],
        ) -> Result<(), Box<dyn std::error::Error + 'a>> {
            Err(Box::new(TargetError))
        }
    }

    impl From<FailingStubTarget> for Box<dyn Target> {
        fn from(stub_target: FailingStubTarget) -> Self {
            Box::new(stub_target)
        }
    }
}
