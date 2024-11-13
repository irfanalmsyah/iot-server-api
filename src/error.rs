use crate::{constants, models::response::ResponseBody};
use derive_more::{Display, Error};
use ntex::web::{HttpRequest, HttpResponse, WebResponseError};

#[derive(Debug, Display, Error)]
pub enum ServiceError {
    #[display(fmt = "{error_message}")]
    Unauthorized { error_message: String },

    #[display(fmt = "{error_message}")]
    InternalServerError { error_message: String },

    #[display(fmt = "{error_message}")]
    BadRequest { error_message: String },

    #[display(fmt = "{error_message}")]
    NotFound { error_message: String },
}

impl WebResponseError for ServiceError {
    fn error_response(&self, _: &HttpRequest) -> HttpResponse {
        match *self {
            ServiceError::Unauthorized { ref error_message } => HttpResponse::Unauthorized()
                .json(&ResponseBody::new(&error_message, constants::EMPTY)),
            ServiceError::InternalServerError { ref error_message } => {
                HttpResponse::InternalServerError()
                    .json(&ResponseBody::new(&error_message, constants::EMPTY))
            }
            ServiceError::BadRequest { ref error_message } => HttpResponse::BadRequest()
                .json(&ResponseBody::new(&error_message, constants::EMPTY)),
            ServiceError::NotFound { ref error_message } => {
                HttpResponse::NotFound().json(&ResponseBody::new(&error_message, constants::EMPTY))
            }
        }
    }
}
