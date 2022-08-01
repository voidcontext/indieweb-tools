use oauth2::{AccessToken, RefreshToken};

pub trait TokenDB {
    fn get_access_token(&self, provider: &str) -> Result<AccessToken, Box<dyn std::error::Error>>;
    fn get_refresh_token(&self, provider: &str)
        -> Result<RefreshToken, Box<dyn std::error::Error>>;
    fn store(
        &self,
        provider: &str,
        access_token: &AccessToken,
        refresh_token: &RefreshToken,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct SledTokenDB {
    db: sled::Db,
}

impl SledTokenDB {
    pub fn new(path: &str) -> Self {
        let db = sled::open(path).expect("Cannot open sled db");
        Self { db }
    }
}

impl TokenDB for SledTokenDB {
    fn get_access_token(&self, provider: &str) -> Result<AccessToken, Box<dyn std::error::Error>> {
        self.db
            .get(bincode::serialize(&format!("auth:{}:access_token", provider)).unwrap())
            .map(|value| bincode::deserialize::<AccessToken>(&value.unwrap()).unwrap())
            .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)
    }

    fn get_refresh_token(
        &self,
        provider: &str,
    ) -> Result<RefreshToken, Box<dyn std::error::Error>> {
        self.db
            .get(bincode::serialize(&format!("auth:{}:refresh_token", provider)).unwrap())
            .map(|value| bincode::deserialize::<RefreshToken>(&value.unwrap()).unwrap())
            .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)
    }

    fn store(
        &self,
        provider: &str,
        access_token: &AccessToken,
        refresh_token: &RefreshToken,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.db
            .insert(
                bincode::serialize(&format!("auth:{}:access_token", provider)).unwrap(),
                bincode::serialize(&access_token).unwrap(),
            )
            .and_then(|_| {
                self.db.insert(
                    bincode::serialize(&format!("auth:{}:refresh_token", provider)).unwrap(),
                    bincode::serialize(&refresh_token).unwrap(),
                )
            })
            .map(|_| ())
            .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)
    }
}

#[cfg(test)]
pub mod stubs {
    use std::sync::Mutex;

    use oauth2::{AccessToken, RefreshToken};

    use super::TokenDB;

    pub struct StubTokenDB {
        access_token: Mutex<AccessToken>,
        refresh_token: Mutex<RefreshToken>,
    }

    impl StubTokenDB {
        pub fn new() -> Self {
            StubTokenDB {
                access_token: Mutex::new(AccessToken::new(String::from("initial-access-token"))),
                refresh_token: Mutex::new(RefreshToken::new(String::from("initial-refresh-token"))),
            }
        }
    }

    impl TokenDB for StubTokenDB {
        fn get_access_token(
            &self,
            _provider: &str,
        ) -> Result<oauth2::AccessToken, Box<dyn std::error::Error>> {
            let guard = self.access_token.lock().unwrap();
            Ok((*guard).clone())
        }

        fn get_refresh_token(
            &self,
            _provider: &str,
        ) -> Result<oauth2::RefreshToken, Box<dyn std::error::Error>> {
            let guard = self.refresh_token.lock().unwrap();
            Ok((*guard).clone())
        }

        fn store(
            &self,
            _provider: &str,
            access_token: &AccessToken,
            refresh_tokem: &RefreshToken,
        ) -> Result<(), Box<dyn std::error::Error>> {
            let mut guard = self.access_token.lock().unwrap();
            *guard = access_token.clone();

            let mut guard = self.refresh_token.lock().unwrap();
            *guard = refresh_tokem.clone();

            Ok(())
        }
    }
}
