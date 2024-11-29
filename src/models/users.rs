use diesel::{query_dsl::methods::SelectDsl, AsChangeset, Insertable, QueryResult, Queryable, Selectable};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use serde::{Deserialize, Serialize};

use crate::schema::users::{self, dsl::*};

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
    pub async fn all(conn: &mut AsyncPgConnection) -> QueryResult<Vec<UserSlim>> {
            users
                .select((id, username, email, status, isadmin))
                .load::<UserSlim>(conn)
                .await
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
