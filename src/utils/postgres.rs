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


pub async fn create_user(client: &Client, username: &str) -> Result<i32, Error> {
    let rows = client.query(
        "SELECT id FROM user_list ORDER BY id DESC",
        &[]
    ).await?;

    if rows.len() == 0 {
        let user_id: i32 = 1;

        client.query(
            "INSERT INTO user_list\
                (id,name)\
             VALUES\
                ($1::INTEGER, $2::VARCHAR)",
            &[&user_id, &username]
        ).await?;

        Ok(user_id)
    }
    else {
        let latest: Option<i32> = rows[0].get(0);
        let next_id: i32 = latest.unwrap() + 1;

        client.query(
            "INSERT INTO user_list\
                (id,name)\
            VALUES\
                ($1::INTEGER, $2::VARCHAR)",
            &[&next_id, &username]
        ).await?;

        Ok(next_id)
    }
}
