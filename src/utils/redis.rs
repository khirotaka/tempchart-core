use num_traits::cast::ToPrimitive;
use redis::{Commands, RedisResult};
use chrono::{Local, DateTime, TimeZone};
use rand::Rng;


pub struct Database {
    client: redis::Client,
    connection: Option<redis::Connection>,
}

impl Database {
    pub fn new() -> Database {
        let result = redis::Client::open("redis://127.0.0.1/");
        let client = match result {
            Ok(c) => c,
            Err(e) => {
                panic!("Failed to open Database: {:?}", e);
            },
        };

        Database { client, connection: None }
    }

    pub fn connect(&mut self) {
        let result = self.client.get_connection();

        let connection = match result {
            Ok(c) => c,
            Err(e) => {
                panic!("Failed to connect Database: {:?}", e)
            },
        };

        self.connection = Some(connection);
    }

    pub fn record(&mut self, user_id: u8, temperature: f32) {
        let mut rng = rand::thread_rng();
        let rand_value: f64 = rng.gen();

        let now = Local::now();
        let timestamp = now.timestamp().to_f64().unwrap() + rand_value;

        let result: RedisResult<f32> = self.connection.as_mut()
            .expect("Failed to connect database")
            .zadd(
                user_id,
                temperature,
                timestamp,
            );

        if let Err(e) = result {
            panic!("Failed to record temperature: {:?}", e)
        };
    }

    pub fn fetch_record(
        &mut self,
        user_id: u8,
        start: DateTime<Local>,
        end: DateTime<Local>
    ) -> Vec<(DateTime<Local>, f32)> {
        if end > start {
            let start_unix = start.timestamp().to_f64().unwrap();
            let end_unix = end.timestamp().to_f64().unwrap();

            let result: RedisResult<Vec<(f32, f32)>> =
                self.connection.as_mut()
                    .expect("Failed to connect database")
                    .zrangebyscore_withscores(
                        user_id, start_unix, end_unix
                    );

            let time: Vec<(f32, f32)> = match result {
                Ok(t) => t,
                Err(e) => {
                    panic!("Failed to fetch the record: {:?}", e)
                }
            };

            let mut result = Vec::<(DateTime<Local>, f32)>::new();

            for (temperature, unix_time) in time {
                let date_time = Local.timestamp(unix_time.to_i64().unwrap(), 0);
                result.push((date_time, temperature));
            }

            result
        }
        else {
            panic!("The end time is older than the start time.")
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::redis::Database;
    use chrono::Local;

    #[test]
    fn test_db() {
        let start = Local::now();
        let mut db = Database::new();
        db.connect();

        db.record(1, 32.3);
        std::thread::sleep(std::time::Duration::from_secs(1));

        db.record(1, 54.3);
        std::thread::sleep(std::time::Duration::from_secs(1));

        db.record(1, 76.3);
        std::thread::sleep(std::time::Duration::from_secs(1));

        let end = Local::now();
        let result = db.fetch_record(1, start, end);

        let values: Vec<&f32> = result.iter().map(|(_,v)|v).collect();
        println!("{:?}", values);

        db.record(2, 33.1);
        std::thread::sleep(std::time::Duration::from_secs(1));

        db.record(2, 3.21);
        std::thread::sleep(std::time::Duration::from_secs(1));

        db.record(2, 4.32);
        std::thread::sleep(std::time::Duration::from_secs(1));

        let end = Local::now();
        let result = db.fetch_record(2, start, end);

        let values: Vec<&f32> = result.iter().map(|(_,v)|v).collect();
        println!("{:?}", values);
    }
}