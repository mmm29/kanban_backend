use std::sync::Arc;

use crate::model::{
    tasks::{TaskCategoryDescription, TaskDescription},
    TaskId, UserId,
};

use super::repositories::TasksRepository;

pub struct TasksService {
    tasks: Arc<dyn TasksRepository>,
}

impl TasksService {
    pub fn new(tasks: Arc<dyn TasksRepository>) -> Self {
        Self { tasks }
    }

    pub async fn fetch_tasks(&self, user_id: UserId) -> anyhow::Result<Vec<TaskDescription>> {
        self.tasks.fetch_tasks(user_id).await
    }

    pub async fn create_task(
        &self,
        user_id: UserId,
        label: &str,
        description: &str,
        category_id: &str,
    ) -> anyhow::Result<TaskId> {
        self.tasks
            .create_task(user_id, label, description, category_id)
            .await
    }

    pub async fn modify_task(
        &self,
        user_id: UserId,
        task_id: &str,
        label: &str,
        description: &str,
        category_id: &str,
    ) -> anyhow::Result<()> {
        self.tasks
            .modify_task(user_id, task_id, label, description, category_id)
            .await
    }

    pub async fn delete_task(&self, user_id: UserId, task_id: &str) -> anyhow::Result<()> {
        self.tasks.delete_task(user_id, task_id).await
    }

    pub async fn fetch_categories(
        &self,
        user_id: UserId,
    ) -> anyhow::Result<Vec<TaskCategoryDescription>> {
        self.tasks.fetch_categories(user_id).await
    }
}
