use std::sync::Mutex;

use crate::{
    app::repositories::TasksRepository,
    model::{
        tasks::{self, TaskCategoryDescription, TaskDescription},
        TaskId, UserId,
    },
};

struct TaskCategoryStorage {
    user_id: UserId,
    category_desc: TaskCategoryDescription,
}

struct TaskStorage {
    user_id: UserId,
    task_desc: TaskDescription,
}

pub struct InMemoryTasks {
    // TODO: use more efficient data structure
    categories: Mutex<Vec<TaskCategoryStorage>>,
    tasks: Mutex<Vec<TaskStorage>>,
}

impl InMemoryTasks {
    pub fn new() -> Self {
        Self {
            categories: Mutex::new(Vec::new()),
            tasks: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl TasksRepository for InMemoryTasks {
    async fn fetch_tasks(&self, user_id: UserId) -> anyhow::Result<Vec<TaskDescription>> {
        let tasks = self.tasks.lock().unwrap();

        Ok(tasks
            .iter()
            .filter(|s| s.user_id == user_id)
            .map(|x| x.task_desc.clone())
            .collect())
    }

    async fn create_task(
        &self,
        user_id: UserId,
        label: &str,
        description: &str,
        category_id: &str,
    ) -> anyhow::Result<TaskId> {
        let task_id = tasks::generate_random_task_id();

        let mut tasks = self.tasks.lock().unwrap();

        // Ensure no task with this ID exists.
        if tasks
            .iter()
            .find(|t| t.user_id == user_id && t.task_desc.task_id == task_id)
            .is_some()
        {
            return Err(anyhow::anyhow!("could not generate unique task id"));
        }

        tasks.push(TaskStorage {
            user_id,
            task_desc: TaskDescription {
                task_id: task_id.clone(),
                label: label.to_string(),
                description: description.to_string(),
                category_id: category_id.to_string(),
            },
        });

        Ok(task_id)
    }

    async fn modify_task(
        &self,
        user_id: UserId,
        task_id: &str,
        label: &str,
        description: &str,
        category_id: &str,
    ) -> anyhow::Result<()> {
        let mut tasks = self.tasks.lock().unwrap();

        let Some(task) = tasks
            .iter_mut()
            .find(|t| t.user_id == user_id && t.task_desc.task_id == task_id)
        else {
            return Err(anyhow::anyhow!("no such task"));
        };

        task.task_desc.label = label.to_string();
        task.task_desc.description = description.to_string();
        task.task_desc.category_id = category_id.to_string();
        Ok(())
    }

    async fn delete_task(&self, user_id: UserId, task_id: &str) -> anyhow::Result<()> {
        let mut tasks = self.tasks.lock().unwrap();

        tasks.retain_mut(|t| t.user_id == user_id && t.task_desc.task_id == task_id);

        Ok(())
    }

    async fn fetch_categories(
        &self,
        user_id: UserId,
    ) -> anyhow::Result<Vec<TaskCategoryDescription>> {
        let categories = self.categories.lock().unwrap();

        Ok(categories
            .iter()
            .filter(|c| c.user_id == user_id)
            .map(|x| x.category_desc.clone())
            .collect())
    }

    async fn add_categories(
        &self,
        user_id: UserId,
        labels: &[&str],
    ) -> anyhow::Result<Vec<TaskCategoryDescription>> {
        let descriptions: Vec<TaskCategoryDescription> = labels
            .into_iter()
            .map(|label| TaskCategoryDescription {
                category_id: tasks::generate_random_task_id(),
                label: label.to_string(),
            })
            .collect();

        let mut categories = self.categories.lock().unwrap();

        for d in &descriptions {
            if categories
                .iter()
                .find(|x| x.user_id == user_id && x.category_desc.category_id == d.category_id)
                .is_some()
            {
                return Err(anyhow::anyhow!(
                    "could not generate unique task category id"
                ));
            }
        }

        for d in &descriptions {
            categories.push(TaskCategoryStorage {
                user_id,
                category_desc: d.clone(),
            });
        }

        Ok(descriptions)
    }
}
