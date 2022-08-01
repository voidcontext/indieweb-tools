use std::fs;

use oauth2::ClientId;
use serde_derive::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
pub struct Config {
    pub rss: RSSConfig,
    pub db: DBConfig,
    pub twitter: TwitterConfig,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct RSSConfig {
    pub urls: Vec<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct DBConfig {
    pub path: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct TwitterConfig {
    pub client_id: ClientId,
}

impl Config {
    pub fn from_file(file_name: &str) -> Result<Config, toml::de::Error> {
        let config_str = fs::read_to_string(file_name).unwrap();

        toml::from_str(&config_str)
    }
}

#[cfg(test)]
mod test {
    use oauth2::ClientId;

    use super::Config;
    use super::DBConfig;
    use super::RSSConfig;
    use super::TwitterConfig;

    #[test]
    fn config_model_should_be_deserializable() {
        let config = r#"
        [rss]
        urls = [
          "http://exmample.com/rss.xml",
          "http://exmample.com/some-site/rss.xml"
        ]
        [db]
        path = "some/path"
        [twitter]
        client_id = "some_client_id"
        "#;

        assert_eq!(
            toml::from_str::<Config>(config),
            Ok(Config {
                rss: RSSConfig {
                    urls: vec![
                        "http://exmample.com/rss.xml".to_string(),
                        "http://exmample.com/some-site/rss.xml".to_string()
                    ]
                },
                db: DBConfig {
                    path: String::from("some/path")
                },
                twitter: TwitterConfig {
                    client_id: ClientId::new(String::from("some_client_id"))
                }
            })
        )
    }
}
