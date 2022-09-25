use tokio_postgres::{NoTls, Error, Client};


pub async fn create_connection(
    user: &str, password: &str
) -> Result<Client,Error> {
    let config = format!("host=localhost user={} password={}", user, password);

    let (client, connection) = tokio_postgres::connect(
        config.as_str(), NoTls
    ).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    Ok(client)
}
