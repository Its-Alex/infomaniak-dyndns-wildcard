use reqwest::blocking::{Client, Response};
use serde_json::{json, Value};
use std::error::Error;

const INFOMANIAK_API_URL: &str = "https://api.infomaniak.com/2/zones";

pub fn get_dns_records(
    client: &Client,
    ip: &str,
    dns_zone_id: &str,
    record_name: &str,
    record_type: &str,
) -> Result<Option<String>, Box<dyn Error>> {
    // Retrieve existing records
    let response: Response = client
        .get(format!("{}/{}/records", INFOMANIAK_API_URL, dns_zone_id))
        .send()?;
    if !response.status().is_success() {
        return Err(format!("Error retrieving DNS records: {}", response.status()).into());
    }
    let records: Value = response.json()?;

    // Check if an existing record matches
    let mut record_id: Option<String> = None;
    if let Some(records_array) = records["data"].as_array() {
        for record in records_array {
            if record["source"] == record_name && record["type"] == record_type {
                if let Some(target) = Some(
                    record["target"]
                        .as_str()
                        .unwrap()
                        .trim_matches('"')
                        .to_string(),
                ) {
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
pub fn update_dns_record(
    client: &Client,
    ip: &str,
    record_id: Option<&str>,
    dns_zone_id: &str,
    record_name: &str,
    record_type: &str,
) -> Result<Value, Box<dyn Error>> {
    // Prepare data for updating or creating
    let record_data = json!({
        "source": record_name,
        "target": ip,
        "type": record_type,
        "ttl": "300" // TTL in seconds
    });

    if let Some(id) = record_id {
        // Update existing record
        let update_url = format!("{}/{}/records/{}", INFOMANIAK_API_URL, dns_zone_id, id);
        let result = client.delete(&update_url).send()?;
        if !result.status().is_success() {
            return Err(format!(
                "Error updating DNS record {} of type {}: {}",
                record_name,
                record_type,
                result.status()
            )
            .into());
        }
    }

    // Create a new record
    let result = client
        .post(format!("{}/{}/records", INFOMANIAK_API_URL, dns_zone_id))
        .json(&record_data)
        .send()?;
    if !result.status().is_success() {
        return Err(format!(
            "Error updating DNS records: {}, body: {:?}",
            result.status(),
            result.text()
        )
        .into());
    }
    Ok(result.json()?)
}
