use crate::{
    constants,
    error::ServiceError,
    models::{
        response::ResponseBody,
        users::{ChangePasswordDTO, LoginDTO, UserDTO},
    },
    services::users::{change_password, get_all_users, get_user_by_id, login, signup},
};
use ntex::web::{self, HttpResponse};

pub async fn get_all_users_controller() -> Result<HttpResponse, ServiceError> {
    match get_all_users().await {
        Ok(users) => Ok(HttpResponse::Ok().json(&ResponseBody::new(constants::MESSAGE_OK, users))),
        Err(err) => Err(err),
    }
}

pub async fn get_user_by_id_controller(
    user_id: web::types::Path<i32>,
) -> Result<HttpResponse, ServiceError> {
    match get_user_by_id(user_id.into_inner()).await {
        Ok(user) => Ok(HttpResponse::Ok().json(&ResponseBody::new(constants::MESSAGE_OK, user))),
        Err(err) => Err(err),
    }
}

pub async fn login_controller(
    login_dto: web::types::Json<LoginDTO>,
) -> Result<HttpResponse, ServiceError> {
    match login(login_dto.0) {
        Ok(token_res) => Ok(HttpResponse::Ok().json(&ResponseBody::new(
            constants::MESSAGE_LOGIN_SUCCESS,
            token_res,
        ))),
        Err(err) => Err(err),
    }
}

pub async fn signup_controller(
    user_dto: web::types::Json<UserDTO>,
) -> Result<HttpResponse, ServiceError> {
    match signup(user_dto.0) {
        Ok(message) => Ok(HttpResponse::Ok().json(&ResponseBody::new(&message, constants::EMPTY))),
        Err(err) => Err(err),
    }
}

pub async fn change_password_controller(
    user_id: web::types::Path<i32>,
    change_password_dto: web::types::Json<ChangePasswordDTO>,
) -> Result<HttpResponse, ServiceError> {
    match change_password(user_id.into_inner(), change_password_dto.0) {
        Ok(message) => Ok(HttpResponse::Ok().json(&ResponseBody::new(&message, constants::EMPTY))),
        Err(err) => Err(err),
    }
}
