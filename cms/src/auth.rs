use std::{
    collections::{BTreeMap, HashMap},
    fmt,
    num::NonZero,
};

use axum::{
    body::{Body, HttpBody},
    extract::{FromRequestParts, Request, State},
    http::{
        header::AUTHORIZATION, request::Parts, HeaderMap,
        HeaderValue, Response, StatusCode,
    },
    middleware::{from_fn, Next},
    response::IntoResponse,
    routing::{get, post},
    Extension, Json, Router,
};
use chrono::Utc;
use hmac::{Hmac, Mac};
use ijwt::IClaims;
use jwt::{FromBase64, SignWithKey};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::json;
use sqlx::{
    database::HasArguments, ColumnIndex, Database, Decode,
    Executor, IntoArguments, Pool, Sqlite,
};

use crate::traits::{Collection, Update};

#[derive(Serialize, Deserialize)]
pub struct SuperUser {
    pub user_name: Option<String>,
    pub email: Option<String>,
    pub password: Option<Password>,
}

pub fn migration_st<S: Database>() -> &'static str {
    r#"
    CREATE TABLE IF NOT EXISTS _super_users (
        id SERIAL,
        user_name TEXT NOT NULL,
        email TEXT,
        password TEXT,
        PRIMARY KEY (user_name, email)
    );
    "#
}

pub async fn cold_start_no_super_users<DB: Database, E>(exec: &E)
where
    DB: Database,
    i32: sqlx::Type<DB> + for<'r> Decode<'r, DB>,
    for<'c> &'c E: Executor<'c, Database = DB>,
    for<'a> <DB as HasArguments<'a>>::Arguments:
        IntoArguments<'a, DB>,
    usize: ColumnIndex<<DB as Database>::Row>,
{
    let res: (i32,) =
        sqlx::query_as("SELECT COUNT(*) FROM _super_user")
            .fetch_one(exec)
            .await
            .expect("Failed to check for super users");

    if res.0 == 0 {
        let id: (i32,) = sqlx::query_as(
            "INSERT INTO _super_user () VALUES () RETURNING id",
        )
        .fetch_one(exec)
        .await
        .expect("Failed to insert super user");

        let token = ijwt::sign_for(
            &id.0.to_string(),
            chrono::Duration::minutes(30),
        );

        println!("Super user token: {}", token);
    }
}

pub struct AuthError;

impl<T: fmt::Debug> From<T> for AuthError {
    fn from(value: T) -> Self {
        AuthError
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::BAD_REQUEST, ()).into_response()
    }
}

pub async fn create_super_user_if_not_exist(
    p: Pool<Sqlite>,
) -> Option<String> {
    sqlx::query(migration_st::<Sqlite>())
        .execute(&p)
        .await
        .unwrap();

    let count: (i32,) = sqlx::query_as(
        "
SELECT Count(*) FROM _super_users;
        ",
    )
    .fetch_one(&p)
    .await
    .unwrap();

    if count.0 == 0 {
        let id: (i32,) = sqlx::query_as(
            "
INSERT INTO _super_users (user_name) VALUES (\"super_admin\") RETURNING (id);
",
        )
        .fetch_one(&p)
        .await
        .unwrap();

        let id = id.0;

        let token = ijwt::sign_and(
            &id.to_string(),
            chrono::Duration::minutes(30),
            vec![(
                "privilege".to_string(),
                json!("init_application"),
            )]
            .into_iter()
            .collect(),
        );

        return Some(token);
    }

    return None;
}

pub fn auth_router() -> Router<Pool<Sqlite>> {
    Router::new()
        .route(
            "/init_set_up_first",
            post(init_set_up_first)
                .route_layer(from_fn(need_super_user)),
        )
        .route(
            "/set_up_invited",
            post(set_up_invited)
                .route_layer(from_fn(need_super_user)),
        )
        .route(
            "/login",
            post(login)
                .route_layer(from_fn(extend_for_authinticated)),
        )
}

impl EmailPassword {
    fn new(mut uns: EmailPasswordUnsafe) -> Self {
        hash_pass(&mut uns.password);

        EmailPassword {
            email: uns.email,
            password: uns.password,
        }
    }
}
fn hash_pass(ser: &mut String) {
    let salt = std::env::var("SALT").unwrap();

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
    email: String,
    password: String,
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
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let basic = parts
            .headers
            .get(AUTHORIZATION)
            .ok_or("authorization header does exist")?;
        let basic = basic.to_str().map_err(|e| e.to_string())?;
        let basic =
            basic.strip_prefix("Basic ").ok_or("basic token")?;
        let mut basic = EmailPassword::from_base64(basic)
            .map_err(|e| e.to_string())?;

        Ok(basic)
    }
}

