#![feature(try_trait_v2)]

#[macro_use]
extern crate rocket;

use std::sync::Arc;

use context::Context;

mod context;
mod controllers;
mod model;
mod response;

fn initialize_database(context: Arc<Context>, database_url: &str) {
    let database_url = database_url.to_string();

    rocket::tokio::task::spawn(async move {
        log::info!("Connecting to database: {}", database_url);

        context
            .initialize_database(&database_url)
            .await
            .expect("could not initialize database");

        log::info!("Connected to database: {}", database_url);
    });
}

struct Environment {
    database_url: Option<String>,
}

fn read_environment() -> Environment {
    let database_url = std::env::var("DATABASE").ok();

    Environment { database_url }
}

fn init_logging() {
    env_logger::init();
}

#[launch]
async fn rocket() -> _ {
    init_logging();

    let context = Arc::new(Context::new());
    let environment = read_environment();

    let database_url = environment
        .database_url
        .expect("no DATABASE environment variable");

    initialize_database(Arc::clone(&context), &database_url);

    let api_routes = routes![
        controllers::auth::login,
        controllers::auth::register,
        controllers::auth::get_user,
        controllers::tasks::get_tasks,
        controllers::tasks::create_task,
        controllers::tasks::delete_task,
        controllers::tasks::modify_task,
    ];

    rocket::build()
        .manage(Arc::clone(&context))
        .mount("/api", api_routes)
}
