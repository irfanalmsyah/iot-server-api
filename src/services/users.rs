use diesel::QueryDsl;
use diesel::RunQueryDsl;
use ntex::web;
use serde_json::json;
use crate::models::token::TokenBodyResponse;
use crate::models::token::UserToken;
use crate::models::users::UserSlim;
use crate::schema::users;
use crate::{constants, db};
use crate::error::ServiceError;
use crate::models::users::{ChangePasswordDTO, LoginDTO, User, UserDTO};

pub async fn get_all_users(
    pool: &web::types::State<db::Pool>
) -> Result<Vec<UserSlim>, ServiceError> {
    match User::all(&mut pool.get().unwrap()) {
        Ok(users) => Ok(users),
        Err(_) => Err(ServiceError::InternalServerError {
            error_message: "Error loading users".to_string(),
        }),
    }
}

pub async fn get_user_by_id(
    pool: &web::types::State<db::Pool>,
    user_id: i32
) -> Result<User, ServiceError> {
    match User::find_by_id(&mut pool.get().unwrap(), user_id) {
        Ok(user) => Ok(user),
        Err(_) => Err(ServiceError::NotFound {
            error_message: format!("Person with id {} not found", user_id),
        }),
    }
}

pub fn signup(
    pool: &web::types::State<db::Pool>,
    user: UserDTO
) -> Result<String, ServiceError> {
    match User::signup(&mut pool.get().unwrap(), user) {
        Ok(message) => Ok(message),
        Err(message) => Err(ServiceError::BadRequest {
            error_message: message,
        }),
    }
}

pub fn login(
    pool: &web::types::State<db::Pool>,
    login: LoginDTO
) -> Result<TokenBodyResponse, ServiceError> {
    let mut conn = pool.get().expect("couldn't get db connection from pool");
    match User::login(&mut conn, login) {
        Some(logged_user) => {
            match serde_json::from_value(
                json!({ "token": UserToken::generate_token(&logged_user)}),
            ) {
                Ok(token_res) => {
                    return Ok(token_res);
                }
                Err(_) => {
                    return Err(ServiceError::InternalServerError {
                        error_message: constants::MESSAGE_INTERNAL_SERVER_ERROR.to_string(),
                    });
                }
            }
        }
        None => {
            return Err(ServiceError::Unauthorized {
                error_message: constants::MESSAGE_LOGIN_FAILED.to_string(),
            });
        }
    }
}

pub fn change_password(
    pool: &web::types::State<db::Pool>,
    user_id: i32,
    change_password: ChangePasswordDTO
) -> Result<String, ServiceError> {
    match User::find_by_id(&mut pool.get().unwrap(), user_id) {
        Ok(mut user) => {
            if !user.password.is_empty() && bcrypt::verify(&change_password.old_password, &user.password).unwrap() {
                user.password = bcrypt::hash(&change_password.new_password, bcrypt::DEFAULT_COST).unwrap();
                match diesel::update(users::table.find(user_id))
                    .set(&user)
                    .execute(&mut pool.get().unwrap())
                {
                    Ok(_) => Ok(constants::MESSAGE_CHANGE_PASSWORD_SUCCESS.to_string()),
                    Err(_) => Err(ServiceError::InternalServerError {
                        error_message: constants::MESSAGE_INTERNAL_SERVER_ERROR.to_string(),
                    }),
                }
            } else {
                Err(ServiceError::Unauthorized {
                    error_message: constants::MESSAGE_PASSWORD_NOT_MATCH.to_string(),
                })
            }
        }
        Err(_) => Err(ServiceError::NotFound {
            error_message: constants::MESSAGE_USER_NOT_FOUND.to_string(),
        }),
    }
}