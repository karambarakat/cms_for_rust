use std::fmt;

use super::email_password::EmailPassword;
use super::ijwt::IClaims;
use axum::{
    body::Body,
    extract::{Request, State},
    http::{
        header::AUTHORIZATION, HeaderMap, HeaderValue, Response,
    },
    Extension, Json,
};
use serde::Deserialize;
use serde_json::json;
use sqlx::{
    database::HasArguments, ColumnIndex, Database, Decode,
    Executor, IntoArguments, Pool, Sqlite,
};

use crate::{auth::ijwt, error::ClientError};

pub async fn init_auth(db: Pool<Sqlite>) -> Result<(), String> {
    std::env::var("JWT_SALT").map_err(|_| "JWT_SALT not set")?;

    if let Some(token) = create_super_user_if_not_exist(db).await
    {
        let be = "http://localhost:3000";
        let fe = "http://localhost:5173";
        println!("Looks like you have no super user");
        print!("Create your first at ");
        println!(
            "{fe}/auth/init?token={token}&backend_url={be}"
        );
        println!(
            "Or initiate different database at the same page"
        );
    }

    Ok(())
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

pub struct AuthError(String);

impl<T: fmt::Debug> From<T> for AuthError {
    fn from(value: T) -> Self {
        AuthError(format!("{value:?}"))
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

#[derive(Deserialize)]
pub struct SetupFirstUser {
    user_name: String,
    #[serde(flatten)]
    email_password: EmailPassword,
}

#[axum::debug_middleware]
pub async fn can_init(
    user: Extension<IClaims>, // authenticated
    req: Request<Body>,
    next: axum::middleware::Next,
) -> Result<Response<Body>, ClientError> {
    if user.0.todos.get("privilege")
        != Some(&json!("init_application"))
    {
        Err("under privileged")?;
    }

    Ok(next.run(req).await)
}

#[axum::debug_handler]
pub async fn sign_in_existing(
    user: Extension<IClaims>, // authenticated
    db: State<Pool<Sqlite>>,
    body: Json<SetupFirstUser>,
) -> Result<(), ()> {
    sqlx::query(
        "
    UPDATE _super_user SET 
       user_name = $1, 
       email = $2, 
       password = $4 
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

/// check if Bearer token is valid and
/// extend it for one more day
#[axum::debug_middleware]
pub async fn need_super_user(
    headers: HeaderMap,
    mut req: Request<Body>,
    next: axum::middleware::Next,
) -> Result<Response<Body>, ClientError> {
    let bearer = headers
        .get(AUTHORIZATION)
        .ok_or("authorization not found")?
        .to_str()
        .unwrap();

    let bearer = bearer
        .strip_prefix("Bearer ")
        .ok_or("authorization should start with Bearer")?;

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
//
// #[derive(Serialize, Deserialize)]
// pub struct SuperUser {
//     pub user_name: Option<String>,
//     pub email: Option<String>,
//     pub password: Option<Password>,
// }
// #[derive(Deserialize, Serialize)]
// pub struct Password {
//     #[serde(skip_serializing)]
//     password: String,
// }
