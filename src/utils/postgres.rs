use chrono::{DateTime, Local};
use tokio_postgres::{Client, Error, NoTls};

pub async fn create_connection(user: &str, password: &str) -> Result<Client, Error> {
    let config = format!("host=localhost user={} password={}", user, password);

    let (client, connection) = tokio_postgres::connect(config.as_str(), NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    Ok(client)
}

pub async fn create_user(client: &Client, username: &str) -> Result<i32, Error> {
    let rows = client
        .query("SELECT id FROM user_list ORDER BY id DESC", &[])
        .await?;

    if rows.is_empty() {
        let user_id: i32 = 1;

        client
            .query(
                "INSERT INTO user_list\
                (id,name)\
             VALUES\
                ($1::INTEGER, $2::VARCHAR)",
                &[&user_id, &username],
            )
            .await?;

        Ok(user_id)
    } else {
        let latest: Option<i32> = rows[0].get(0);
        let next_id: i32 = latest.unwrap() + 1;

        client
            .query(
                "INSERT INTO user_list\
                (id,name)\
            VALUES\
                ($1::INTEGER, $2::VARCHAR)",
                &[&next_id, &username],
            )
            .await?;

        Ok(next_id)
    }
}

pub async fn record(client: &Client, user_id: i32, temperature: f32) -> Result<(), Error> {
    // log_id (primary key, i64), date(timestamp), user_id(i32), temperature(f32)
    let timestamp: DateTime<Local> = Local::now();

    let rows = client
        .query("SELECT log_id FROM temperatures ORDER BY log_id DESC", &[])
        .await?;

    let log_id: i64 = if rows.is_empty() {
        1
    } else {
        let tmp: Option<i64> = rows[0].get(0);
        tmp.unwrap() + 1
    };

    client
        .query(
            "INSERT INTO temperatures\
            (log_id,user_id,date,temperature)\
        VALUES\
            ($1::BIGINT, $2::INTEGER, $3::TIMESTAMP WITH TIME ZONE,$4::REAL)",
            &[&log_id, &user_id, &timestamp, &temperature],
        )
        .await?;

    Ok(())
}

pub struct Record {
    date: DateTime<Local>,
    name: String,
    temperature: f32,
}

pub async fn fetch_record(
    client: &Client,
    user_id: i32,
    start: &DateTime<Local>,
    end: &DateTime<Local>,
) -> Result<Vec<Record>, Error> {
    let rows = client
        .query(
            "SELECT \
            temperatures.date, user_list.name, temperatures.temperature \
        FROM \
            temperatures \
        INNER JOIN \
            user_list \
        ON \
            temperatures.user_id = user_list.id \
        WHERE \
            user_list.id = $1::INTEGER \
        AND \
            temperatures.date \
        BETWEEN \
            $2::TIMESTAMP WITH TIME ZONE \
        AND \
            $3::TIMESTAMP WITH TIME ZONE",
            &[&user_id, &start, &end],
        )
        .await?;

    let mut records = Vec::<Record>::new();

    for r in &rows {
        let record = Record {
            date: r.get(0),
            name: r.get(1),
            temperature: r.get(2),
        };
        records.push(record);
    }

    Ok(records)
}
