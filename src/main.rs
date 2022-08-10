#[actix_web::main]
async fn main() -> std::io::Result<()> {
    transaction_decoder::run().await
}
