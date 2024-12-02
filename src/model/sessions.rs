use rand::Rng;
use sqlx::Row;

use crate::model::{database::DbError, UserId};

use super::DatabaseConnectionRef;

// Represents a valid session token.
#[derive(Debug)]
pub struct SessionToken(String);

impl SessionToken {
    pub fn from_str(token: &str) -> Option<SessionToken> {
        if !Self::is_valid_token(token) {
            return None;
        }

        Some(Self(token.to_string()))
    }

    pub fn generate_random() -> SessionToken {
        let mut rng = rand::thread_rng();

        let mut bytes: [u8; 16] = [0; 16];
        bytes.iter_mut().for_each(|b| *b = rng.gen());

        let token = hex::encode(bytes);
        debug_assert!(Self::is_valid_token(&token));

        Self(token)
    }

    fn is_valid_token(token: &str) -> bool {
        // All tokens are of length 32.
        token.len() == 32
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub struct SessionModel {
    db: DatabaseConnectionRef,
}

impl SessionModel {
    pub fn new(db: DatabaseConnectionRef) -> Self {
        Self { db }
    }

    pub async fn get_authorized_user_id(
        &self,
        token: &SessionToken,
    ) -> Result<Option<UserId>, DbError> {
        let optional_row = sqlx::query("SELECT user_id FROM sessions WHERE token = $1")
            .bind(token.as_str())
            .fetch_optional(self.db.as_pool())
            .await?;

        let Some(row) = optional_row else {
            return Ok(None);
        };

        let raw_user_id: i32 = row.try_get(0)?;
        Ok(Some(UserId::from_raw(raw_user_id as i64)))
    }

    pub async fn create_user_session(&self, user_id: UserId) -> Result<SessionToken, DbError> {
        let token = SessionToken::generate_random();

        sqlx::query("INSERT INTO sessions (user_id, token) VALUES ($1, $2)")
            .bind(user_id.raw() as i32)
            .bind(token.as_str())
            .execute(self.db.as_pool())
            .await?;

        Ok(token)
    }
}
