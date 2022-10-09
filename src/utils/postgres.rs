use chrono::{DateTime, Local};

#[derive(Debug)]
pub struct Record {
    date: DateTime<Local>,
    name: String,
    temperature: f32,
}

pub mod raw {
    use crate::utils::postgres::Record;
    use chrono::{DateTime, Local};
    use tokio_postgres::{Client, Error, GenericClient, NoTls};

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

    pub async fn create_user(
        client: &Client,
        user_id: &str,
        username: &str,
    ) -> Result<Option<()>, Error> {
        let rows = client
            .query(
                "SELECT id FROM user_list WHERE id = $1::VARCHAR",
                &[&user_id],
            )
            .await?;

        if rows.is_empty() {
            client
                .query(
                    "INSERT INTO user_list\
                (id,name)\
             VALUES\
                ($1::VARCHAR, $2::VARCHAR)",
                    &[&user_id, &username],
                )
                .await?;

            Ok(Some(()))
        } else {
            Ok(None)
        }
    }

    pub async fn record(client: &Client, user_id: &str, temperature: f32) -> Result<(), Error> {
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
                ($1::BIGINT, $2::VARCHAR, $3::TIMESTAMP WITH TIME ZONE,$4::REAL)",
                &[&log_id, &user_id, &timestamp, &temperature],
            )
            .await?;

        Ok(())
    }

    pub async fn fetch_record(
        client: &Client,
        user_id: &str,
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
                user_list.id = $1::VARCHAR \
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
}

pub mod with_valid {
    use crate::utils::postgres::Record;
    use crate::utils::{auth, postgres::raw};
    use chrono::{DateTime, Local};
    use thiserror::Error;
    use tokio_postgres::{Client, Error};

    #[warn(clippy::enum_variant_names)]
    #[derive(Debug, Error)]
    pub enum DBError {
        #[error("Certificate verification failed")]
        JWTValidError,
        #[error("A problem occurred on the database.  {0}")]
        PostgresError(Error),
        #[error("User already registered")]
        UserAlreadyRegisteredError,
    }

    pub async fn create_connection(
        token_id: &str,
        user: &str,
        password: &str,
    ) -> Result<Client, DBError> {
        match auth::valid_jwt(token_id).await {
            Ok(_) => {
                // Firebase AuthからのJWTの検証に成功
                match raw::create_connection(user, password).await {
                    Ok(c) => Ok(c),
                    Err(e) => Err(DBError::PostgresError(e)),
                }
            }
            Err(_) => {
                // Firebase AuthからのJWTの検証に失敗
                Err(DBError::JWTValidError)
            }
        }
    }

    pub async fn create_user(
        token_id: &str,
        client: &Client,
        username: &str,
    ) -> Result<(), DBError> {
        match auth::valid_jwt(token_id).await {
            Ok(token) => {
                let user_id = token.claims.get("user_id").unwrap().as_str().unwrap();
                match raw::create_user(client, user_id, username).await {
                    Ok(result) => match result {
                        Some(_) => Ok(()),
                        None => Err(DBError::UserAlreadyRegisteredError),
                    },
                    Err(e) => Err(DBError::PostgresError(e)),
                }
            }
            Err(_) => {
                // Firebase AuthからのJWTの検証に失敗
                Err(DBError::JWTValidError)
            }
        }
    }

    pub async fn record(token_id: &str, client: &Client, temperature: f32) -> Result<(), DBError> {
        match auth::valid_jwt(token_id).await {
            Ok(token) => {
                let user_id = token.claims.get("user_id").unwrap().as_str().unwrap();
                match raw::record(client, user_id, temperature).await {
                    Ok(_) => Ok(()),
                    Err(e) => Err(DBError::PostgresError(e)),
                }
            }
            Err(_) => {
                // Firebase AuthからのJWTの検証に失敗
                Err(DBError::JWTValidError)
            }
        }
    }

    pub async fn fetch_record(
        token_id: &str,
        client: &Client,
        start: &DateTime<Local>,
        end: &DateTime<Local>,
    ) -> Result<Vec<Record>, DBError> {
        match auth::valid_jwt(token_id).await {
            Ok(token) => {
                let user_id = token.claims.get("user_id").unwrap().as_str().unwrap();
                match raw::fetch_record(client, user_id, start, end).await {
                    Ok(record) => Ok(record),
                    Err(e) => Err(DBError::PostgresError(e)),
                }
            }
            Err(_) => {
                // Firebase AuthからのJWTの検証に失敗
                Err(DBError::JWTValidError)
            }
        }
    }
}
