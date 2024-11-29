use crate::{
    constants, db, error::ServiceError, models::response::ResponseBody,
    services::users::get_all_users,
};
use ntex::web::{self, HttpResponse};

pub async fn get_all_users_controller(
    pool: web::types::State<db::PgPool>,
) -> Result<HttpResponse, ServiceError> {
    match get_all_users(&pool).await {
        Ok(users) => Ok(HttpResponse::Ok().json(&ResponseBody::new(constants::MESSAGE_OK, users))),
        Err(err) => Err(err),
    }
}
