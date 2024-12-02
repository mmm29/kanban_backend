use std::{convert::Infallible, error::Error};

use anyhow::anyhow;
use rocket::{
    http::{Cookie, CookieJar, Status},
    outcome::try_outcome,
    request::{FromRequest, Outcome},
    serde::{json::Json, Deserialize, Serialize},
    Request,
};

use crate::{
    app::auth::{CreateUserError, LoginError},
    model::{SessionToken, UserId},
};

use super::super::{ContextState, Response};

struct SessionTokenCookie<'a>(&'a CookieJar<'a>);

impl<'a> SessionTokenCookie<'a> {
    const COOKIE_NAME: &'static str = "session";

    pub fn new(jar: &'a CookieJar<'a>) -> Self {
        Self(jar)
    }

    pub fn read(&self) -> Option<SessionToken> {
        let raw = self.0.get(Self::COOKIE_NAME)?;

        SessionToken::from_str(raw.value_trimmed())
    }

    pub fn write(&self, session_token: &SessionToken) {
        let s = session_token.as_str().to_string();

        let cookie = Cookie::build((Self::COOKIE_NAME, s))
            .http_only(true)
            .build();

        self.0.add(cookie);
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for SessionToken {
    type Error = Infallible;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match SessionTokenCookie::new(request.cookies()).read() {
            Some(s) => Outcome::Success(s),
            None => Outcome::Forward(Status::Unauthorized),
        }
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

        match context.auth.get_authorized_user_id(&session_token).await {
            Ok(Some(user_id)) => Outcome::Success(AuthorizedUser {
                user_id,
                session_token,
            }),
            Ok(None) => Outcome::Forward(Status::Unauthorized),
            Err(err) => return Outcome::Error((Status::InternalServerError, Some(err.into()))),
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

#[post("/login", format = "application/json", data = "<user>")]
pub async fn login(
    context: &ContextState,
    jar: &CookieJar<'_>,
    user: Json<LoginParams>,
) -> Response<UserResponse> {
    let auth = &context.auth;

    match auth.login_user(&user.username, &user.password).await? {
        Ok((_user_id, token)) => {
            SessionTokenCookie::new(jar).write(&token);

            Response::from_data(UserResponse {
                username: user.username.to_string(),
            })
        }
        Err(LoginError::UserNotFound) => Response::from_error("user_not_found"),
        Err(LoginError::IncorrectPassword) => Response::from_error("incorrect_password"),
    }
}

#[post("/register", format = "application/json", data = "<user>")]
pub async fn register(
    context: &ContextState,
    jar: &CookieJar<'_>,
    user: Json<LoginParams>,
) -> Response<UserResponse> {
    let auth = &context.auth;

    match auth.create_user(&user.username, &user.password).await? {
        Ok((_user_id, token)) => {
            SessionTokenCookie::new(jar).write(&token);

            Response::from_data(UserResponse {
                username: user.username.to_string(),
            })
        }
        Err(CreateUserError::InvalidUsername) => Response::from_error("invalid_username"),
        Err(CreateUserError::InvalidPassword) => Response::from_error("invalid_password"),
        Err(CreateUserError::UserAlreadyExists) => Response::from_error("user_already_exists"),
    }
}

#[get("/user")]
pub async fn get_user(
    context: &ContextState,
    authorized_user: AuthorizedUser,
) -> Response<UserResponse> {
    let auth = &context.auth;

    let username = auth
        .get_username(authorized_user.user_id)
        .await?
        .ok_or_else(|| anyhow!("no username"))?;

    Response::from_data(UserResponse { username })
}
