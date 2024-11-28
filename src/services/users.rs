use crate::constants;
use crate::constants::MESSAGE_INTERNAL_SERVER_ERROR;
use crate::constants::MESSAGE_LOGIN_FAILED;
use crate::constants::MESSAGE_SIGNUP_SUCCESS;
use crate::constants::USERSNAME_ALREADY_EXIST;
use crate::error::ServiceError;
use crate::models::token::TokenBodyResponse;
use crate::models::token::UserToken;
use crate::models::users::UserSlim;
use crate::models::users::{LoginDTO, RegisterUser, User};

pub async fn get_all_users() -> Result<Vec<UserSlim>, ServiceError> {
    match User::all() {
        Some(users) => Ok(users),
        None => Err(ServiceError::InternalServerError {
            error_message: constants::MESSAGE_INTERNAL_SERVER_ERROR,
        }),
    }
}

pub async fn get_user_by_id(user_id: i32) -> Result<User, ServiceError> {
    let user = User::find_by_id(user_id);
    if user.is_some() {
        Ok(user.unwrap())
    } else {
        Err(ServiceError::NotFound {
            error_message: constants::MESSAGE_USER_NOT_FOUND,
        })
    }
}

pub fn signup(user: RegisterUser) -> Result<&'static str, ServiceError> {
    let existing_user = User::find_user_by_username(&user.username);

    if existing_user.is_none() {
        let new_user = RegisterUser {
            password: bcrypt::hash(&user.password, bcrypt::DEFAULT_COST).unwrap(),
            ..user
        };
        let inserted_user = User::insert(new_user);

        if inserted_user.is_some() {
            Ok(MESSAGE_SIGNUP_SUCCESS)
        } else {
            Err(ServiceError::InternalServerError {
                error_message: MESSAGE_INTERNAL_SERVER_ERROR,
            })
        }
    } else {
        Err(ServiceError::BadRequest {
            error_message: USERSNAME_ALREADY_EXIST,
        })
    }
}

pub fn login(login: LoginDTO) -> Result<TokenBodyResponse, ServiceError> {
    let user = User::find_user_by_username(&login.username);
    if user.is_some() {
        let user = user.unwrap();
        if !user.password.is_empty() && bcrypt::verify(&login.password, &user.password).unwrap() {
            let token = UserToken::generate_token(&user);
            let token_res = TokenBodyResponse { token };
            Ok(token_res)
        } else {
            Err(ServiceError::Unauthorized {
                error_message: MESSAGE_LOGIN_FAILED,
            })
        }
    } else {
        Err(ServiceError::NotFound {
            error_message: MESSAGE_LOGIN_FAILED,
        })
    }
}

/* pub fn change_password(
    user_id: i32,
    change_password: ChangePasswordDTO,
) {
    /* let mut conn = database::get_conn();
    match User::find_by_id(user_id) {
        Ok(mut user) => {
            if !user.password.is_empty()
                && bcrypt::verify(&change_password.old_password, &user.password).unwrap()
            {
                user.password =
                    bcrypt::hash(&change_password.new_password, bcrypt::DEFAULT_COST).unwrap();
                match diesel::update(users::table.find(user_id))
                    .set(&user)
                    .execute(&mut conn)
                {
                    Ok(_) => Ok(constants::MESSAGE_CHANGE_PASSWORD_SUCCESS),
                    Err(_) => Err(ServiceError::InternalServerError {
                        error_message: constants::MESSAGE_INTERNAL_SERVER_ERROR,
                    }),
                }
            } else {
                Err(ServiceError::Unauthorized {
                    error_message: constants::MESSAGE_PASSWORD_NOT_MATCH,
                })
            }
        }
        Err(_) => Err(ServiceError::NotFound {
            error_message: constants::MESSAGE_USER_NOT_FOUND,
        }),
    } */
    let user = User::find_by_id(user_id);
}
 */
