use crate::{app::repositories::UsersRepositry, model::UserId};

use super::DatabaseConnectionRef;
use sqlx::Row;

pub struct DbUsers {
    db: DatabaseConnectionRef,
}

impl DbUsers {
    pub fn new(db: DatabaseConnectionRef) -> Self {
        Self { db }
    }
}

#[async_trait]
impl UsersRepositry for DbUsers {
    async fn get_username(&self, user_id: UserId) -> anyhow::Result<Option<String>> {
        let row = sqlx::query("SELECT username FROM users WHERE user_id=$1")
            .bind(user_id.raw() as i32)
            .fetch_one(self.db.as_pool())
            .await?;

        Ok(row.try_get(0).ok())
    }

    async fn does_user_exist_by_username(&self, username: &str) -> anyhow::Result<bool> {
        let row = sqlx::query("SELECT 1 FROM users WHERE username=$1")
            .bind(username)
            .fetch_optional(self.db.as_pool())
            .await?;

        Ok(row.is_some())
    }

    async fn create_user(&self, username: &str, password: &str) -> anyhow::Result<UserId> {
        let row =
            sqlx::query("INSERT INTO users (username, password) VALUES ($1, $2) RETURNING user_id")
                .bind(username)
                .bind(password)
                .fetch_one(self.db.as_pool())
                .await?;

        let raw_user_id: i32 = row.try_get(0)?;
        Ok(UserId::from_raw(raw_user_id as i64))
    }

    async fn find_user_with_password(
        &self,
        username: &str,
    ) -> anyhow::Result<Option<(UserId, String)>> {
        let optional_row = sqlx::query("SELECT user_id, password FROM users WHERE username=$1")
            .bind(username)
            .fetch_optional(self.db.as_pool())
            .await?;

        let Some(row) = optional_row else {
            return Ok(None);
        };

        let raw_user_id: i32 = row.try_get(0)?;
        let password = row.try_get(1)?;
        let user_id = UserId::from_raw(raw_user_id as i64);

        Ok(Some((user_id, password)))
    }
}
