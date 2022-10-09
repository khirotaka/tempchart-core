use anyhow::{anyhow, Result};
use chrono::{Local, TimeZone};
use jsonwebtoken::{Algorithm, DecodingKey, TokenData, Validation};
use openssl::x509::X509;
use serde_json::Value;
use std::collections::HashMap;
use std::str;

async fn fetch_google_pem(key_id: &str) -> Result<Vec<u8>> {
    let google_api_setting = config::Config::builder()
        .add_source(config::File::with_name("config/googleapi.yaml"))
        .build()
        .unwrap();

    let url = google_api_setting.get_string("url")?;

    let pub_key_json = reqwest::get(url)
        .await?
        .json::<HashMap<String, String>>()
        .await?;

    let pub_key = match pub_key_json.get(key_id) {
        Some(key) => key.as_bytes(),
        None => {
            return Err(anyhow!(
                "The corresponding key was not found in the obtained public key list."
            ))
        }
    };

    let certificate = match X509::from_pem(pub_key) {
        Ok(cert) => cert,
        Err(e) => panic!("{}", e),
    };

    let pem_bytes = certificate.public_key()?.rsa()?.public_key_to_pem()?;

    Ok(pem_bytes)
}

pub async fn valid_jwt(id_token: &str) -> Result<TokenData<HashMap<String, Value>>> {
    let header = jsonwebtoken::decode_header(id_token)?;
    let kid = match header.kid {
        Some(k) => k,
        None => return Err(anyhow!("Token doesn't have a `kid` header field")),
    };

    let google_pem = fetch_google_pem(kid.as_str()).await?;

    let decode_key = DecodingKey::from_rsa_pem(google_pem.as_slice())?;

    let validation = Validation::new(Algorithm::RS256);
    let decoded_token =
        jsonwebtoken::decode::<HashMap<String, Value>>(id_token, &decode_key, &validation).unwrap();

    let now = Local::now();
    let expiration_time = Local.timestamp(
        decoded_token.claims.get("exp").unwrap().as_i64().unwrap(),
        0,
    );

    if expiration_time > now {
        Ok(decoded_token)
    } else {
        Err(anyhow!("The certificate has expired."))
    }
}
