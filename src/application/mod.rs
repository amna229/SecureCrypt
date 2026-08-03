use std::error::Error;
use tokio::net::TcpListener;

pub mod handlers;
pub mod routes;
pub mod http;


// pub async fn run_application() -> Result<(), Box<dyn Error + Send + Sync>> {

//     let app = routes::create_router();

//     let listener = TcpListener::bind("127.0.0.1:8080").await?;

//     println!("Application at http://127.0.0.1:8080");

//     axum::serve(listener, app).await?;

//     Ok(())
// }