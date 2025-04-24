use crate::{
    app::repositories::TasksRepository,
    model::{
        tasks::{generate_random_task_id, TaskCategoryDescription, TaskDescription},
        TaskId, UserId,
    },
};

use super::{DatabaseConnectionRef, DbError};

use sqlx::Row;

pub struct DbTasks {
    db: DatabaseConnectionRef,
}

impl DbTasks {
    pub fn new(db: DatabaseConnectionRef) -> Self {
        Self { db }
    }
}

#[async_trait]
impl TasksRepository for DbTasks {
    async fn fetch_tasks(&self, user_id: UserId) -> anyhow::Result<Vec<TaskDescription>> {
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

    async fn create_task(
        &self,
        user_id: UserId,
        label: &str,
        description: &str,
        category_id: &str,
    ) -> anyhow::Result<TaskId> {
        let random_task_id = generate_random_task_id();

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

    async fn modify_task(
        &self,
        user_id: UserId,
        task_id: &str,
        label: &str,
        description: &str,
        category_id: &str,
    ) -> anyhow::Result<()> {
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
            return Err(DbError::RowNotFound.into());
        }

        Ok(())
    }

    async fn delete_task(&self, user_id: UserId, task_id: &str) -> anyhow::Result<()> {
        let res = sqlx::query("DELETE FROM tasks WHERE user_id=$1 AND task_id=$2")
            .bind(user_id.raw())
            .bind(task_id)
            .execute(self.db.as_pool())
            .await?;

        // Expected to delete at least 1 task.
        if res.rows_affected() == 0 {
            return Err(DbError::RowNotFound.into());
        }

        Ok(())
    }

    async fn fetch_categories(
        &self,
        user_id: UserId,
    ) -> anyhow::Result<Vec<TaskCategoryDescription>> {
        let rows = sqlx::query("SELECT category_id, label FROM task_categories WHERE user_id=$1")
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

    async fn add_categories(
        &self,
        user_id: UserId,
        labels: &[&str],
    ) -> anyhow::Result<Vec<TaskCategoryDescription>> {
        let descriptions: Vec<TaskCategoryDescription> = labels
            .iter()
            .map(|label| TaskCategoryDescription {
                category_id: generate_random_task_id(),
                label: label.to_string(),
            })
            .collect();

        let mut tx = self.db.as_pool().begin().await?;

        for desc in &descriptions {
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
        Ok(descriptions)
    }
}
