use bcrypt::{hash, verify, DEFAULT_COST};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    constants,
    db::Connection,
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
    pub fn all(conn: &mut PgConnection) -> QueryResult<Vec<UserSlim>> {
        users
            .select((id, username, email, status, isadmin))
            .load::<UserSlim>(conn)
    }

    pub fn find_by_id(conn: &mut Connection, user_id: i32) -> QueryResult<User> {
        users.find(user_id).first::<User>(conn)
    }

    pub fn signup(conn: &mut Connection, new_user: UserDTO) -> Result<String, String> {
        if Self::find_user_by_username(&new_user.username, conn).is_err() {
            let new_user = UserDTO {
                password: hash(&new_user.password, DEFAULT_COST).unwrap(),
                ..new_user
            };
            let _ = diesel::insert_into(users).values(new_user).execute(conn);
            Ok(constants::MESSAGE_SIGNUP_SUCCESS.to_string())
        } else {
            Err(format!(
                "User '{}' is already registered",
                &new_user.username
            ))
        }
    }

    pub fn login(conn: &mut Connection, login: LoginDTO) -> Option<LoginInfoDTO> {
        if let Ok(user_to_verify) = users
            .filter(username.eq(&login.username))
            .get_result::<User>(conn)
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
        conn: &mut Connection,
        user_id: i32,
        change_password: ChangePasswordDTO,
    ) -> Result<String, String> {
        if let Ok(mut user) = users.find(user_id).get_result::<User>(conn) {
            if verify(&change_password.old_password, &user.password).unwrap() {
                user.password = hash(&change_password.new_password, DEFAULT_COST).unwrap();
                let _ = diesel::update(users.find(user_id)).set(&user).execute(conn);
                return Ok(constants::MESSAGE_CHANGE_PASSWORD_SUCCESS.to_string());
            }
        }

        Err(constants::MESSAGE_CHANGE_PASSWORD_FAILED.to_string())
    }

    pub fn find_user_by_username(un: &str, conn: &mut Connection) -> QueryResult<User> {
        users.filter(username.eq(un)).get_result::<User>(conn)
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
