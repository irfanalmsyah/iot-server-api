use jsonwebtoken::{decode, DecodingKey, Validation};

use crate::models::{jwt::Claims, response::ApiResponse};

pub async fn verify_jwt(token: &str) -> Result<Claims, &'static str> {
    let key = "your-secret-key"; // Replace with your actual secret key
    let validation = Validation::default();
    match decode::<Claims>(token, &DecodingKey::from_secret(key.as_ref()), &validation) {
        Ok(token_data) => Ok(token_data.claims),
        Err(_) => Err("Invalid or expired token"),
    }
}

pub async fn authenticate(token: Option<&str>) -> Result<(), ApiResponse<()>> {
    match token {
        Some(t) => match verify_jwt(t).await {
            Ok(_) => Ok(()),
            Err(err) => {
                let error_response: ApiResponse<()> = ApiResponse {
                    message: err,
                    data: vec![],
                };
                Err(error_response)
            }
        },
        None => {
            let error_response: ApiResponse<()> = ApiResponse {
                message: "Token not provided",
                data: vec![],
            };
            Err(error_response)
        }
    }
}
