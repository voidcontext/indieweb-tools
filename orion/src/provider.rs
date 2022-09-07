use std::fmt::Display;

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub enum Provider {
    Twitter,
    Mastodon,
}

impl Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Provider::Twitter => write!(f, "twitter"),
            Provider::Mastodon => write!(f, "mastodon"),
        }
    }
}
