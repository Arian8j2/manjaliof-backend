use rocket::{request::{FromRequest, Outcome, Request}, http::Status};
use async_trait::async_trait;
use std::env;

lazy_static! {
    static ref MANJALIOF_TOKEN: String = env::var("MANJALIOF_BACKEND_TOKEN")
        .expect("environemnt variable 'MANJALIOF_BACKEND_TOKEN' is not set");
}

#[derive(Clone)]
pub struct Token;

#[async_trait]
impl<'r> FromRequest<'r> for Token {
    type Error = &'static str;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let maybe_token = request.headers().get_one("auth_token");
        match maybe_token {
            Some(token) => {
                if token == MANJALIOF_TOKEN.to_string() {
                    Outcome::Success(Token)
                } else {
                    Outcome::Failure((Status::Unauthorized, "unauthorized token"))
                }
            },
            None => Outcome::Failure((Status::Unauthorized, "no token"))
        }
    }
}

