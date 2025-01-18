use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use serde_json::json;

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
