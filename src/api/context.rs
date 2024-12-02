use std::sync::Arc;

use rocket::State;

use crate::app::{auth::AuthService, tasks::TasksService};

pub type ContextState = State<Arc<Context>>;

pub struct Context {
    pub auth: Box<AuthService>,
    pub tasks: Box<TasksService>,
}
