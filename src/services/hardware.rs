use crate::{error::ServiceError, models::hardwares::Hardware};

pub async fn get_all_hardwares() -> Result<Vec<Hardware>, ServiceError> {
    match Hardware::all() {
        Ok(hardwares) => Ok(hardwares),
        Err(_) => Err(ServiceError::InternalServerError {
            error_message: "Error loading hardwares".to_string(),
        }),
    }
}
