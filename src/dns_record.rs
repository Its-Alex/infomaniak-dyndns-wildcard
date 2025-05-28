use reqwest::blocking::{Client, Response};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::error::Error;

const INFOMANIAK_API_URL: &str = "https://api.infomaniak.com/2/zones";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DnsRecord {
    pub id: u64,
    pub source: String,
    pub target: String,
    pub ttl: u32,
    #[serde(rename = "type")]
    pub record_type: String,
    pub updated_at: u64,
}

#[derive(Debug, Deserialize)]
struct GetRecordsResponse {
    data: Vec<DnsRecord>,
}

pub fn get_dns_records(
    client: &Client,
    dns_zone_id: &str,
    record_name: &str,
    record_type: &str,
) -> Result<Option<DnsRecord>, Box<dyn Error>> {
    // Retrieve existing records
    let response: Response = client
        .get(format!("{}/{}/records", INFOMANIAK_API_URL, dns_zone_id))
        .send()?;

    // Return an error if the request was not successful
    if !response.status().is_success() {
        return Err(format!(
            "Error retrieving DNS records: {:?}",
            response.json::<serde_json::Value>()?
        )
        .into());
    }

    // Return matching record if it exists
    let api_response: GetRecordsResponse = response.json()?;
    for record in api_response.data {
        if record.source == record_name && record.record_type == record_type {
            return Ok(Some(record));
        }
    }

    Ok(None)
}

#[derive(Debug, Deserialize)]
struct UpdateRecordResponse {
    data: DnsRecord,
}

/// Updates or creates a DNS record via the Infomaniak API.
pub fn update_dns_record(
    client: &Client,
    ip: &str,
    record_id_to_delete: Option<&str>,
    dns_zone_id: &str,
    record_name: &str,
    record_type: &str,
) -> Result<DnsRecord, Box<dyn Error>> {
    // Prepare data for updating or creating
    let record_data = json!({
        "source": record_name,
        "target": ip,
        "type": record_type,
        "ttl": "300" // TTL in seconds
    });

    if let Some(id) = record_id_to_delete {
        // Update existing record
        let delete_record_result = client
            .delete(format!(
                "{}/{}/records/{}",
                INFOMANIAK_API_URL, dns_zone_id, id
            ))
            .send()?;

        if !delete_record_result.status().is_success() {
            return Err(format!(
                "Error updating DNS record {} of type {}: {}",
                record_name,
                record_type,
                delete_record_result.status()
            )
            .into());
        }
    }

    // Create a new record
    let create_record_result = client
        .post(format!("{}/{}/records", INFOMANIAK_API_URL, dns_zone_id))
        .json(&record_data)
        .send()?;

    // Check if the request was successful
    if !create_record_result.status().is_success() {
        return Err(format!(
            "Error updating DNS records: {}, body: {:?}",
            create_record_result.status(),
            create_record_result.text()
        )
        .into());
    }

    let create_record_result: UpdateRecordResponse = create_record_result.json()?;

    Ok(create_record_result.data)
}
