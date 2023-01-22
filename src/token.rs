use rocket::{request::{FromRequest, Outcome, Request}, http::Status};
use async_trait::async_trait;
use std::env;

lazy_static! {
    static ref VALID_TOKENS: Vec<Token> = {
        let valid_tokens = env::var("MANJALIOF_VALID_TOKENS")
            .expect("environemnt variable 'MANJALIOF_VALID_TOKENS' is not set");

        let mut result = Vec::new();
        for reffer in valid_tokens.split(";") {
            let chunks: Vec<&str> = reffer.split("=").collect();
            result.push(Token {
                reffer_name: chunks[0].to_string(),
                token: chunks[1].to_string()
            });
        }

        result
    };
}

#[derive(Clone)]
pub struct Token {
    pub reffer_name: String,
    token: String
}

#[async_trait]
impl<'r> FromRequest<'r> for Token {
    type Error = &'static str;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let maybe_token = request.headers().get_one("auth_token");
        match maybe_token {
            Some(token) => {
                for valid_token in VALID_TOKENS.iter() {
                    if valid_token.token == token {
                        return Outcome::Success(valid_token.clone());
                    }
                }
                Outcome::Failure((Status::Unauthorized, "unauthorized token"))
            },
            None => Outcome::Failure((Status::Unauthorized, "no token"))
        }
    }
}

