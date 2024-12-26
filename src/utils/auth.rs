use jsonwebtoken::{decode, DecodingKey, Validation};
use ntex::http::Request;

use crate::models::{jwt::Claims, response::ApiResponse};

pub async fn verify_jwt(token: &str) -> Result<Claims, &'static str> {
    let key = "your-secret-key"; // Replace with your actual secret key
    let validation = Validation::default();
    match decode::<Claims>(token, &DecodingKey::from_secret(key.as_ref()), &validation) {
        Ok(token_data) => Ok(token_data.claims),
        Err(_) => Err("Invalid or expired token"),
    }
}

pub async fn authenticate(req: &Request) -> Result<Claims, ApiResponse<()>> {
    let token = get_token(req);
    match token {
        Some(t) => match verify_jwt(t).await {
            Ok(claims) => Ok(claims),
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

pub fn get_token(req: &Request) -> Option<&str> {
    let token = req
        .headers()
        .get("Authorization")
        .map(|t| t.to_str().unwrap());
    let token = token.and_then(|t| t.strip_prefix("Bearer "));

    token
}
