use backend::app;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("128.0.0.1:0")
        .await
        .expect("failed to bind to port 0");
    axum::serve(listener, app())
        .await
        .expect("failed to serve axum server");
    Ok(())
}
