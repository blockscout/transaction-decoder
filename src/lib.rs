mod bytes;
mod handlers;
mod types;

use crate::{
    bytes::Bytes,
    types::{AbiMethod, Request, Response, Transaction},
};

pub use handlers::index;

use actix_web::{web, App, HttpServer};
pub async fn run() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/", web::post().to(index)))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await?;
    Ok(())
}
