use ntex::web::HttpResponse;

use crate::{
    constants, error::ServiceError, models::response::ResponseBody,
    services::hardware::get_all_hardwares,
};

pub async fn get_all_hardwares_controller() -> Result<HttpResponse, ServiceError> {
    match get_all_hardwares().await {
        Ok(hardwares) => {
            Ok(HttpResponse::Ok().json(&ResponseBody::new(constants::MESSAGE_OK, hardwares)))
        }
        Err(err) => Err(err),
    }
}
