use crate::utils::postgres::with_valid;
use axum::{http::StatusCode, response::IntoResponse, Json};
use chrono::{DateTime, Local, TimeZone};
use std::collections::HashMap;

pub mod structs {
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct CreateUser {
        pub token_id: String,
        pub username: String,
    }

    #[derive(Deserialize)]
    pub struct RecordTemperature {
        pub token_id: String,
        pub temperature: f32,
    }

    #[derive(Deserialize)]
    pub struct FetchRecord {
        pub token_id: String,
        pub start: String,
        pub end: String,
    }
}

pub async fn create_user(Json(payload): Json<structs::CreateUser>) -> impl IntoResponse {
    match with_valid::create_connection(payload.token_id.as_str()).await {
        Ok(client) => {
            match with_valid::create_user(
                payload.token_id.as_str(),
                &client,
                payload.username.as_str(),
            )
            .await
            {
                Ok(_) => StatusCode::CREATED,
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
            }
        }
        Err(_) => StatusCode::UNAUTHORIZED,
    }
}

pub async fn record_temperature(
    Json(payload): Json<structs::RecordTemperature>,
) -> impl IntoResponse {
    match with_valid::create_connection(payload.token_id.as_str()).await {
        Ok(client) => {
            match with_valid::record(payload.token_id.as_str(), &client, payload.temperature).await
            {
                Ok(_) => StatusCode::OK,
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
            }
        }
        Err(_) => StatusCode::UNAUTHORIZED,
    }
}

pub async fn fetch_record(Json(payload): Json<structs::FetchRecord>) -> impl IntoResponse {
    let start = Local
        .datetime_from_str(payload.start.as_str(), "%Y/%m/%d %H:%M:%S")
        .unwrap();
    let end = Local
        .datetime_from_str(payload.end.as_str(), "%Y/%m/%d %H:%M:%S")
        .unwrap();

    match with_valid::create_connection(payload.token_id.as_str()).await {
        Ok(client) => {
            match with_valid::fetch_record(payload.token_id.as_str(), &client, &start, &end).await {
                Ok(records) => {
                    if records.is_empty() {
                        (StatusCode::NOT_FOUND, "".to_string())
                    } else {
                        let mut record_dict = HashMap::<DateTime<Local>, f32>::new();

                        for record in records {
                            record_dict.insert(record.date, record.temperature);
                        }

                        let record_json = serde_json::to_string(&record_dict).unwrap();
                        (StatusCode::OK, record_json)
                    }
                }
                Err(_) => (StatusCode::UNAUTHORIZED, "".to_string()),
            }
        }
        Err(_) => (StatusCode::UNAUTHORIZED, "".to_string()),
    }
}
