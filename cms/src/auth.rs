#![allow(unused)]
use std::task::{Context, Poll};

use axum::{
    extract::{FromRequestParts, Request},
    http::{request::Parts, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use entity::User;
use futures_util::future::BoxFuture;
use tower::{Layer, Service};
use tracing::error;

mod entity {
    use inventory::submit;
    use queries_for_sqlx::{IntoMutArguments, SupportNamedBind};
    use serde::{Deserialize, Serialize};
    use sqlx::{
        prelude::{FromRow, Type},
        Arguments, ColumnIndex, Database, Decode, Encode, Row,
        Sqlite,
    };
    use std::marker::PhantomData;

    use crate::{
        entities::{Entity, EntityPhantom},
        queries_for_sqlx_extention::{
            col_type_check_if_null, primary_key, SqlxQuery,
        },
    };

    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        Serialize,
        Deserialize,
        FromRow,
    )]
    pub struct User {
        pub user_name: String,
        pub email: String,
        pub password: Option<String>,
    }

    impl<S> Entity<S> for User
    where
        S: SupportNamedBind + Sync + SqlxQuery,
        for<'s> &'s str: ColumnIndex<S::Row>,
        String: Type<S> + for<'d> Decode<'d, S>,
    {
        type Partial = User;

        fn migrate<'q>(
            stmt: &mut queries_for_sqlx::prelude::stmt::CreateTableSt<S, queries_for_sqlx::quick_query::QuickQuery<'q>>,
        ) {
            stmt.column(
                "user_name",
                (
                    col_type_check_if_null::<S::KeyType>(),
                    primary_key(),
                ),
            );
        }

        fn table_name() -> &'static str {
            "User"
        }

        fn from_row(
            row: &<S as sqlx::Database>::Row,
        ) -> Result<Self, sqlx::Error> {
            Ok(Self {
                user_name: row.try_get("user_name")?,
                password: row.try_get("password")?,
                email: row.try_get("email")?,
            })
        }

        fn from_row_scoped(
            row: &<S as sqlx::Database>::Row,
        ) -> Result<Self, sqlx::Error> {
            Ok(Self {
                user_name: row.try_get("user_user_name")?,
                password: row.try_get("user_password")?,
                email: row.try_get("user_email")?,
            })
        }

        fn members_scoped() -> Vec<&'static str> {

            vec![
                "user_user_name",
                "user_password",
                "user_email"]
        }

        fn members() -> Vec<&'static str> {
            vec!["user_name", "password", "email"]
        }
    }

    submit!(crate::entities::DynEntitySubmitable::<Sqlite> {
        object: || {
            Box::new(EntityPhantom::<User>(PhantomData))
        }
    });

    submit!(crate::migration::Submitable::<Sqlite> {
        object: || {
            Box::new(EntityPhantom::<User>(PhantomData))
        }
    });

    impl<'q, S> IntoMutArguments<'q, S> for User
    where
        S: Database,
        String: Type<S> + Encode<'q, S> + Send + 'q,
    {
        fn into_arguments(
            self,
            argument: &mut <S as sqlx::database::HasArguments<
                'q,
            >>::Arguments,
        ) {
            argument.add(self.user_name);
            argument.add(self.password);
            argument.add(self.email);
        }
    }
}

fn from_part(parts: &HeaderMap) -> Result<User, Response> {
    let auth = parts
        .get("authorization")
        .ok_or(StatusCode::UNAUTHORIZED.into_response())?
        .to_str()
        .map_err(|_| StatusCode::UNAUTHORIZED.into_response())?
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED.into_response())?
        .to_string();

    Ok(todo!())
}

#[async_trait::async_trait]
impl<S> FromRequestParts<S> for User {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        _: &S,
    ) -> Result<Self, Self::Rejection> {
        let auth = if let Some(e) =
            parts.extensions.get::<User>()
        {
            return Ok(e.clone());
        } else {
            error!("AuthLayer is not used, the client will not get new token");
            return Ok(from_part(&parts.headers)?);
        };
    }
}

#[derive(Debug, Clone)]
pub struct AuthLayer;

#[derive(Debug, Clone)]
pub struct AuthLayerS<S> {
    inner: S,
}

impl<S> Layer<S> for AuthLayer {
    type Service = AuthLayerS<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthLayerS { inner }
    }
}

impl<S> Service<Request> for AuthLayerS<S>
where
    S: Service<Request, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future =
        BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut request: Request) -> Self::Future {
        let token = from_part(request.headers());

        if let Ok(token) = token {
            request.extensions_mut().insert::<User>(token);
            let future = self.inner.call(request);
            Box::pin(async move {
                let mut response: Response = future.await?;

                response
                    .headers_mut()
                    .insert("X-token", "".parse().unwrap());

                Ok(response)
            })
        } else {
            Box::pin(async move {
                Ok((StatusCode::UNAUTHORIZED, "")
                    .into_response())
            })
        }
    }
}

pub fn layer() -> AuthLayer {
    AuthLayer
}
