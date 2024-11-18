use bcrypt::{hash, verify, DEFAULT_COST};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    constants, db,
    schema::users::{self, dsl::*},
};

#[derive(Queryable, Selectable, Serialize, Deserialize, AsChangeset)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password: String,
    pub status: Option<bool>,
    pub isadmin: Option<bool>,
}

#[derive(Insertable, Serialize, Deserialize)]
#[diesel(table_name = users)]
pub struct UserDTO {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct LoginDTO {
    pub username: String,
    pub password: String,
}

#[derive(Insertable, Serialize, Deserialize)]
#[diesel(table_name = users)]
pub struct LoginInfoDTO {
    pub username: String,
}

#[derive(Serialize, Deserialize)]
pub struct ChangePasswordDTO {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Queryable, Serialize)]
pub struct UserSlim {
    id: i32,
    username: String,
    email: String,
    status: Option<bool>,
    isadmin: Option<bool>,
}

impl User {
    pub fn all() -> QueryResult<Vec<UserSlim>> {
        let mut conn = db::get_conn();
        users
            .select((id, username, email, status, isadmin))
            .load::<UserSlim>(&mut conn)
    }

    pub fn find_by_id(user_id: i32) -> QueryResult<User> {
        let mut conn = db::get_conn();
        users.find(user_id).first::<User>(&mut conn)
    }

    pub fn signup(new_user: UserDTO) -> Result<String, String> {
        let mut conn = db::get_conn();
        if Self::find_user_by_username(&new_user.username).is_err() {
            let new_user = UserDTO {
                password: hash(&new_user.password, DEFAULT_COST).unwrap(),
                ..new_user
            };
            let _ = diesel::insert_into(users)
                .values(new_user)
                .execute(&mut conn);
            Ok(constants::MESSAGE_SIGNUP_SUCCESS.to_string())
        } else {
            Err(format!(
                "User '{}' is already registered",
                &new_user.username
            ))
        }
    }

    pub fn login(login: LoginDTO) -> Option<LoginInfoDTO> {
        let mut conn = db::get_conn();
        if let Ok(user_to_verify) = users
            .filter(username.eq(&login.username))
            .get_result::<User>(&mut conn)
        {
            if !user_to_verify.password.is_empty()
                && verify(&login.password, &user_to_verify.password).unwrap()
            {
                return Some(LoginInfoDTO {
                    username: user_to_verify.username,
                });
            }
        }

        None
    }

    pub fn change_password(
        user_id: i32,
        change_password: ChangePasswordDTO,
    ) -> Result<String, String> {
        let mut conn = db::get_conn();
        if let Ok(mut user) = users.find(user_id).get_result::<User>(&mut conn) {
            if verify(&change_password.old_password, &user.password).unwrap() {
                user.password = hash(&change_password.new_password, DEFAULT_COST).unwrap();
                let _ = diesel::update(users.find(user_id))
                    .set(&user)
                    .execute(&mut conn);
                return Ok(constants::MESSAGE_CHANGE_PASSWORD_SUCCESS.to_string());
            }
        }

        Err(constants::MESSAGE_CHANGE_PASSWORD_FAILED.to_string())
    }

    pub fn find_user_by_username(un: &str) -> QueryResult<User> {
        let mut conn = db::get_conn();
        users.filter(username.eq(un)).get_result::<User>(&mut conn)
    }

    pub fn default() -> User {
        User {
            id: 0,
            username: "".to_string(),
            email: "".to_string(),
            password: "".to_string(),
            status: None,
            isadmin: None,
        }
    }
}
