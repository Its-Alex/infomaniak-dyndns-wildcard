use config::Config;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use std::thread;
use std::time::Duration;

mod dns_record;
mod public_ip;

const RECORD_TYPE: &str = "A";

fn create_http_client(api_token: &str) -> Client {
    let mut headers: HeaderMap = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", api_token))
            .expect("Failed to create authorization header"),
    );
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("Content-Type: application/json"),
    );
    Client::builder()
        .default_headers(headers)
        .build()
        .expect("Failed to build client")
}

fn main() {
    let config = Config::builder()
        .add_source(config::Environment::with_prefix(
            "infomaniak_dyndns_wildcard",
        ))
        .build()
        .unwrap();
    let time_between_updates_in_seconds = config
        .get::<u64>("time_between_updates_in_seconds")
        .expect("time_between_updates_in_seconds must be set");
    let api_token = config
        .get_string("infomaniak_api_token")
        .expect("infomaniak_api_token must be set");
    let dns_zone_id = config
        .get_string("dns_zone_id")
        .expect("dns_zone_id must be set");
    let record_name = config
        .get_string("record_name")
        .expect("record_name must be set");

    let client = create_http_client(&api_token);

    loop {
        match public_ip::get_public_ipv4(&client) {
            Ok(ip) => {
                println!("Public IP: {}", ip);
                match dns_record::get_dns_records(
                    &client,
                    &ip.to_string(),
                    &dns_zone_id,
                    &record_name,
                    RECORD_TYPE,
                ) {
                    Ok(record_id) => {
                        match dns_record::update_dns_record(
                            &client,
                            &ip.to_string(),
                            record_id.as_deref(),
                            &dns_zone_id,
                            &record_name,
                            RECORD_TYPE,
                        ) {
                            Ok(result) => println!("Update successful: {:?}", result),
                            Err(e) => eprintln!("Error updating DNS: {}", e),
                        }
                    }
                    Err(e) => eprintln!("Error retrieving dns records: {}", e),
                }
            }
            Err(e) => eprintln!("Error retrieving public IP: {}", e),
        }
        thread::sleep(Duration::from_secs(time_between_updates_in_seconds));
    }
}
