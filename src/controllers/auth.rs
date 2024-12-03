use std::{convert::Infallible, error::Error};

use rocket::{
    http::{Cookie, CookieJar, Status},
    outcome::try_outcome,
    request::{FromRequest, Outcome},
    serde::{json::Json, Deserialize, Serialize},
    Request,
};

use crate::{
    context::{self, ContextState},
    model::{DbError, SessionToken, UserId},
    response::Response,
};

const SESSION_COOKIE_NAME: &str = "session";

#[rocket::async_trait]
impl<'r> FromRequest<'r> for SessionToken {
    type Error = Infallible;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let Some(session) = request.cookies().get(SESSION_COOKIE_NAME) else {
            return Outcome::Forward(Status::Unauthorized);
        };

        let Some(session_token) = SessionToken::from_str(session.value_trimmed()) else {
            return Outcome::Forward(Status::BadRequest);
        };

        Outcome::Success(session_token)
    }
}

pub struct AuthorizedUser {
    pub user_id: UserId,
    #[allow(unused)]
    pub session_token: SessionToken,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthorizedUser {
    type Error = Option<Box<dyn Error>>;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let session_token = try_outcome!(request
            .guard::<SessionToken>()
            .await
            .map_error(|(x, _)| (x, None)));

        let context = ContextState::get(request.rocket()).expect("no context");

        let server = match context.server() {
            Ok(server) => server,
            Err(reason) => {
                return Outcome::Error((
                    Status::InternalServerError,
                    Some(Box::new(reason).into()),
                ));
            }
        };

        let sessions = server.sessions();

        match sessions.get_authorized_user_id(&session_token).await {
            Ok(Some(user_id)) => Outcome::Success(AuthorizedUser {
                user_id,
                session_token,
            }),
            Ok(None) => Outcome::Forward(Status::Unauthorized),
            Err(err) => {
                return Outcome::Error((Status::InternalServerError, Some(Box::new(err).into())))
            }
        }
    }
}

#[derive(Deserialize)]
pub struct LoginParams {
    username: String,
    password: String,
}

#[derive(Serialize)]
pub struct UserResponse {
    username: String,
}

fn set_session_token_cookie(jar: &CookieJar, token: &SessionToken) {
    jar.add(
        Cookie::build((SESSION_COOKIE_NAME, token.as_str().to_string()))
            .http_only(true)
            .build(),
    );
}

#[post("/login", format = "application/json", data = "<user>")]
pub async fn login(
    context: &ContextState,
    jar: &CookieJar<'_>,
    user: Json<LoginParams>,
) -> Response<UserResponse> {
    let server = context.server()?;
    let sessions = server.sessions();
    let users = server.users();

    // Find the user by username.
    let Some((user_id, password)) = users.find_user_with_password(&user.username).await? else {
        // Return "user not found" error.
        return Response::from_error("user_not_found");
    };

    // Check if the passwords match.
    // TODO: hash the password in the database
    if password != user.password {
        // Passwords do not match.s
        return Response::from_error("incorrect_password");
    }

    // Create a session token for the user.
    let token = sessions.create_user_session(user_id).await?;

    // Set session token cookie.
    set_session_token_cookie(jar, &token);

    Response::from_data(UserResponse {
        username: user.username.to_string(),
    })
}

async fn add_user_default_categories(
    server: &context::Server,
    user_id: UserId,
) -> Result<(), DbError> {
    const DEFAULT_CATEGORIES: &[&str] = &["ToDo", "In progress", "Completed"];

    let task_categories = server.task_categories();

    task_categories
        .add_categories(user_id, DEFAULT_CATEGORIES)
        .await?;
    Ok(())
}

async fn on_user_register(server: &context::Server, user_id: UserId) -> Result<(), DbError> {
    add_user_default_categories(server, user_id).await?;
    Ok(())
}

fn validate_password(password: &str) -> bool {
    const SPECIAL_CHARS: &[char] = &['$', '@', '!'];

    fn is_allowed_password_character(c: char) -> bool {
        c.is_alphabetic() || c.is_digit(10) || SPECIAL_CHARS.into_iter().any(|&sc| sc == c)
    }

    fn any_char(password: &str, f: impl Fn(char) -> bool) -> bool {
        password.chars().any(f)
    }

    let valid_length = password.len() >= 8;
    let all_chars_allowed = password.chars().all(|c| is_allowed_password_character(c));
    let has_lowercase_letter = any_char(password, |c| c.is_lowercase());
    let has_uppercase_letter = any_char(password, |c| c.is_uppercase());
    let has_digit = any_char(password, |c| c.is_digit(10));
    let has_special_symbol = any_char(password, |c| SPECIAL_CHARS.into_iter().any(|&sc| sc == c));

    valid_length
        && all_chars_allowed
        && has_lowercase_letter
        && has_uppercase_letter
        && has_digit
        && has_special_symbol
}

#[post("/register", format = "application/json", data = "<user>")]
pub async fn register(
    context: &ContextState,
    jar: &CookieJar<'_>,
    user: Json<LoginParams>,
) -> Response<UserResponse> {
    let server = context.server()?;
    let sessions = server.sessions();
    let users = server.users();

    // Check if the user with this username already exists.
    if users.does_user_exist_by_username(&user.username).await? {
        return Response::from_error("already_exists");
    }

    // Validate the password.
    if !validate_password(&user.password) {
        return Response::from_error("invalid_password");
    }

    // Create the user.
    let user_id = users.create_user(&user.username, &user.password).await?;

    // Create a session token for the user.
    let token = sessions.create_user_session(user_id).await?;

    // Set session token cookie.
    set_session_token_cookie(jar, &token);

    on_user_register(&server, user_id).await?;

    Response::from_data(UserResponse {
        username: user.username.to_string(),
    })
}

#[get("/user")]
pub async fn get_user(
    context: &ContextState,
    authorized_user: AuthorizedUser,
) -> Response<UserResponse> {
    let server = context.server()?;
    let users = server.users();

    let username = users.get_username(authorized_user.user_id).await?;

    Response::from_data(UserResponse { username })
}
