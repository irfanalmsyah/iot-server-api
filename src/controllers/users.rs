use crate::{
    constants, db,
    error::ServiceError,
    models::{
        response::ResponseBody,
        users::{ChangePasswordDTO, LoginDTO, UserDTO},
    },
    services::users::{change_password, get_all_users, get_user_by_id, login, signup},
};
use ntex::web::{self, HttpResponse};

pub async fn get_all_users_controller(
    pool: web::types::State<db::Pool>,
) -> Result<HttpResponse, ServiceError> {
    match get_all_users(&pool).await {
        Ok(users) => Ok(HttpResponse::Ok().json(&ResponseBody::new(constants::MESSAGE_OK, users))),
        Err(err) => Err(err),
    }
}

pub async fn get_user_by_id_controller(
    pool: web::types::State<db::Pool>,
    user_id: web::types::Path<i32>,
) -> Result<HttpResponse, ServiceError> {
    match get_user_by_id(&pool, user_id.into_inner()).await {
        Ok(user) => Ok(HttpResponse::Ok().json(&ResponseBody::new(constants::MESSAGE_OK, user))),
        Err(err) => Err(err),
    }
}

pub async fn login_controller(
    pool: web::types::State<db::Pool>,
    login_dto: web::types::Json<LoginDTO>,
) -> Result<HttpResponse, ServiceError> {
    match login(&pool, login_dto.0) {
        Ok(token_res) => Ok(HttpResponse::Ok().json(&ResponseBody::new(
            constants::MESSAGE_LOGIN_SUCCESS,
            token_res,
        ))),
        Err(err) => Err(err),
    }
}

pub async fn signup_controller(
    pool: web::types::State<db::Pool>,
    user_dto: web::types::Json<UserDTO>,
) -> Result<HttpResponse, ServiceError> {
    match signup(&pool, user_dto.0) {
        Ok(message) => Ok(HttpResponse::Ok().json(&ResponseBody::new(&message, constants::EMPTY))),
        Err(err) => Err(err),
    }
}

pub async fn change_password_controller(
    pool: web::types::State<db::Pool>,
    user_id: web::types::Path<i32>,
    change_password_dto: web::types::Json<ChangePasswordDTO>,
) -> Result<HttpResponse, ServiceError> {
    match change_password(&pool, user_id.into_inner(), change_password_dto.0) {
        Ok(message) => Ok(HttpResponse::Ok().json(&ResponseBody::new(&message, constants::EMPTY))),
        Err(err) => Err(err),
    }
}
