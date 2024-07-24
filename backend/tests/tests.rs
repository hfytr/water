use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use backend::app;
use tower::ServiceExt;

#[tokio::test]
async fn health_check_works() {
    assert_eq!(
        app()
            .await
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/")
                    .body(Body::empty())
                    .expect("failed to build request")
            )
            .await
            .expect("failed to send request")
            .status(),
        StatusCode::OK
    );
}
