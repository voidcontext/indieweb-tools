use rss::{extension::Extension, Item};

use crate::social;

/// Rust representation of the Indieweb Tools RSS extension
#[derive(Debug, PartialEq)]
pub struct IwtRssExtension {
    /// The target networks where Item should be syndicated to
    pub target_networks: Vec<IwtRssTargetNetwork>,
}

#[derive(Debug, PartialEq)]
pub struct IwtRssTargetNetwork {
    pub network: social::Network,
}

pub trait RssItemExt {
    fn get_iwt_extension(&self) -> Option<IwtRssExtension>;
}

fn get_children<'a>(ext: &'a Extension, key: &str) -> Vec<&'a Extension> {
    ext.children()
        .get(key)
        .iter()
        .flat_map(|children| children.iter())
        .collect::<Vec<_>>()
}

impl RssItemExt for Item {
    fn get_iwt_extension(&self) -> Option<IwtRssExtension> {
        // todo!()
        self.extensions()
            .get(&"iwt".to_string())
            .and_then(|iwt_root| iwt_root.get("extension").map(|extensions| &extensions[0]))
            .map(|iwt_extension| {
                println!("iwt: {:?}", iwt_extension);
                let target_networks = get_children(iwt_extension, "targetNetwork")
                    .iter()
                    .map(|target_network| {
                        let target_network_name = get_children(target_network, "targetNetworkName")
                            [0]
                        .value()
                        .unwrap();

                        match target_network_name {
                            "Twitter" => IwtRssTargetNetwork {
                                network: social::Network::Twitter,
                            },
                            "Mastodon" => IwtRssTargetNetwork {
                                network: social::Network::Mastodon,
                            },
                            _ => panic!("Unknown netowrk: {}", target_network_name),
                        }
                    })
                    .collect::<Vec<_>>();

                IwtRssExtension { target_networks }
            })
    }
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use crate::{
        cross_publisher::rss_item_ext::{IwtRssExtension, IwtRssTargetNetwork},
        social,
    };
    use rss::{
        extension::{Extension, ExtensionBuilder, ExtensionMap},
        Item,
    };

    use super::RssItemExt;

    #[test]
    fn test_get_iwt_extension_should_return_none_when_extension_is_not_available() {
        let item = Item::default();

        let extension = item.get_iwt_extension();

        assert_eq!(extension, None);
    }

    #[test]
    fn test_get_iwt_extension_should_return_the_extension_with_zero_target_networks_if_no_children()
    {
        let iwt_extension = ExtensionBuilder::default().name("iwt:extension").build();

        let mut iwt_root = BTreeMap::new();
        iwt_root.insert("extension".to_string(), vec![iwt_extension]);

        let mut extensions = BTreeMap::new();
        extensions.insert("iwt".to_string(), iwt_root);

        let item = Item {
            extensions,
            ..Default::default()
        };
        let extension = item.get_iwt_extension();

        assert_eq!(
            extension,
            Some(IwtRssExtension {
                target_networks: vec![]
            })
        );
    }

    fn create_extension(name: &str, value: &str) -> Extension {
        ExtensionBuilder::default()
            .name(name)
            .value(Some(value.to_string()))
            .build()
    }

    fn create_extension_with_children(
        name: &str,
        key: &str,
        children: Vec<Extension>,
    ) -> Extension {
        let mut children_map = BTreeMap::new();
        children_map.insert(key.to_string(), children);
        ExtensionBuilder::default()
            .name(name)
            .children(children_map)
            .build()
    }

    fn create_iwt_extension_map(iwt_extension: Extension) -> ExtensionMap {
        let mut iwt_root = BTreeMap::new();
        iwt_root.insert("extension".to_string(), vec![iwt_extension]);

        let mut extensions = BTreeMap::new();
        extensions.insert("iwt".to_string(), iwt_root);

        extensions
    }

    #[test]
    fn test_get_iwt_extension_should_return_the_extension_with_target_networks() {
        let mastodon_target_network_name = create_extension("iwt:targetNetworkName", "Mastodon");
        let twitter_target_network_name = create_extension("iwt:targetNetworkName", "Twitter");

        let iwt_extension = create_extension_with_children(
            "iwt:extension",
            "targetNetwork",
            vec![
                create_extension_with_children(
                    "iwt:targetNetwork",
                    "targetNetworkName",
                    vec![mastodon_target_network_name],
                ),
                create_extension_with_children(
                    "iwt:targetNetwork",
                    "targetNetworkName",
                    vec![twitter_target_network_name],
                ),
            ],
        );

        let item = Item {
            extensions: create_iwt_extension_map(iwt_extension),
            ..Default::default()
        };
        let extension = item.get_iwt_extension();

        assert_eq!(
            extension,
            Some(IwtRssExtension {
                target_networks: vec![
                    IwtRssTargetNetwork {
                        network: social::Network::Mastodon
                    },
                    IwtRssTargetNetwork {
                        network: social::Network::Twitter
                    },
                ]
            })
        );
    }
}
