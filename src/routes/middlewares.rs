use jsonwebtoken::errors::ErrorKind::ExpiredSignature;
use ntex::service::{Middleware, Service, ServiceCtx};
use ntex::web::{Error, ErrorRenderer, WebRequest, WebResponse, WebResponseError};

use crate::constants::{
    MESSAGE_INVALID_TOKEN, MESSAGE_TOKEN_EXPIRED, MESSAGE_UNAUTHENTICATED, MESSAGE_UNAUTHORIZED,
};
use crate::error::ServiceError::{self, Forbidden, Unauthorized};
use crate::models::token::UserToken;

pub struct UserRequired;
pub struct AdminRequired;

impl<S> Middleware<S> for UserRequired {
    type Service = UserRequiredMiddleware<S>;

    fn create(&self, service: S) -> Self::Service {
        UserRequiredMiddleware { service }
    }
}

impl<S> Middleware<S> for AdminRequired {
    type Service = AdminRequiredMiddleware<S>;

    fn create(&self, service: S) -> Self::Service {
        AdminRequiredMiddleware { service }
    }
}

pub struct UserRequiredMiddleware<S> {
    service: S,
}

pub struct AdminRequiredMiddleware<S> {
    service: S,
}

impl<S, Err> Service<WebRequest<Err>> for UserRequiredMiddleware<S>
where
    S: Service<WebRequest<Err>, Response = WebResponse, Error = Error>,
    Err: ErrorRenderer,
{
    type Response = WebResponse;
    type Error = Error;

    ntex::forward_ready!(service);

    async fn call(
        &self,
        mut req: WebRequest<Err>,
        ctx: ServiceCtx<'_, Self>,
    ) -> Result<Self::Response, Self::Error> {
        if let Err(err) = insert_token_into_request(&mut req) {
            return respond_with_error(req, err);
        }
        let res = ctx.call(&self.service, req).await?;
        Ok(res)
    }
}

impl<S, Err> Service<WebRequest<Err>> for AdminRequiredMiddleware<S>
where
    S: Service<WebRequest<Err>, Response = WebResponse, Error = Error>,
    Err: ErrorRenderer,
{
    type Response = WebResponse;
    type Error = Error;

    ntex::forward_ready!(service);

    async fn call(
        &self,
        mut req: WebRequest<Err>,
        ctx: ServiceCtx<'_, Self>,
    ) -> Result<Self::Response, Self::Error> {
        if let Err(err) = insert_token_into_request(&mut req) {
            return respond_with_error(req, err);
        }

        if !is_admin(&req) {
            return respond_with_error(
                req,
                Forbidden {
                    error_message: MESSAGE_UNAUTHORIZED,
                },
            );
        }
        let res = ctx.call(&self.service, req).await?;
        Ok(res)
    }
}

fn insert_token_into_request<Err>(req: &mut WebRequest<Err>) -> Result<(), ServiceError>
where
    Err: ErrorRenderer,
{
    let authorization_header = req.headers().get("Authorization");
    if authorization_header.is_none() {
        return Err(Unauthorized {
            error_message: MESSAGE_UNAUTHENTICATED,
        });
    }
    let token = authorization_header
        .unwrap()
        .to_str()
        .unwrap()
        .trim_start_matches("Bearer")
        .trim();
    match UserToken::verify_token(token) {
        Ok(user_token) => {
            req.extensions_mut().insert(user_token);
            Ok(())
        }
        Err(err) => {
            let err = match *err.kind() {
                ExpiredSignature => Unauthorized {
                    error_message: MESSAGE_TOKEN_EXPIRED,
                },
                _ => Unauthorized {
                    error_message: MESSAGE_INVALID_TOKEN,
                },
            };

            return Err(err);
        }
    }
}

fn respond_with_error<Err>(req: WebRequest<Err>, err: ServiceError) -> Result<WebResponse, Error>
where
    Err: ErrorRenderer,
{
    let (http_req, _) = req.into_parts();
    let response = err.error_response(&http_req);
    Ok(WebResponse::new(response, http_req))
}

fn is_admin<Err>(req: &WebRequest<Err>) -> bool
where
    Err: ErrorRenderer,
{
    req.extensions()
        .get::<UserToken>()
        .map_or(false, |token| token.isadmin)
}
