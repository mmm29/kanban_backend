use super::{DatabaseConnectionRef, DbError, UserId};
use rand::Rng;
use sqlx::Row;

pub type TaskId = String;
pub type TaskCategoryId = String;

pub fn generate_random() -> String {
    let mut rng = rand::thread_rng();

    let mut bytes: [u8; 16] = [0; 16];
    bytes.iter_mut().for_each(|b| *b = rng.gen());

    hex::encode(bytes)
}

pub struct TaskDescription {
    pub task_id: TaskId,
    pub label: String,
    pub description: String,
    pub category_id: TaskCategoryId,
}

pub struct TaskModel {
    db: DatabaseConnectionRef,
}

impl TaskModel {
    pub fn new(db: DatabaseConnectionRef) -> Self {
        Self { db }
    }

    pub async fn fetch_tasks(&self, user_id: UserId) -> Result<Vec<TaskDescription>, DbError> {
        let rows = sqlx::query(
            "SELECT task_id, category_id, label, description FROM tasks WHERE user_id=$1",
        )
        .bind(user_id.raw())
        .fetch_all(self.db.as_pool())
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                // TODO: use try_get instead. Otherwise, panic is possible.
                let task_id = row.get(0);
                let category_id = row.get(1);
                let label = row.get(2);
                let description = row.get(3);

                TaskDescription {
                    task_id,
                    category_id,
                    label,
                    description,
                }
            })
            .collect())
    }

    pub async fn create_task(
        &self,
        user_id: UserId,
        label: &str,
        description: &str,
        category_id: &str,
    ) -> Result<TaskId, DbError> {
        let random_task_id = generate_random();

        sqlx::query("INSERT INTO tasks (user_id, task_id, category_id, label, description) VALUES ($1, $2, $3, $4, $5)")
            .bind(user_id.raw())
            .bind(&random_task_id)
            .bind(category_id)
            .bind(label)
            .bind(description)
            .execute(self.db.as_pool())
            .await?;

        Ok(random_task_id)
    }

    pub async fn modify_task(
        &self,
        user_id: UserId,
        task_id: &str,
        label: &str,
        description: &str,
        category_id: &str,
    ) -> Result<(), DbError> {
        let res = sqlx::query("UPDATE tasks SET label=$1, description=$2, category_id=$3 WHERE user_id=$4 AND task_id=$5")
        .bind(label)
        .bind(description)
            .bind(category_id)
            .bind(user_id.raw())
            .bind(task_id)
            .execute(self.db.as_pool())
            .await?;

        // Expected to modify at least 1 task.
        if res.rows_affected() == 0 {
            return Err(DbError::RowNotFound);
        }

        Ok(())
    }

    pub async fn delete_task(&self, user_id: UserId, task_id: &str) -> Result<(), DbError> {
        let res = sqlx::query("DELETE FROM tasks WHERE user_id=$1 AND task_id=$2")
            .bind(user_id.raw())
            .bind(task_id)
            .execute(self.db.as_pool())
            .await?;

        // Expected to delete at least 1 task.
        if res.rows_affected() == 0 {
            return Err(DbError::RowNotFound);
        }

        Ok(())
    }
}

pub struct TaskCategoryDescription {
    pub category_id: TaskCategoryId,
    pub label: String,
}

pub struct TaskCategoryModel {
    db: DatabaseConnectionRef,
}

impl TaskCategoryModel {
    pub fn new(db: DatabaseConnectionRef) -> Self {
        Self { db }
    }

    pub async fn fetch_categories(
        &self,
        user_id: UserId,
    ) -> Result<Vec<TaskCategoryDescription>, DbError> {
        let rows =
            sqlx::query("SELECT category_id, label FROM task_categories WHERE user_id=$1")
                .bind(user_id.raw())
                .fetch_all(self.db.as_pool())
                .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                // TODO: use try_get instead. Otherwise, panic is possible.
                let category_id = row.get(0);
                let label = row.get(1);

                TaskCategoryDescription { category_id, label }
            })
            .collect())
    }

    pub async fn add_categories(
        &self,
        user_id: UserId,
        labels: &[&str],
    ) -> Result<Vec<TaskCategoryDescription>, DbError> {
        let decsriptions: Vec<TaskCategoryDescription> = labels
            .into_iter()
            .map(|label| TaskCategoryDescription {
                category_id: generate_random(),
                label: label.to_string(),
            })
            .collect();

        let mut tx = self.db.as_pool().begin().await?;

        for desc in &decsriptions {
            sqlx::query(
                "INSERT INTO task_categories (user_id, category_id, label) VALUES ($1, $2, $3)",
            )
            .bind(user_id.raw())
            .bind(&desc.category_id)
            .bind(&desc.label)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(decsriptions)
    }

    // TODO: implement adding, deleting, modifying
}
