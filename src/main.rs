mod utils;

use axum::{routing::post, Router};
use tokio_postgres::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let app = Router::new()
        .route("/user", post(utils::rest::create_user))
        .route("/log", post(utils::rest::record_temperature))
        .route("/fetch", post(utils::rest::fetch_record));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
