use std::collections::HashMap;

use anyhow::anyhow;
use rocket::serde::{json::Json, Deserialize, Serialize};

use crate::model::{
    tasks::{TaskCategoryDescription, TaskDescription},
    TaskCategoryId, TaskId,
};

use super::super::{ContextState, Response};

use super::auth::AuthorizedUser;

#[derive(Serialize)]
pub struct Task {
    task_id: TaskId,
    label: String,
    description: String,
}

#[derive(Serialize)]
pub struct TaskCategory {
    category_id: TaskCategoryId,
    label: String,
    ordered_tasks: Vec<Box<Task>>,
}

#[derive(Serialize)]
pub struct TasksBoard {
    ordered_categories: Vec<Box<TaskCategory>>,
}

fn make_tasks_board(
    tasks: &[TaskDescription],
    categories: &[TaskCategoryDescription],
) -> anyhow::Result<TasksBoard> {
    let mut ordered_categories: Vec<Box<TaskCategory>> = categories
        .iter()
        .map(|ct| {
            Box::new(TaskCategory {
                category_id: ct.category_id.clone(),
                label: ct.label.clone(),
                ordered_tasks: Vec::new(),
            })
        })
        .collect();

    let indices: HashMap<String, usize> = HashMap::from_iter(
        ordered_categories
            .iter()
            .enumerate()
            .map(|(idx, ct)| (ct.category_id.clone(), idx)),
    );

    for task in tasks {
        let Some(&idx) = indices.get(&task.category_id) else {
            return Err(anyhow!(
                "task {} is assigned to category {}, but the category is missing",
                task.task_id,
                task.category_id
            ));
        };

        ordered_categories[idx].ordered_tasks.push(Box::new(Task {
            task_id: task.task_id.clone(),
            label: task.label.clone(),
            description: task.description.clone(),
        }));
    }

    Ok(TasksBoard { ordered_categories })
}

#[get("/tasks")]
pub async fn get_tasks(context: &ContextState, user: AuthorizedUser) -> Response<TasksBoard> {
    let tasks = &context.tasks;

    let task_descriptions = tasks.fetch_tasks(user.user_id).await?;
    let category_descriptions = tasks.fetch_categories(user.user_id).await?;
    let tasks_board = make_tasks_board(&task_descriptions, &category_descriptions)?;

    Response::from_data(tasks_board)
}

#[post("/tasks", format = "application/json", data = "<data>")]
pub async fn create_task(
    context: &ContextState,
    user: AuthorizedUser,
    data: Json<TaskInputData>,
) -> Response<Task> {
    let tasks = &context.tasks;

    let task_id = tasks
        .create_task(
            user.user_id,
            &data.label,
            &data.description,
            &data.categoryId,
        )
        .await?;

    Response::from_data(Task {
        task_id,
        label: data.label.clone(),
        description: data.description.clone(),
    })
}

#[delete("/tasks/<task_id>")]
pub async fn delete_task(
    context: &ContextState,
    user: AuthorizedUser,
    task_id: &str,
) -> Response<()> {
    let tasks = &context.tasks;

    tasks.delete_task(user.user_id, task_id).await?;

    Response::from_data(())
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub struct TaskInputData {
    categoryId: TaskCategoryId,
    label: String,
    description: String,
}

#[put("/tasks/<task_id>", format = "application/json", data = "<data>")]
pub async fn modify_task(
    context: &ContextState,
    user: AuthorizedUser,
    task_id: &str,
    data: Json<TaskInputData>,
) -> Response<()> {
    let tasks = &context.tasks;

    tasks
        .modify_task(
            user.user_id,
            task_id,
            &data.label,
            &data.description,
            &data.categoryId,
        )
        .await?;

    Response::from_data(())
}
