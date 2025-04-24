#![feature(try_trait_v2)]

#[macro_use]
extern crate rocket;

mod api;
mod app;
mod model;
mod storage;

use std::sync::Arc;

use api::{initialize_api, Context};
use app::{
    auth::AuthService,
    repositories::{SessionsRepository, TasksRepository, UsersRepositry},
    tasks::TasksService,
};
use storage::{
    db::{self, DatabaseConnection, DatabaseConnectionRef},
    inmemory,
};

struct Environment {
    database_url: Option<String>,
}

fn read_environment() -> Environment {
    let database_url = std::env::var("DATABASE").ok();

    Environment { database_url }
}

fn init_logging() {
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();
}

struct Repositories {
    users: Arc<dyn UsersRepositry>,
    sessions: Arc<dyn SessionsRepository>,
    tasks: Arc<dyn TasksRepository>,
}

fn create_inmemory_repositories() -> Repositories {
    Repositories {
        sessions: Arc::new(inmemory::InMemorySessions::new()),
        users: Arc::new(inmemory::InMemoryUsers::new()),
        tasks: Arc::new(inmemory::InMemoryTasks::new()),
    }
}

fn create_db_repositories(db: DatabaseConnectionRef) -> Repositories {
    Repositories {
        sessions: Arc::new(db::DbSessions::new(db.clone())),
        users: Arc::new(db::DbUsers::new(db.clone())),
        tasks: Arc::new(db::DbTasks::new(db.clone())),
    }
}

fn create_context(repos: Repositories) -> Context {
    Context {
        auth: Box::new(AuthService::new(
            repos.sessions,
            repos.users,
            repos.tasks.clone(),
        )),
        tasks: Box::new(TasksService::new(repos.tasks)),
    }
}

async fn create_repos(env: &Environment) -> anyhow::Result<Repositories> {
    if let Some(uri) = &env.database_url {
        log::info!("Connecting to database: {}", uri);
        let db = Arc::new(DatabaseConnection::connect(uri)?);

        Ok(create_db_repositories(db))
    } else {
        log::info!("Using in-memory repositories, since database URI is not set.");
        Ok(create_inmemory_repositories())
    }
}

#[launch]
async fn rocket() -> _ {
    init_logging();

    log::info!("Start");

    let environment = read_environment();

    let repos = create_repos(&environment)
        .await
        .expect("failed to initialize repositories");

    let context = Arc::new(create_context(repos));

    initialize_api(context)
}
