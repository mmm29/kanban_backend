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

    /// Creates a new user with the provided username and password,
    /// returning the ID of the created user and a [`SessionToken`], or an error otherwise.
    pub async fn create_user(
        &self,
        username: &str,
        password: &str,
    ) -> anyhow::Result<Result<(UserId, SessionToken), CreateUserError>> {
        // Validate the username.
        if !validate_username(username) {
            return Ok(Err(CreateUserError::InvalidUsername));
        }

        // Validate the password.
        if !validate_password(password) {
            return Ok(Err(CreateUserError::InvalidPassword));
        }

        // Check if the user with this username already exists.
        if self.users.does_user_exist_by_username(username).await? {
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
            self.users.find_user_with_password(username).await?
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

/// Validates the provided username, returning true if the username is valid.
/// # Example
/// ```
/// assert_eq!(validate_username("user123465"), true);
/// assert_eq!(validate_username("m"), false);
/// ```
fn validate_username(username: &str) -> bool {
    fn is_allowed_username_character(c: char) -> bool {
        c.is_alphabetic() || c.is_ascii_digit() || c == '_'
    }

    let sufficient_length = username.len() >= 6;
    let all_chars_allowed = username.chars().all(is_allowed_username_character);

    sufficient_length && all_chars_allowed
}

fn validate_password(password: &str) -> bool {
    const SPECIAL_CHARS: &[char] = &['$', '@', '!'];

    fn is_allowed_password_character(c: char) -> bool {
        c.is_alphabetic() || c.is_ascii_digit() || SPECIAL_CHARS.contains(&c)
    }

    fn any_char(password: &str, f: impl Fn(char) -> bool) -> bool {
        password.chars().any(f)
    }

    let sufficient_length = password.len() >= 8;
    let all_chars_allowed = password.chars().all(is_allowed_password_character);
    let has_lowercase_letter = any_char(password, |c| c.is_lowercase());
    let has_uppercase_letter = any_char(password, |c| c.is_uppercase());
    let has_digit = any_char(password, |c| c.is_ascii_digit());
    let has_special_symbol = any_char(password, |c| SPECIAL_CHARS.contains(&c));

    sufficient_length
        && all_chars_allowed
        && has_lowercase_letter
        && has_uppercase_letter
        && has_digit
        && has_special_symbol
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        app::auth::{CreateUserError, LoginError},
        model::{SessionToken, UserId},
        storage::inmemory,
    };

    use super::{validate_password, validate_username, AuthService};

    #[test]
    fn username_validation_test() {
        const POSITIVE: &[&str] = &[
            "Ab12345_",
            "12345Ab_",
            "AAABBb1_",
            "aaabbB1_",
            "_aaabbB1",
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ123456789_",
            "________________________________________________________",
            "user_user",
            "test_1514_test",
            // Very long string, but valid
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ123456789_\
            abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ123456789_\
            abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ123456789_\
            abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ123456789_\
            abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ123456789_\
            abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ123456789_\
            abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ123456789",
        ];

        const NEGATIVE: &[&str] = &[
            // Length < 6
            "",
            "test",
            "a",
            "%",
            "ABab5",
            // Not allowed characters
            "12345678A@",
            "ABCDEFGH1@",
            "!@$ABCDEF123",
            "ABCDEFGHIJKLMNOPQRSTUVWXYZYZ123456789@!$",
            "12345678a@",
            "abcdefh1@",
            "!@$abcdefh123",
            "abcdefghijklmnopqrstuvwxyz123456789@!$",
            "abcdefhABCDEF@",
            "!@$abcdefh_ABCDEF",
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ@!$",
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ123456789@$!_",
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ123456789@$!+",
        ];

        for &username in POSITIVE {
            assert!(
                validate_username(username),
                "username {} was expected to be valid",
                username
            );
        }

        for &username in NEGATIVE {
            assert!(
                !validate_username(username),
                "username {} was expected to be invalid",
                username
            );
        }
    }

    #[test]
    fn password_validation_test() {
        // These passwords are expected to be valid.
        const POSITIVE: &[&str] = &[
            "Ab12345@",
            "12345Ab@",
            "AAABBb1@",
            "aaabbB1@",
            "@aaabbB1",
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ123456789@$!",
            // Very long string, but valid
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ123456789@$!\
            abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ123456789@$!\
            abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ123456789@$!\
            abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ123456789@$!\
            abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ123456789@$!\
            abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ123456789@$!\
            abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ123456789",
        ];

        // These passwords are expected to be invalid.
        const NEGATIVE: &[&str] = &[
            // Length < 8
            "",
            "test",
            "a",
            "%",
            "ABab1@",
            // No lowercase
            "12345678A@",
            "ABCDEFGH1@",
            "!@$ABCDEF123",
            "ABCDEFGHIJKLMNOPQRSTUVWXYZYZ123456789@!$",
            // No uppercase
            "12345678a@",
            "abcdefh1@",
            "!@$abcdefh123",
            "abcdefghijklmnopqrstuvwxyz123456789@!$",
            // No digit
            "abcdefhABCDEF@",
            "!@$abcdefhABCDEF",
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ@!$",
            // No special char
            "ABCabc123",
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ123456789",
            // Not allowed chars
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ123456789@$!_",
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ123456789@$!+",
            "Aa123456@_",
            "Aa123456@-",
            "Aa123456@%",
            "Aa123456^%&*",
            "Aa123456@\\",
            "Aa123456@/",
            "Aa123456@\r\n",
            "Aa123456@\n",
            "Aa123456@\t",
            "Aa123456@\0", // rust strings are not null-terminated
        ];

        for &password in POSITIVE {
            assert!(
                validate_password(password),
                "password {} was expected to be valid",
                password
            );
        }

        for &password in NEGATIVE {
            assert!(
                !validate_password(password),
                "password {} was expected to be invalid",
                password
            );
        }
    }

    const USER_ID: UserId = UserId::from_raw(1);
    const USERNAME: &str = "user123";
    const USER_PASSWORD: &str = "Abc123456@";

    fn setup_inmemory_auth_service() -> AuthService {
        AuthService::new(
            Arc::new(inmemory::InMemorySessions::new()),
            Arc::new(inmemory::InMemoryUsers::new()),
            Arc::new(inmemory::InMemoryTasks::new()),
        )
    }

    async fn setup_inmemory_auth_service_with_user() -> AuthService {
        let users = inmemory::InMemoryUsers::new();

        users.add_user(USER_ID, USERNAME, USER_PASSWORD).unwrap();

        AuthService::new(
            Arc::new(inmemory::InMemorySessions::new()),
            Arc::new(users),
            Arc::new(inmemory::InMemoryTasks::new()),
        )
    }

    #[tokio::test]
    async fn login_user_not_found() -> anyhow::Result<()> {
        let auth = setup_inmemory_auth_service();

        let result = auth.login_user(USERNAME, USER_PASSWORD).await?;

        assert!(
            matches!(result, Err(LoginError::UserNotFound)),
            "login succeeded although there is no such user: {:?}",
            result
        );

        Ok(())
    }

    #[tokio::test]
    async fn login_incorrect_password() -> anyhow::Result<()> {
        let auth = setup_inmemory_auth_service_with_user().await;

        let incorrect_passwords = [
            format!("{}a", USER_PASSWORD),                        // add character
            format!("a{}", USER_PASSWORD),                        // add character
            USER_PASSWORD[..USER_PASSWORD.len() - 1].to_string(), // remove last character
            "".to_string(),
            "#".to_string(),
            "\0".to_string(),
        ];

        for password in &incorrect_passwords {
            let result = auth.login_user(USERNAME, password).await?;

            assert!(
                matches!(result, Err(LoginError::IncorrectPassword)),
                "login succeeded although password \"{}\" was incorrect: {:?}",
                password,
                result
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn login_successful() -> anyhow::Result<()> {
        let auth = setup_inmemory_auth_service_with_user().await;

        let result = auth.login_user(USERNAME, USER_PASSWORD).await?;

        assert!(
            result.is_ok(),
            "login failed, but should have succeeded: {:?}",
            result
        );

        Ok(())
    }

    #[tokio::test]
    async fn username() -> anyhow::Result<()> {
        let auth = setup_inmemory_auth_service_with_user().await;

        let (user_id, _token) = auth.login_user(USERNAME, USER_PASSWORD).await?.unwrap();

        let username = auth
            .get_username(user_id)
            .await?
            .expect("no username at all");

        assert_eq!(username, "user123", "Wrong username");

        Ok(())
    }

    #[tokio::test]
    async fn session_valid() -> anyhow::Result<()> {
        let auth = setup_inmemory_auth_service_with_user().await;

        let (login_user_id, session_token) =
            auth.login_user(USERNAME, USER_PASSWORD).await?.unwrap();

        let session_token_user_id = auth
            .get_authorized_user_id(&session_token)
            .await?
            .expect("failed to get user from session token");

        assert_eq!(login_user_id, session_token_user_id, "Wrong user id");

        Ok(())
    }

    #[tokio::test]
    async fn session_invalid() -> anyhow::Result<()> {
        let auth = setup_inmemory_auth_service_with_user().await;

        let (_login_user_id, _session_token) =
            auth.login_user(USERNAME, USER_PASSWORD).await?.unwrap();

        let authorized_user_id = auth
            .get_authorized_user_id(&SessionToken::generate_random())
            .await?;

        assert!(
            authorized_user_id.is_none(),
            "found authorized user, although used a wrong session token: {:?}",
            authorized_user_id
        );

        Ok(())
    }

    #[tokio::test]
    async fn create_user_already_existing() -> anyhow::Result<()> {
        let auth = setup_inmemory_auth_service_with_user().await;

        let result = auth.create_user(USERNAME, USER_PASSWORD).await?;

        assert!(
            matches!(result, Err(CreateUserError::UserAlreadyExists)),
            "create user succeeded although existing username was used: {:?}",
            result
        );

        Ok(())
    }

    #[tokio::test]
    async fn create_user_invalid_password() -> anyhow::Result<()> {
        let auth = setup_inmemory_auth_service();

        const INVALID_PASSWORD: &str = "ABc123456";

        assert!(!validate_password(INVALID_PASSWORD));

        let result = auth.create_user("user123", INVALID_PASSWORD).await?;

        assert!(
            matches!(result, Err(CreateUserError::InvalidPassword)),
            "create user succeeded although invalid password was used: {:?}",
            result
        );

        Ok(())
    }

    #[tokio::test]
    async fn create_user_invalid_username() -> anyhow::Result<()> {
        let auth = setup_inmemory_auth_service();

        const INVALID_USERNAME: &str = "user1";

        assert!(!validate_username(INVALID_USERNAME));

        let result = auth.create_user(INVALID_USERNAME, "ABc123456@").await?;

        assert!(
            matches!(result, Err(CreateUserError::InvalidUsername)),
            "create user succeeded although invalid username was used: {:?}",
            result
        );

        Ok(())
    }

    #[tokio::test]
    async fn create_user_successful() -> anyhow::Result<()> {
        let auth = setup_inmemory_auth_service();

        let result = auth.create_user("user123", "ABc123456@").await?;

        assert!(
            matches!(result, Ok(_)),
            "create user failed but should have succeeded: {:?}",
            result
        );

        let (user_id, token) = result.unwrap();

        let session_user_id = auth.get_authorized_user_id(&token).await?.unwrap();
        assert_eq!(session_user_id, user_id);

        Ok(())
    }

    #[tokio::test]
    async fn create_user_many() -> anyhow::Result<()> {
        let auth = setup_inmemory_auth_service();

        const NUM_USERS: usize = 1000;
        const BASE_USERNAME: &str = "user123";
        const BASE_PASSWORD: &str = "ABc123456@";

        let mut created_users: Vec<(UserId, String, SessionToken)> = Vec::with_capacity(NUM_USERS);

        for n in 0..NUM_USERS {
            let username = format!("{}{}", BASE_USERNAME, n);
            let password = format!("{}{}", BASE_PASSWORD, n);

            let (user_id, token) = auth
                .create_user(&username, &password)
                .await?
                .expect("failed to create user");

            created_users.push((user_id, username, token));
        }

        // Verify that all users exist.
        for (user_id, username, token) in created_users {
            let session_user_id = auth
                .get_authorized_user_id(&token)
                .await?
                .expect("could not obtain user id from session token");

            assert_eq!(user_id, session_user_id);

            let obtained_username = auth.get_username(user_id).await?.expect("no username");
            assert_eq!(username, obtained_username);
        }

        Ok(())
    }
}
