use std::fs;

use serde_derive::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
pub struct Config {
    pub rss: RSSConfig,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct RSSConfig {
    pub urls: Vec<String>,
}

impl Config {
    pub fn from_file(file_name: &str) -> Result<Config, toml::de::Error> {
        let config_str = fs::read_to_string(file_name).unwrap();

        toml::from_str(&config_str)
    }
}

#[cfg(test)]
mod test {
    use super::Config;
    use super::RSSConfig;

    #[test]
    fn config_model_should_be_deserializable() {
        let config = r#"
        [rss]
        urls = [
          "http://exmample.com/rss.xml",
          "http://exmample.com/some-site/rss.xml"
        ]
        "#;

        assert_eq!(
            toml::from_str::<Config>(config),
            Ok(Config {
                rss: RSSConfig {
                    urls: vec![
                        "http://exmample.com/rss.xml".to_string(),
                        "http://exmample.com/some-site/rss.xml".to_string()
                    ]
                }
            })
        )
    }
}
