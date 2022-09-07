use std::fmt::Display;

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub enum Network {
    Twitter,
    Mastodon,
}

impl Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Network::Twitter => write!(f, "twitter"),
            Network::Mastodon => write!(f, "mastodon"),
        }
    }
}
