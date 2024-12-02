use super::{DatabaseConnectionRef, DbError, UniqueId};
use sqlx::Row;

pub type UserId = UniqueId;

pub struct UserModel {
    db: DatabaseConnectionRef,
}

impl UserModel {
    pub fn new(db: DatabaseConnectionRef) -> Self {
        Self { db }
    }

    pub async fn get_username(&self, user_id: UserId) -> Result<String, DbError> {
        let row = sqlx::query("SELECT username FROM users WHERE user_id=$1")
            .bind(user_id.raw() as i32)
            .fetch_one(self.db.as_pool())
            .await?;

        Ok(row.try_get(0)?)
    }

    pub async fn does_user_exist_by_username(&self, username: &str) -> Result<bool, DbError> {
        let row = sqlx::query("SELECT 1 FROM users WHERE username=$1")
            .bind(username)
            .fetch_optional(self.db.as_pool())
            .await?;

        Ok(row.is_some())
    }

    pub async fn create_user(&self, username: &str, password: &str) -> Result<UserId, DbError> {
        let row = sqlx::query(
            "INSERT INTO users (username, password) VALUES ($1, $2) RETURNING user_id",
        )
        .bind(username)
        .bind(password)
        .fetch_one(self.db.as_pool())
        .await?;

        let raw_user_id: i32 = row.try_get(0)?;
        Ok(UserId::from_raw(raw_user_id as i64))
    }

    pub async fn find_user_with_password(
        &self,
        username: &str,
    ) -> Result<Option<(UserId, String)>, DbError> {
        let optional_row =
            sqlx::query("SELECT user_id, password FROM users WHERE username=$1")
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
