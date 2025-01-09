pub struct TestHandler(Router<()>);
use std::str::from_utf8;

use axum::{
    body::{Body, HttpBody},
    extract::Request,
    handler::Handler,
    http::{self, header, request::Builder, Method, Response},
    routing::get,
    Router,
};
use http_body_util::BodyExt;
use tower::ServiceExt;
impl TestHandler {
    pub fn new<T, Hanlder, Transformer, S>(
        handler: Hanlder,
        transformer: Transformer,
    ) -> TestHandler
    where
        S: Clone + Send + Sync + 'static,
        Hanlder: Handler<T, S>,
        T: 'static,
        Transformer: FnOnce(Router<S>) -> Router<()>,
    {
        let app = Router::new().route("/", get(handler));
        let app = transformer(app);
        TestHandler(app)
    }

    pub async fn test(
        &self,
        body: http::Request<Body>,
        builder: impl FnOnce(Builder) -> Builder,
    ) -> Response<Body> {
        let res = self
            .0
            .clone()
            .oneshot({
                let b = Request::builder()
                    .uri("/")
                    .method(Method::GET);
                let b = builder(b);
                b.body(body).unwrap()
            })
            .await
            .unwrap();

        res
    }

    pub async fn json_test(
        &self,
        body: serde_json::Value,
    ) -> Response<serde_json::Value> {
        let res = self
            .0
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/")
                    .method(Method::GET)
                    .header(
                        header::CONTENT_TYPE,
                        "application/json",
                    )
                    .body(serde_json::to_string(&body).unwrap())
                    .unwrap(),
            )
            .await
            .unwrap();

        let (p, b) = res.into_parts();
        let b = b.collect().await.unwrap();
        let b = b.to_bytes();
        let b = from_utf8(&b).unwrap();
        let b = serde_json::from_str(b).unwrap();
        Response::from_parts(p, b)
    }
}
