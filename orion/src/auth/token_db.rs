use std::rc::Rc;

use oauth2::{AccessToken, RefreshToken};
use rusqlite::Connection;

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

pub struct SqliteTokenDB {
    conn: Rc<Connection>,
}

impl SqliteTokenDB {
    pub fn new(conn: Rc<Connection>) -> Self {
        Self { conn }
    }
}

impl TokenDB for SqliteTokenDB {
    fn get_access_token(&self, provider: &str) -> Result<AccessToken, Box<dyn std::error::Error>> {
        self.conn
            .query_row(
                "SELECT access_token FROM auth_token WHERE provider = :provider",
                &[(":provider", provider)],
                |row| row.get("access_token").map(AccessToken::new),
            )
            .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)
    }

    fn get_refresh_token(
        &self,
        provider: &str,
    ) -> Result<RefreshToken, Box<dyn std::error::Error>> {
        self.conn
            .query_row(
                "SELECT refresh_token FROM auth_token WHERE provider = :provider",
                &[(":provider", provider)],
                |row| row.get("refresh_token").map(RefreshToken::new),
            )
            .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)
    }

    fn store(
        &self,
        provider: &str,
        access_token: &AccessToken,
        refresh_token: &RefreshToken,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.conn.execute(
            "INSERT INTO auth_token (provider, access_token, refresh_token)
             VALUES (?1, ?2, ?3)
             ON CONFLICT (provider) 
                DO UPDATE SET access_token = excluded.access_token, refresh_token = excluded.refresh_token",
            (provider, access_token.secret(), refresh_token.secret())
        )
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
