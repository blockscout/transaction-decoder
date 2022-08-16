mod bytes;
mod error;
mod handlers;
mod types;

use crate::{
    error::Error,
    types::{
        AbiResponse, EventRequest, EventResponse, Response, ResponseMethod, Transaction, TxLog,
    },
};

pub use crate::{bytes::Bytes as DisplayBytes, types::Request};
pub use handlers::{decode, decode_events};

use actix_web::{web, App, HttpServer};
pub async fn run() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::post().to(decode))
            .route("/events", web::post().to(decode_events))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;
    Ok(())
}
