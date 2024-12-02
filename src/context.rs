use std::sync::{Arc, OnceLock};

use rocket::State;

use crate::{
    model::{
        tasks::TaskCategoryModel, DatabaseConnection, DatabaseConnectionRef, DbError, SessionModel,
        TaskModel, UserModel,
    },
    response::ServiceUnavailableError,
};

pub type ContextState = State<Arc<Context>>;

pub struct Context {
    database: OnceLock<DatabaseConnectionRef>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            database: OnceLock::new(),
        }
    }

    pub async fn initialize_database(&self, url: &str) -> Result<(), DbError> {
        if self.database.get().is_some() {
            panic!("database is already initialized");
        }

        let db = Arc::new(DatabaseConnection::connect(url).await?);

        self.database
            .set(db)
            .ok()
            .expect("database was initialized twice");

        Ok(())
    }

    // Check if the server is initialized and ready to serve requests.
    // E.g. check database connection, etc.
    // Return the server object.
    pub fn server(&self) -> Result<Server, ServiceUnavailableError> {
        let Some(db) = self.database.get() else {
            return Err(ServiceUnavailableError::DatabaseConnection);
        };

        Ok(Server {
            database: db.clone(),
        })
    }
}

pub struct Server {
    database: DatabaseConnectionRef,
}

impl Server {
    fn reference_database(&self) -> DatabaseConnectionRef {
        self.database.clone()
    }

    pub fn sessions(&self) -> SessionModel {
        SessionModel::new(self.reference_database())
    }

    pub fn users(&self) -> UserModel {
        UserModel::new(self.reference_database())
    }

    pub fn tasks(&self) -> TaskModel {
        TaskModel::new(self.reference_database())
    }

    pub fn task_categories(&self) -> TaskCategoryModel {
        TaskCategoryModel::new(self.reference_database())
    }
}
