use std::{future::Future, pin::Pin, sync::Arc};

use crate::model::{SessionToken, UserId};

use super::repositories::{SessionsRepository, TasksRepository, UsersRepositry};

pub struct AuthService {
    sessions: Arc<dyn SessionsRepository + Send + Sync>,
    users: Arc<dyn UsersRepositry + Send + Sync>,
    on_created_user: OnCreatedUserCb,
}

pub type OnCreatedUserCb =
    Box<dyn Fn(UserId) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

#[derive(Debug)]
pub enum LoginError {
    UserNotFound,
    IncorrectPassword,
}

#[derive(Debug)]
pub enum CreateUserError {
    InvalidUsername,
    InvalidPassword,
    UserAlreadyExists,
}

impl AuthService {
    pub fn new(
        sessions: Arc<dyn SessionsRepository>,
        users: Arc<dyn UsersRepositry>,
        tasks: Arc<dyn TasksRepository>,
    ) -> Self {
        let on_created_user: OnCreatedUserCb = Box::new(move |user_id| {
            async fn add_user_default_categories(
                tasks: &dyn TasksRepository,
                user_id: UserId,
            ) -> anyhow::Result<()> {
                const DEFAULT_CATEGORIES: &[&str] = &["ToDo", "In progress", "Completed"];

                tasks.add_categories(user_id, DEFAULT_CATEGORIES).await?;

                Ok(())
            }

            let tasks_c = tasks.clone();

            Box::pin(async move {
                add_user_default_categories(tasks_c.as_ref(), user_id)
                    .await
                    .expect("add_user_default_categories");
            })
        });

        Self {
            sessions,
            users,
            on_created_user,
        }
    }

    pub async fn get_authorized_user_id(
        &self,
        token: &SessionToken,
    ) -> anyhow::Result<Option<UserId>> {
        self.sessions.get_authorized_user_id(token).await
    }

    pub async fn create_user(
        &self,
        username: &str,
        password: &str,
    ) -> anyhow::Result<Result<(UserId, SessionToken), CreateUserError>> {
        // Validate the username.
        if !validate_username(&username) {
            return Ok(Err(CreateUserError::InvalidUsername));
        }

        // Validate the password.
        if !validate_password(&password) {
            return Ok(Err(CreateUserError::InvalidPassword));
        }

        // Check if the user with this username already exists.
        if self.users.does_user_exist_by_username(&username).await? {
            return Ok(Err(CreateUserError::UserAlreadyExists));
        }

        // Create the user.
        let user_id = self.users.create_user(username, password).await?;

        // Create a session token for the user.
        let token = self.sessions.create_user_session(user_id).await?;

        (self.on_created_user)(user_id).await;

        Ok(Ok((user_id, token)))
    }

    pub async fn get_username(&self, user_id: UserId) -> anyhow::Result<Option<String>> {
        self.users.get_username(user_id).await
    }

    pub async fn login_user(
        &self,
        username: &str,
        password: &str,
    ) -> anyhow::Result<Result<(UserId, SessionToken), LoginError>> {
        // Find the user by username.
        let Some((user_id, actual_password)) =
            self.users.find_user_with_password(&username).await?
        else {
            return Ok(Err(LoginError::UserNotFound));
        };

        // Check if the passwords match.
        // TODO: hash the password in the database
        if actual_password != password {
            return Ok(Err(LoginError::IncorrectPassword));
        }

        // Create a session token for the user.
        let token = self.sessions.create_user_session(user_id).await?;

        Ok(Ok((user_id, token)))
    }
}

fn validate_username(username: &str) -> bool {
    fn is_allowed_username_character(c: char) -> bool {
        c.is_alphabetic() || c.is_digit(10) || c == '_'
    }

    let sufficient_length = username.len() >= 6;
    let all_chars_allowed = username.chars().all(|c| is_allowed_username_character(c));

    sufficient_length && all_chars_allowed
}

fn validate_password(password: &str) -> bool {
    const SPECIAL_CHARS: &[char] = &['$', '@', '!'];

    fn is_allowed_password_character(c: char) -> bool {
        c.is_alphabetic() || c.is_digit(10) || SPECIAL_CHARS.into_iter().any(|&sc| sc == c)
    }

    fn any_char(password: &str, f: impl Fn(char) -> bool) -> bool {
        password.chars().any(f)
    }

    let sufficient_length = password.len() >= 8;
    let all_chars_allowed = password.chars().all(|c| is_allowed_password_character(c));
    let has_lowercase_letter = any_char(password, |c| c.is_lowercase());
    let has_uppercase_letter = any_char(password, |c| c.is_uppercase());
    let has_digit = any_char(password, |c| c.is_digit(10));
    let has_special_symbol = any_char(password, |c| SPECIAL_CHARS.into_iter().any(|&sc| sc == c));

    sufficient_length
        && all_chars_allowed
        && has_lowercase_letter
        && has_uppercase_letter
        && has_digit
        && has_special_symbol
}
