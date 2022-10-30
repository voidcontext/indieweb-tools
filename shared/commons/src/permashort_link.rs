struct PermashortCitation {
    domain: String,
    short_url: String,
}

impl PermashortCitation {
    pub fn to_uri(&self, protocol: &str) -> String {
        format!("{}://{}/{}", protocol, self.domain, self.short_url)
    }
}

impl ToString for PermashortCitation {
    fn to_string(&self) -> String {
        format!("{} {}", self.domain, self.short_url)
    }
}

#[cfg(test)]
mod test {
    use super::PermashortCitation;

    #[test]
    fn test_to_string() {
        let psc = PermashortCitation {
            domain: String::from("vdx.hu"),
            short_url: String::from("s/Df3l"),
        };

        assert_eq!(psc.to_string().as_str(), "vdx.hu s/Df3l")
    }

    #[test]
    fn test_to_uri() {
        let psc = PermashortCitation {
            domain: String::from("vdx.hu"),
            short_url: String::from("s/Df3l"),
        };

        assert_eq!(psc.to_uri("https").as_str(), "https://vdx.hu/s/Df3l")
    }
}
