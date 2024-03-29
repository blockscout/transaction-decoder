mod bytes;
mod handlers;
mod types;

use crate::types::{Response, Transaction};

pub use crate::{bytes::Bytes as DisplayBytes, types::Request};
pub use handlers::decode;

use actix_web::{web, App, HttpServer};
pub async fn run() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/", web::post().to(decode)))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await?;
    Ok(())
}
