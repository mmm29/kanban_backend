use rand::Rng;

pub fn generate_random_task_id() -> String {
    let mut rng = rand::thread_rng();

    let mut bytes: [u8; 16] = [0; 16];
    bytes.iter_mut().for_each(|b| *b = rng.gen());

    hex::encode(bytes)
}

pub type TaskId = String;
pub type TaskCategoryId = String;

#[derive(Debug, Clone)]
pub struct TaskDescription {
    pub task_id: TaskId,
    pub label: String,
    pub description: String,
    pub category_id: TaskCategoryId,
}

#[derive(Debug, Clone)]
pub struct TaskCategoryDescription {
    pub category_id: TaskCategoryId,
    pub label: String,
}
