use crate::error::ServiceError::{
    BadRequest, Forbidden, InternalServerError, NotFound, Unauthorized,
};
use crate::{constants::EMPTY, models::response::ResponseBody};
use derive_more::{Display, Error};
use ntex::web::{HttpRequest, HttpResponse, WebResponseError};

#[derive(Debug, Display, Error)]
pub enum ServiceError {
    #[display(fmt = "{error_message}")]
    BadRequest { error_message: &'static str },

    #[display(fmt = "{error_message}")]
    Unauthorized { error_message: &'static str },

    #[display(fmt = "{error_message}")]
    Forbidden { error_message: &'static str },

    #[display(fmt = "{error_message}")]
    NotFound { error_message: &'static str },

    #[display(fmt = "{error_message}")]
    InternalServerError { error_message: &'static str },
}

impl WebResponseError for ServiceError {
    fn error_response(&self, _: &HttpRequest) -> HttpResponse {
        match *self {
            BadRequest { ref error_message } => {
                HttpResponse::BadRequest().json(&ResponseBody::new(&error_message, EMPTY))
            }
            Unauthorized { ref error_message } => {
                HttpResponse::Unauthorized().json(&ResponseBody::new(&error_message, EMPTY))
            }
            Forbidden { ref error_message } => {
                HttpResponse::Forbidden().json(&ResponseBody::new(&error_message, EMPTY))
            }
            NotFound { ref error_message } => {
                HttpResponse::NotFound().json(&ResponseBody::new(&error_message, EMPTY))
            }
            InternalServerError { ref error_message } => {
                HttpResponse::InternalServerError().json(&ResponseBody::new(&error_message, EMPTY))
            }
        }
    }
}
