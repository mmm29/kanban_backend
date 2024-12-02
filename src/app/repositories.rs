use crate::model::{
    tasks::{TaskCategoryDescription, TaskDescription},
    SessionToken, TaskId, UserId,
};

#[async_trait]
pub trait SessionsRepository: Send + Sync {
    async fn get_authorized_user_id(&self, token: &SessionToken) -> anyhow::Result<Option<UserId>>;

    async fn create_user_session(&self, user_id: UserId) -> anyhow::Result<SessionToken>;
}

#[async_trait]
pub trait UsersRepositry: Send + Sync {
    async fn does_user_exist_by_username(&self, username: &str) -> anyhow::Result<bool>;

    async fn get_username(&self, user_id: UserId) -> anyhow::Result<Option<String>>;

    async fn create_user(&self, username: &str, password: &str) -> anyhow::Result<UserId>;

    async fn find_user_with_password(
        &self,
        username: &str,
    ) -> anyhow::Result<Option<(UserId, String)>>;
}

#[async_trait]
pub trait TasksRepository: Send + Sync {
    async fn fetch_tasks(&self, user_id: UserId) -> anyhow::Result<Vec<TaskDescription>>;

    async fn create_task(
        &self,
        user_id: UserId,
        label: &str,
        description: &str,
        category_id: &str,
    ) -> anyhow::Result<TaskId>;

    async fn modify_task(
        &self,
        user_id: UserId,
        task_id: &str,
        label: &str,
        description: &str,
        category_id: &str,
    ) -> anyhow::Result<()>;

    async fn delete_task(&self, user_id: UserId, task_id: &str) -> anyhow::Result<()>;

    async fn fetch_categories(
        &self,
        user_id: UserId,
    ) -> anyhow::Result<Vec<TaskCategoryDescription>>;

    async fn add_categories(
        &self,
        user_id: UserId,
        labels: &[&str],
    ) -> anyhow::Result<Vec<TaskCategoryDescription>>;
}
