use reqwest::blocking::{Client, Response};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::{json, Value};
use std::error::Error;
use config::Config;
use std::time::Duration;
use std::thread;

const RECORD_TYPE: &str = "A";
const INFOMANIAK_API_URL: &str = "https://api.infomaniak.com/2/zones";

/// Retrieves the public IP address.
fn get_public_ip() -> Result<String, Box<dyn Error>> {
    let response = reqwest::blocking::get("https://ipinfo.io/ip")?;
    Ok(response.text()?.trim().to_string())
}

fn create_client(api_token: &str) -> Client {
    let mut headers: HeaderMap = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", api_token))
        .expect("Failed to create authorization header"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("Content-Type: application/json"));
    Client::builder().default_headers(headers).build()
        .expect("Failed to build client")
}

fn get_dns_records(client: &Client, ip: &str, dns_zone_id: &str, record_name: &str) -> Result<Option<String>, Box<dyn Error>> {
    // Retrieve existing records
    let response: Response = client.get(format!("{}/{}/records", INFOMANIAK_API_URL, dns_zone_id)).send()?;
    if !response.status().is_success() {
        return Err(format!("Error retrieving DNS records: {}", response.status()).into());
    }
    let records: Value = response.json()?;

    // Check if an existing record matches
    let mut record_id: Option<String> = None;
    if let Some(records_array) = records["data"].as_array() {
        for record in records_array {
            if record["source"] == record_name && record["type"] == RECORD_TYPE {
                if let Some(target) = Some(record["target"].as_str().unwrap().trim_matches('"').to_string()) {
                    if target == ip {
                        return Err("Record found, but no changes detected".into());
                    }
                }

                record_id = Some(record["id"].to_string());
                break;
            }
        }
    }

    if record_id.is_none() {
        println!("No existing record found");
        return Ok(None);
    }

    Ok(record_id)
}

/// Updates or creates a DNS record via the Infomaniak API.
fn update_dns_record(client: &Client, ip: &str, record_id: Option<&str>, dns_zone_id: &str, record_name: &str) -> Result<Value, Box<dyn Error>> {
    // Prepare data for updating or creating
    let record_data = json!({
        "source": record_name,
        "target": ip,
        "type": RECORD_TYPE,
        "ttl": "300" // TTL in seconds
    });

    if let Some(id) = record_id {
        // Update existing record
        let update_url = format!("{}/{}/records/{}", INFOMANIAK_API_URL, dns_zone_id, id);
        let result = client.delete(&update_url).send()?;
        if !result.status().is_success() {
            return Err(format!("Error updating DNS records: {}", result.status()).into());
        }
    }

    // Create a new record
    let result = client.post(format!("{}/{}/records", INFOMANIAK_API_URL, dns_zone_id)).json(&record_data).send()?;
    if !result.status().is_success() {
        return Err(format!("Error updating DNS records: {}, body: {:?}", result.status(), result.text()).into());
    }
    Ok(result.json()?)
}

fn main() {
    let config = Config::builder()
        .add_source(config::Environment::with_prefix("infomaniak_dyndns_wildcard"))
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

    let client = create_client(&api_token);

    loop {
        match get_public_ip() {
            Ok(ip) => {
                println!("Public IP: {}", ip);
                match get_dns_records(&client, &ip, &dns_zone_id, &record_name) {
                    Ok(record_id) => {
                        match update_dns_record(&client, &ip, record_id.as_deref(), &dns_zone_id, &record_name) {
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