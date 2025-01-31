
    use chrono::DateTime;
    use chrono::Duration;
    use chrono::Utc;
    use hmac::Hmac;
    use hmac::Mac;
    use jwt::Claims;
    use jwt::FromBase64;
    use jwt::SignWithKey;
    use jwt::ToBase64;
    use jwt::VerifyWithKey;
    use serde::Deserialize;
    use serde::Serialize;
    use std::collections::HashMap;

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
        more_data: HashMap<String, serde_json::Value>,
    ) -> String {
        let iat = Utc::now();
        let exp = iat + until;

        let mut map = serde_json::to_value(IClaims {
            id: id.to_owned(),
            iat: iat.timestamp(),
            exp: exp.timestamp(),
            todos: HashMap::new(),
        })
        .unwrap();

        sign(map)
    }

    pub fn sign_for(
        id: &str,
        until: chrono::TimeDelta,
    ) -> String {
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

    pub fn verify_exp(data: &str) -> Result<IClaims, String> {
        let env =
            std::env::var("JWT_SALT").expect("JWT_SALT not set");

        let key =
            Hmac::<sha2::Sha256>::new_from_slice(env.as_bytes())
                .expect("HMAC can take key of any size");

        let claims: IClaims = data
            .verify_with_key(&key)
            .map_err(|_| "token invalid")?;

        let exp =
            DateTime::from_timestamp(claims.exp, 0).unwrap();
        let now = Utc::now();

        if exp < now {
            return Err("token expired")?;
        }

        Ok(claims)
    }