#[derive(Deserialize)]
pub struct SetupFirstUser {
    user_name: String,
    #[serde(flatten)]
    email_password: EmailPassword,
}

#[axum::debug_handler]
pub async fn init_set_up_first(
    user: Extension<IClaims>, // authenticated
    db: State<Pool<Sqlite>>,
    body: Json<SetupFirstUser>,
) -> Result<(), ()> {
    if user.0.todos.get("privilege")
        != Some(&json!("init_application"))
    {
        return Err(());
    }

    sqlx::query(
        "
    INSERT (user_name, email, password) 
    VALUES ($1, $2, $3)
    INTO _super_user WHERE id = $4",
    )
    .bind(body.0.user_name)
    .bind(body.0.email_password.email)
    .bind(body.0.email_password.password)
    .bind(user.0.id)
    .fetch_one(&db.0)
    .await
    .unwrap();

    Ok(())
}

#[axum::debug_handler]
pub async fn set_up_invited(
    user: Extension<IClaims>, // authenticated
    db: State<Pool<Sqlite>>,
    // safe: password was already hashed
    basic_safe: EmailPassword,
    body: Json<SetupFirstUser>,
) -> Result<(), ()> {
    sqlx::query(
        "
    INSERT (user_name, email, password) 
    VALUES ($1, $2, $3)
    INTO _super_user WHERE id = $4",
    )
    .bind(body.0.user_name)
    .bind(basic_safe.email)
    .bind(basic_safe.password)
    .bind(user.0.id)
    .fetch_one(&db.0)
    .await
    .unwrap();

    Ok(())
}

#[axum::debug_handler]
pub async fn login(
    db: State<Pool<Sqlite>>,
    mut may_extend: Extension<MayExtend>,
    // safe: password was already hashed
    basic_safe: EmailPassword,
) -> Result<(), ()> {
    let id: (i32,) = sqlx::query_as(
        "
SELECT id FROM _super_user WHERE passsword = $1 AND email = $2",
    )
    .bind(basic_safe.password)
    .bind(basic_safe.email)
    .fetch_one(&db.0)
    .await
    .unwrap();

    may_extend.0.extend_for(&id.0.to_string());

    Ok(())
}

#[derive(Clone)]
pub struct MayExtend(Option<String>);
impl MayExtend {
    pub fn extend_for(&mut self, id: &str) {
        self.0 = Some(id.to_string());
    }
}

#[axum::debug_middleware]
pub async fn extend_for_authinticated(
    mut req: Request<Body>,
    next: axum::middleware::Next,
) -> Result<Response<Body>, String> {
    let may_extend = MayExtend(None);
    req.extensions_mut().insert(may_extend);

    let mut res = next.run(req).await;

    let may_extend =
        res.extensions().get::<MayExtend>().unwrap();
    if let Some(id) = &may_extend.0 {
        let token =
            ijwt::sign_for(id, chrono::Duration::days(1));

        res.headers_mut().append(
            "X-token",
            HeaderValue::from_str(&token).unwrap(),
        );
    }

    Ok(res)
}

#[axum::debug_middleware]
pub async fn need_super_user(
    headers: HeaderMap,
    mut req: Request<Body>,
    next: axum::middleware::Next,
) -> Result<Response<Body>, AuthError> {
    let bearer = headers
        .get(AUTHORIZATION)
        .ok_or("no auth found")?
        .to_str()?;

    let bearer = bearer
        .strip_prefix("Bearer ")
        .ok_or("should start with Bearer")?;

    let map = ijwt::verify_exp(bearer)?;

    let id = map.id.clone();

    req.extensions_mut().insert(map);

    let mut res = next.run(req).await;

    // extend for authiticated users
    let token = ijwt::sign_for(
        &id.to_string(),
        chrono::Duration::days(1),
    );

    // I don't want to use cookies for potentioal misuse
    res.headers_mut().append(
        "X-token",
        HeaderValue::from_str(&token).unwrap(),
    );

    Ok(res)
}

mod ijwt {
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
}

#[derive(Deserialize, Serialize)]
pub struct Password {
    #[serde(skip_serializing)]
    password: String,
}
