use axum::{http::StatusCode, response::IntoResponse, Json};
use case::CaseExt;
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Action(pub &'static str);

#[derive(Debug)]
pub struct UserError {
    pub code: String,
    pub info: String,
    /// in forms it's useful to assign error to each field
    pub structured: Option<HashMap<String, String>>,
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
    pub canonical_reason: StatusCode,
    pub non_canonical_reason: Option<String>,
    pub user_error: Option<UserError>,
}

pub struct CatchAll;
impl IntoResponse for CatchAll {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": {
                    "canonical_reason": "INTERNAL_SERVER_ERROR",
                    "non_canonical_reason": null,
                    "user_error": null,
                },
            })),
        )
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
            info: info.to_string(),
            structured: None,
            server_suggest: None,
        };
        opt(&mut user_error);
        self.user_error = Some(user_error);
        self
    }
}

impl Default for ClientError {
    fn default() -> Self {
        ClientError {
            canonical_reason: StatusCode::BAD_REQUEST,
            non_canonical_reason: None,
            user_error: None,
        }
    }
}

impl From<StatusCode> for ClientError {
    fn from(value: StatusCode) -> Self {
        ClientError {
            canonical_reason: value,
            non_canonical_reason: None,
            user_error: None,
        }
    }
}

impl<T: Into<ClientError>> From<(StatusCode, T)>
    for ClientError
{
    fn from(value: (StatusCode, T)) -> Self {
        let this: ClientError = value.1.into();
        ClientError {
            canonical_reason: value.0,
            non_canonical_reason: this.non_canonical_reason,
            user_error: this.user_error,
        }
    }
}

impl From<&'static str> for ClientError {
    fn from(value: &'static str) -> Self {
        let non_canonical_reason = Some(value.to_snake());
        ClientError {
            canonical_reason: StatusCode::BAD_REQUEST,
            non_canonical_reason,
            user_error: None,
        }
    }
}

impl From<String> for ClientError {
    fn from(value: String) -> Self {
        let non_canonical_reason = Some(value.to_snake());
        ClientError {
            canonical_reason: StatusCode::BAD_REQUEST,
            non_canonical_reason,
            user_error: None,
        }
    }
}

impl IntoResponse for ClientError {
    fn into_response(self) -> axum::response::Response {
        if self.canonical_reason.is_server_error() {
            panic!("Server errors are not meant to be handled by 'ClientError': {:?}", self);
        }

        let canonical_reason = self
            .canonical_reason
            .canonical_reason()
            .unwrap_or("INTERNAL_SERVER_ERROR");

        let non_canonical_reason = self.non_canonical_reason;

        let user_error = ();

        (
            self.canonical_reason,
            Json(json!({
                "error": {
                    "canonical_reason": canonical_reason,
                    "non_canonical_reason": non_canonical_reason,
                    "user_error": user_error,
                },
            })),
        )
            .into_response()
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
