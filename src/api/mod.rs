use std::sync::Arc;

use rocket::{Build, Rocket};

mod context;
pub mod controllers;
mod response;

pub use context::{Context, ContextState};
pub use response::Response;

/// Creates [`Rocket`] object that serves API requests using the provided context.
pub fn initialize_api(context: Arc<Context>) -> Rocket<Build> {
    let api_routes = routes![
        controllers::auth::login,
        controllers::auth::register,
        controllers::auth::get_user,
        controllers::tasks::get_tasks,
        controllers::tasks::create_task,
        controllers::tasks::delete_task,
        controllers::tasks::modify_task,
    ];

    rocket::build().manage(context).mount("/api", api_routes)
}
