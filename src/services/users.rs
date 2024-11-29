use crate::db;
use crate::error::ServiceError;
use crate::models::users::User;
use crate::models::users::UserSlim;
use ntex::web;

pub async fn get_all_users(
    pool: &web::types::State<db::PgPool>,
) -> Result<Vec<UserSlim>, ServiceError> {
    let mut pool_connection = pool
        .get()
        .await
        .map_err(|e| ServiceError::InternalServerError {
            error_message: format!("Database connection error: {}", e),
        })?;

    match User::all(&mut pool_connection).await {
        Ok(users) => Ok(users),
        Err(_) => Err(ServiceError::InternalServerError {
            error_message: "Error loading users".to_string(),
        }),
    }
}
