use backend::app;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dbg!(std::env::var("DB_USER"));
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("failed to bind to port 0");
    let _ = axum::serve(listener, app().await);
    Ok(())
}
