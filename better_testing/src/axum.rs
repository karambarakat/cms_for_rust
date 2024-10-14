use axum::{
    body::Body,
    http::{HeaderMap, Response, StatusCode},
    response::IntoResponse,
    Json, Router,
};
use http::request::Builder;

use crate::Similar;

pub struct Invoking<'r, S> {
    app: &'r Router<S>,
    status: Option<StatusCode>,
    body: Option<Response<Body>>,
    headers: HeaderMap,
}

impl<'r, S> Invoking<'r, S> {
    pub fn json(
        self,
        body: serde_json::Value,
    ) -> Invoking<'r, S> {
        Invoking {
            body: Some(Json(body).into_response()),
            ..self
        }
    }
    pub fn body<BodyParam>(
        self,
        body: BodyParam,
    ) -> Invoking<'r, S>
    where
        BodyParam: IntoResponse,
    {
        Invoking {
            body: Some(body.into_response()),
            ..self
        }
    }
    pub fn status(self, status: StatusCode) -> Invoking<'r, S> {
        Invoking {
            status: Some(status),
            ..self
        }
    }
}

pub fn invoking<'r, S>(app: &'r Router<S>) -> Invoking<'r, S> {
    Invoking {
        app,
        status: None,
        body: None,
        headers: HeaderMap::new(),
    }
}

pub struct With<W> {
    pub builder: W,
}

impl<S> Similar<With<Result<Builder, http::Error>>>
    for Invoking<'_, S>
{
    fn similar(
        &self,
        requst: With<Result<Builder, http::Error>>,
    ) -> crate::SimilarResult {
        let builder = match requst.builder {
            Ok(builder) => builder,
            Err(err) => return Err(err.to_string()),
        };
        <Invoking<'_, S> as Similar<With<Builder>>>::similar(
            self,
            With { builder },
        )
    }
}
impl<S> Similar<With<Builder>> for Invoking<'_, S> {
    fn similar(
        &self,
        _requst: With<Builder>,
    ) -> crate::SimilarResult {
        // let request = requst.builder
        Ok(())
    }
}
