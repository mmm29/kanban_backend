use std::{convert::Infallible, error::Error, ops::FromResidual};

use rocket::{
    http::Status,
    response,
    serde::{json::Json, Serialize},
    Request,
};

use crate::model::database::DbError;

#[derive(Debug)]
pub enum Response<T> {
    Success(Json<ResponseBody<T>>),
    #[allow(unused)]
    Unauthorized,
    #[allow(unused)]
    BadRequest,
    ServiceUnavailable(ServiceUnavailableError),
    ServerError(Box<dyn Error>),
}

#[derive(Debug, Serialize)]
pub struct ResponseBody<T> {
    error_code: &'static str,
    data: Option<T>,
}

impl<T> Response<T> {
    pub fn from_error(error_code: &'static str) -> Self {
        Self::Success(Json(ResponseBody {
            error_code,
            data: None,
        }))
    }

    pub fn from_data(data: T) -> Self {
        Self::Success(Json(ResponseBody {
            error_code: "",
            data: Some(data),
        }))
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ServiceUnavailableError {
    #[error("database is not connected")]
    DatabaseConnection,
}

impl<'r, 'o: 'r, T: Serialize> response::Responder<'r, 'o> for Response<T> {
    fn respond_to(self, request: &'r Request<'_>) -> response::Result<'o> {
        match self {
            Response::Success(r) => r.respond_to(request),
            Response::Unauthorized => Status::Unauthorized.respond_to(request),
            Response::BadRequest => Status::BadRequest.respond_to(request),
            Response::ServiceUnavailable(_reason) => {
                rocket::error_!("ServiceUnavailable: {:?}", _reason);
                Status::ServiceUnavailable.respond_to(request)
            }
            Response::ServerError(_error) => {
                rocket::error_!("ServerError: {:?}", _error);
                Status::InternalServerError.respond_to(request)
            }
        }
    }
}

impl<T> FromResidual<Result<Infallible, ServiceUnavailableError>> for Response<T> {
    fn from_residual(residual: Result<Infallible, ServiceUnavailableError>) -> Self {
        Self::ServiceUnavailable(match residual {
            Err(x) => x,
        })
    }
}

impl<T> FromResidual<Result<Infallible, DbError>> for Response<T> {
    fn from_residual(residual: Result<Infallible, DbError>) -> Self {
        Self::ServerError(match residual {
            Err(x) => x.into(),
        })
    }
}
