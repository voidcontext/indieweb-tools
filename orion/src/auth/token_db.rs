use std::rc::Rc;

use oauth2::{AccessToken, RefreshToken};
use rusqlite::Connection;

use crate::social::Network;

pub trait TokenDB {
    fn get_access_token(
        &self,
        social_network: &Network,
    ) -> Result<AccessToken, Box<dyn std::error::Error>>;
    fn get_refresh_token(
        &self,
        social_network: &Network,
    ) -> Result<RefreshToken, Box<dyn std::error::Error>>;
    fn store(
        &self,
        social_network: &Network,
        access_token: &AccessToken,
        refresh_token: &RefreshToken,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct SqliteTokenDB {
    conn: Rc<Connection>,
}

impl SqliteTokenDB {
    pub fn new(conn: Rc<Connection>) -> Self {
        Self { conn }
    }
}

impl TokenDB for SqliteTokenDB {
    fn get_access_token(
        &self,
        social_network: &Network,
    ) -> Result<AccessToken, Box<dyn std::error::Error>> {
        self.conn
            .query_row(
                "SELECT access_token FROM auth_token WHERE social_network = :social_network",
                &[(":social_network", social_network.to_string().as_str())],
                |row| row.get("access_token").map(AccessToken::new),
            )
            .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)
    }

    fn get_refresh_token(
        &self,
        social_network: &Network,
    ) -> Result<RefreshToken, Box<dyn std::error::Error>> {
        self.conn
            .query_row(
                "SELECT refresh_token FROM auth_token WHERE social_network = :social_network",
                &[(":social_network", social_network.to_string().as_str())],
                |row| row.get("refresh_token").map(RefreshToken::new),
            )
            .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)
    }

    fn store(
        &self,
        social_network: &Network,
        access_token: &AccessToken,
        refresh_token: &RefreshToken,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.conn.execute(
            "INSERT INTO auth_token (social_network, access_token, refresh_token)
             VALUES (?1, ?2, ?3)
             ON CONFLICT (social_network) 
                DO UPDATE SET access_token = excluded.access_token, refresh_token = excluded.refresh_token",
            (social_network.to_string().as_str(), access_token.secret(), refresh_token.secret())
        )
            .map(|_| ())
            .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)
    }
}

#[cfg(test)]
pub mod stubs {
    use std::sync::Mutex;

    use oauth2::{AccessToken, RefreshToken};

    use crate::social::Network;

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
            _social_network: &Network,
        ) -> Result<oauth2::AccessToken, Box<dyn std::error::Error>> {
            let guard = self.access_token.lock().unwrap();
            Ok((*guard).clone())
        }

        fn get_refresh_token(
            &self,
            _social_network: &Network,
        ) -> Result<oauth2::RefreshToken, Box<dyn std::error::Error>> {
            let guard = self.refresh_token.lock().unwrap();
            Ok((*guard).clone())
        }

        fn store(
            &self,
            _social_network: &Network,
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
