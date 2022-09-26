use tokio_postgres::{NoTls, Error, Client};
use chrono::{Local, DateTime};


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

pub async fn record(client: &Client, user_id: i32, temperature: f32) -> Result<(), Error> {
    // log_id (primary key, i64), date(timestamp), user_id(i32), temperature(f32)
    let timestamp: DateTime<Local> = Local::now();

    let rows = client.query(
        "SELECT log_id FROM temperatures ORDER BY log_id DESC",
        &[]
    ).await?;

    let log_id: i64 = if rows.len() == 0 {
        1
    }
    else {
        let tmp: Option<i64> = rows[0].get(0);
        tmp.unwrap() + 1
    };

    client.query(
        "INSERT INTO temperatures\
            (log_id,user_id,date,temperature)\
        VALUES\
            ($1::BIGINT, $2::INTEGER, $3::TIMESTAMP WITH TIME ZONE,$4::REAL)",
        &[&log_id, &user_id, &timestamp, &temperature]
    ).await?;

    Ok(())
}
