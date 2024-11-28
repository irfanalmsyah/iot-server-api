use diesel::prelude::*;
use sonic_rs::{Deserialize, Serialize};

use crate::{
    database::run_query_dsl_ext::RunQueryDslExt,
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

#[derive(Insertable, Serialize, Deserialize, Clone)]
#[diesel(table_name = users)]
pub struct RegisterUser {
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
    pub fn all() -> Option<Vec<UserSlim>> {
        match users
            .select((id, username, email, status, isadmin))
            .load_all()
            .optional()
        {
            Ok(Some(response)) => Some(response),
            Ok(None) => Some(Vec::new()),
            Err(_) => None,
        }
    }

    pub fn find_by_id(user_id: i32) -> Option<User> {
        match users.find(user_id).get_first() {
            Ok(response) => Some(response),
            Err(_) => None,
        }
    }

    pub fn find_user_by_username(un: &str) -> Option<User> {
        match users.filter(username.eq(un)).get_first() {
            Ok(response) => Some(response),
            Err(_) => None,
        }
    }

    pub fn insert(new_user: RegisterUser) -> Option<User> {
        match users.insert(&new_user) {
            Ok(response) => Some(response),
            Err(_) => None,
        }
    }

    // pub fn change_password(
    //     user_id: i32,
    //     change_password: ChangePasswordDTO,
    // ) -> Result<String, String> {
    //     if let Ok(mut user) = users.find(user_id).get_result_query::<User>() {
    //         if verify(&change_password.old_password, &user.password).unwrap() {
    //             user.password = hash(&change_password.new_password, DEFAULT_COST).unwrap();
    //             let _ = diesel::update(users.find(user_id))
    //                 .set(&user)
    //                 .execute_query();
    //             return Ok(constants::MESSAGE_CHANGE_PASSWORD_SUCCESS);
    //         }
    //     }

    //     Err(constants::MESSAGE_CHANGE_PASSWORD_FAILED)
    // }

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
