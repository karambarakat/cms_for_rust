use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub struct Action(pub &'static str);

#[derive(Debug, Serialize)]
pub struct UserError {
    pub code: String,
    pub user_hint: String,
    /// in forms it's useful to assign error to each field
    pub structured_hint: Option<HashMap<String, String>>,
    /// list of actions that the server suggest the user
    /// to take to resolve the error, in other words,
    /// the frontend dev doesn't have have to come up with them
    pub server_suggest: Option<Vec<Action>>,
}

/// these are errors that have 4xx status codes, refer to
/// https://developer.mozilla.org/en-US/docs/Web/HTTP/Status
///
/// some errors are the fault of the server (like bugs), these are 5xx
/// please consider panicing on them, and they will be handled
/// by axum
///
/// some errors are the user's fault, and the frontend dev
/// is responsible to create the appropriate UI for them
/// and display them to the user with the appropriate action
/// these are `#.user_error = Some(...)`
#[derive(Debug)]
pub struct ClientError {
    pub status_code: StatusCode,
    // if its the frontend's responsiblity, they should have
    // an english readable message in the console for help
    pub dev_hint: String,
    pub user_error: Option<UserError>,
}

pub struct PanicError;
impl IntoResponse for PanicError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, "server paniced")
            .into_response()
    }
}

impl ClientError {
    pub fn add_user_error(
        mut self,
        code: &str,
        info: &str,
        opt: impl FnOnce(&mut UserError) -> (),
    ) -> Self {
        let mut user_error = UserError {
            code: code.to_string(),
            user_hint: info.to_string(),
            structured_hint: None,
            server_suggest: None,
        };
        opt(&mut user_error);
        self.user_error = Some(user_error);
        self
    }
}

impl<T: Into<ClientError>> From<(StatusCode, T)>
    for ClientError
{
    fn from(value: (StatusCode, T)) -> Self {
        let this: ClientError = value.1.into();
        ClientError {
            status_code: value.0,
            dev_hint: this.dev_hint,
            user_error: this.user_error,
        }
    }
}

impl From<&'static str> for ClientError {
    fn from(value: &'static str) -> Self {
        ClientError {
            status_code: StatusCode::BAD_REQUEST,
            dev_hint: value.to_string(),
            user_error: None,
        }
    }
}

impl From<String> for ClientError {
    fn from(value: String) -> Self {
        ClientError {
            status_code: StatusCode::BAD_REQUEST,
            dev_hint: value,
            user_error: None,
        }
    }
}

impl IntoResponse for ClientError {
    fn into_response(self) -> axum::response::Response {
        if self.status_code.is_server_error() {
            panic!("Server errors are not meant to be handled by 'ClientError': {:?}", self);
        }

        let hint = self.dev_hint;

        let hint = if let Some(info) =
            self.status_code.canonical_reason()
        {
            format!("{info}: {hint}")
        } else {
            hint
        };

        let user_error = self.user_error;

        return (
            self.status_code,
            Json(json!({
                "error": {
                    "hint": hint,
                    "user_error": user_error,
                },
            })),
        )
            .into_response();
    }
}

pub mod insert {
    use axum::{http::StatusCode, response::IntoResponse, Json};
    use serde_json::json;

    use crate::error::ErrorInternal;

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    pub struct InsertError(StatusCode, InsertErrorInternal);
    impl InsertError {
        pub fn to_refactor(
            code: StatusCode,
            for_dev: &str,
        ) -> Self {
            InsertError(
                code,
                InsertErrorInternal::Other(for_dev.to_string()),
            )
        }
    }

    impl IntoResponse for InsertError {
        fn into_response(self) -> axum::response::Response {
            let mut body = json!({
                "status": self.0.as_u16(),
                "error": self.0.canonical_reason().unwrap_or_default(),
            });

            if let InsertErrorInternal::Other(o) = self.1 {
                body.as_object_mut()
                    .unwrap()
                    .insert("for_dev".to_string(), o.into());
            }

            (self.0, Json(body)).into_response()
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    pub enum InsertErrorInternal {
        ForiegnKeyViolation,
        Other(String),
        Unkown,
    }

    impl From<super::GlobalError> for InsertError {
        fn from(value: super::GlobalError) -> Self {
            let int = match value.1 {
                ErrorInternal::MoreInfo(info) => {
                    InsertErrorInternal::Other(info)
                }
                _ => InsertErrorInternal::Unkown,
            };
            Self(value.0, int)
        }
    }
}

#[derive(Debug, Default, Serialize)]
enum ErrorInternal {
    #[default]
    Unkown,
    EntryNotFound(String),
    MoreInfo(String),
}

#[derive(Debug)]
pub struct GlobalError(StatusCode, ErrorInternal);

impl From<String> for GlobalError {
    fn from(value: String) -> Self {
        GlobalError(
            StatusCode::BAD_REQUEST,
            ErrorInternal::MoreInfo(value),
        )
    }
}

impl GlobalError {
    pub fn info(mut self, msg: &str) -> Self {
        *&mut self.1 = ErrorInternal::MoreInfo(msg.to_owned());
        self
    }
}

impl IntoResponse for GlobalError {
    fn into_response(self) -> axum::response::Response {
        let mut body = json!({
            "status": self.0.as_u16(),
            "error": self.0.canonical_reason().unwrap_or_default(),
        });

        if let ErrorInternal::MoreInfo(o) = self.1 {
            body.as_object_mut()
                .unwrap()
                .insert("info".to_string(), o.into());
        }

        (self.0, Json(body)).into_response()
    }
}

pub fn entry_not_found(entry: &str) -> GlobalError {
    GlobalError(
        StatusCode::NOT_FOUND,
        ErrorInternal::EntryNotFound(entry.to_string()),
    )
}

pub fn to_refactor(info: &str) -> GlobalError {
    GlobalError(
        StatusCode::BAD_REQUEST,
        ErrorInternal::MoreInfo(info.to_string()),
    )
}

pub fn not_found(id: i32) -> GlobalError {
    GlobalError(StatusCode::NOT_FOUND, ErrorInternal::default())
}

pub fn missing_id_in_query() -> GlobalError {
    GlobalError(
        StatusCode::BAD_REQUEST,
        ErrorInternal::default(),
    )
}
