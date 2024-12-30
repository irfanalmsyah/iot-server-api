use jsonwebtoken::{decode, errors::ErrorKind, DecodingKey, Validation};
use ntex::http::Request;

use crate::{
    constant::{config, messages},
    models::jwt::Claims,
};

pub async fn verify_jwt(token: &str) -> Result<Claims, &'static str> {
    let validation = Validation::default();
    match decode::<Claims>(
        token,
        &DecodingKey::from_secret(config::JWT_SECRET.as_ref()),
        &validation,
    ) {
        Ok(token_data) => Ok(token_data.claims),
        Err(err) if err == ErrorKind::ExpiredSignature.into() => Err(messages::TOKEN_EXPIRED),
        Err(_) => Err(messages::INVALID_TOKEN),
    }
}

pub async fn authenticate(req: &Request) -> Result<Claims, &'static str> {
    let token = get_token(req);
    match token {
        Some(t) => match verify_jwt(t).await {
            Ok(claims) => Ok(claims),
            Err(err) => Err(err),
        },
        None => Err(messages::INVALID_TOKEN),
    }
}

pub async fn authenticate_admin(req: &Request) -> Result<Claims, &'static str> {
    let claims = authenticate(req).await?;
    if claims.isadmin {
        Ok(claims)
    } else {
        Err(messages::UNAUTHORIZED)
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
