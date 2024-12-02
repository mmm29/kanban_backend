use sqlx::Row;

use crate::{
    app::repositories::SessionsRepository,
    model::{SessionToken, UserId},
};

use super::DatabaseConnectionRef;

pub struct DbSessions {
    db: DatabaseConnectionRef,
}

impl DbSessions {
    pub fn new(db: DatabaseConnectionRef) -> Self {
        Self { db }
    }
}

#[async_trait]
impl SessionsRepository for DbSessions {
    async fn get_authorized_user_id(&self, token: &SessionToken) -> anyhow::Result<Option<UserId>> {
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

    async fn create_user_session(&self, user_id: UserId) -> anyhow::Result<SessionToken> {
        let token = SessionToken::generate_random();

        sqlx::query("INSERT INTO sessions (user_id, token) VALUES ($1, $2)")
            .bind(user_id.raw() as i32)
            .bind(token.as_str())
            .execute(self.db.as_pool())
            .await?;

        Ok(token)
    }
}
