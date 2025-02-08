use std::num::NonZero;

use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};
use jwt::FromBase64;
use serde::{Deserialize, Deserializer};

fn hash_pass(ser: &mut String) {
    let salt = std::env::var("JWT_SALT").unwrap();

    let mut digest = [0u8; ring::digest::SHA256_OUTPUT_LEN];

    let key = ring::pbkdf2::derive(
        ring::pbkdf2::PBKDF2_HMAC_SHA256,
        NonZero::new(100).unwrap(),
        salt.as_bytes(),
        ser.as_bytes(),
        &mut digest,
    );

    use jwt::ToBase64;

    let n = digest.to_base64().unwrap().to_string();
    *ser = n;
}

pub struct EmailPassword {
    pub email: String,
    pub password: String,
    #[allow(unused)]
    private_to_construct: ()
}

impl<'d> Deserialize<'d> for EmailPassword {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'d>,
    {
        let mut basic =
            EmailPasswordUnsafe::deserialize(deserializer)?;

        hash_pass(&mut basic.password);

        Ok(EmailPassword {
            email: basic.email,
            password: basic.password,
            private_to_construct: (),
        })
    }
}

#[derive(Deserialize)]
struct EmailPasswordUnsafe {
    email: String,
    password: String,
}

/// take it as Basic token
impl<S: Sync> FromRequestParts<S> for EmailPassword {
    type Rejection = String;

    async fn from_request_parts(
        parts: &mut Parts,
        _: &S,
    ) -> Result<Self, Self::Rejection> {
        let basic = parts
            .headers
            .get(AUTHORIZATION)
            .ok_or("authorization header does exist")?;
        let basic = basic.to_str().map_err(|e| e.to_string())?;
        let basic =
            basic.strip_prefix("Basic ").ok_or("basic token")?;
        // from_base64 uses Deserialize, which hash the pass
        let basic = EmailPassword::from_base64(basic)
            .map_err(|e| e.to_string())?;

        Ok(basic)
    }
}
