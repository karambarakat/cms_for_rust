use axum::http::StatusCode;
use chrono::DateTime;
use chrono::Utc;
use hmac::Hmac;
use hmac::Mac;
use jwt::SignWithKey;
use jwt::VerifyWithKey;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

use crate::error::ClientError;
use crate::error::UserError;

#[derive(Clone, Deserialize, Serialize)]
pub struct IClaims {
    pub id: String,
    pub iat: i64,
    pub exp: i64,
    #[serde(flatten)]
    pub todos: HashMap<String, serde_json::Value>,
}

fn sign<S: Serialize>(data: S) -> String {
    let env =
        std::env::var("JWT_SALT").expect("JWT_SALT not set");

    let key =
        Hmac::<sha2::Sha256>::new_from_slice(env.as_bytes())
            .expect("HMAC can take key of any size");

    data.sign_with_key(&key).unwrap()
}

pub fn sign_and(
    id: &str,
    until: chrono::TimeDelta,
    todos: HashMap<String, serde_json::Value>,
) -> String {
    let iat = Utc::now();
    let exp = iat + until;

    let map = serde_json::to_value(IClaims {
        id: id.to_owned(),
        iat: iat.timestamp(),
        exp: exp.timestamp(),
        todos,
    })
    .unwrap();

    sign(map)
}

pub fn sign_for(id: &str, until: chrono::TimeDelta) -> String {
    let iat = Utc::now();
    let exp = iat + until;

    sign(IClaims {
        id: id.to_owned(),
        iat: iat.timestamp(),
        exp: exp.timestamp(),
        todos: HashMap::new(),
    })
}

fn verify_or(data: &str) -> Option<()> {
    let env =
        std::env::var("JWT_SALT").expect("JWT_SALT not set");

    let key =
        Hmac::<sha2::Sha256>::new_from_slice(env.as_bytes())
            .expect("HMAC can take key of any size");

    let _: IClaims = data.verify_with_key(&key).ok()?;

    Some(())
}

pub enum TokenError {
    TokenExpired,
    TokenInvalid,
}

impl From<TokenError> for ClientError {
    fn from(value: TokenError) -> Self {
        match value {
            TokenError::TokenExpired => ClientError {
                status_code: StatusCode::UNAUTHORIZED,
                dev_hint: "token is valid but has expired"
                    .to_owned(),
                user_error: Some(UserError {
                    code: "token_expired".to_owned(),
                    user_hint: "session has expired".to_owned(),
                    structured_hint: None,
                    server_suggest: None,
                }),
            },
            TokenError::TokenInvalid => ClientError {
                status_code: StatusCode::UNAUTHORIZED,
                dev_hint: "invalid signature".to_owned(),
                user_error: None,
            },
        }
    }
}

pub fn verify_exp(data: &str) -> Result<IClaims, TokenError> {
    let env =
        std::env::var("JWT_SALT").expect("JWT_SALT not set");

    let key =
        Hmac::<sha2::Sha256>::new_from_slice(env.as_bytes())
            .expect("HMAC can take key of any size");

    let claims: IClaims = data
        .verify_with_key(&key)
        .map_err(|_| TokenError::TokenInvalid)?;

    let exp = DateTime::from_timestamp(claims.exp, 0).unwrap();
    let now = Utc::now();

    if exp < now {
        Err(TokenError::TokenExpired)?;
    }

    Ok(claims)
}
