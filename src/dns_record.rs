use reqwest::blocking::{Client, Response};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::error::Error;

#[derive(Debug, Deserialize, Serialize)]
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
    infomaniak_zones_api_url: &str,
    dns_zone_id: &str,
    record_name: &str,
    record_type: &str,
) -> Result<Option<DnsRecord>, Box<dyn Error>> {
    // Retrieve existing records
    let response: Response = client
        .get(format!(
            "{}/{}/records",
            infomaniak_zones_api_url, dns_zone_id
        ))
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
    infomaniak_zones_api_url: &str,
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
                infomaniak_zones_api_url, dns_zone_id, id
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
        .post(format!(
            "{}/{}/records",
            infomaniak_zones_api_url, dns_zone_id
        ))
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

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;
    use reqwest::blocking::Client;
    use serde_json::json;

    #[test]
    fn test_get_dns_records_success_with_matching_record() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/test-zone/records")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "data": [
                        {
                            "id": 123,
                            "source": "test.example.com",
                            "target": "192.168.1.1",
                            "ttl": 300,
                            "type": "A",
                            "updated_at": 1234567890
                        },
                        {
                            "id": 124,
                            "source": "other.example.com",
                            "target": "192.168.1.2",
                            "ttl": 300,
                            "type": "A",
                            "updated_at": 1234567890
                        }
                    ]
                })
                .to_string(),
            )
            .create();

        let client = Client::new();
        let result = get_dns_records(&client, &server.url(), "test-zone", "test.example.com", "A");

        mock.assert();
        assert!(result.is_ok());
        let record = result.unwrap();
        assert!(record.is_some());
        let record = record.unwrap();
        assert_eq!(record.id, 123);
        assert_eq!(record.source, "test.example.com");
        assert_eq!(record.target, "192.168.1.1");
    }

    #[test]
    fn test_get_dns_records_success_no_matching_record() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/test-zone/records")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "data": [
                        {
                            "id": 124,
                            "source": "other.example.com",
                            "target": "192.168.1.2",
                            "ttl": 300,
                            "type": "A",
                            "updated_at": 1234567890
                        }
                    ]
                })
                .to_string(),
            )
            .create();

        let client = Client::new();
        let result = get_dns_records(&client, &server.url(), "test-zone", "test.example.com", "A");

        mock.assert();
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_get_dns_records_api_error() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/test-zone/records")
            .with_status(404)
            .with_header("content-type", "application/json")
            .with_body(json!({"error": "Zone not found"}).to_string())
            .create();

        let client = Client::new();
        let result = get_dns_records(&client, &server.url(), "test-zone", "test.example.com", "A");

        mock.assert();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Error retrieving DNS records")
        );
    }

    #[test]
    fn test_update_dns_record_create_new() {
        let mut server = Server::new();
        let mock = server
            .mock("POST", "/test-zone/records")
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "data": {
                        "id": 125,
                        "source": "new.example.com",
                        "target": "192.168.1.3",
                        "ttl": 300,
                        "type": "A",
                        "updated_at": 1234567890
                    }
                })
                .to_string(),
            )
            .create();

        let client = Client::new();
        let result = update_dns_record(
            &client,
            &server.url(),
            "192.168.1.3",
            None,
            "test-zone",
            "new.example.com",
            "A",
        );

        mock.assert();
        assert!(result.is_ok());
        let record = result.unwrap();
        assert_eq!(record.id, 125);
        assert_eq!(record.source, "new.example.com");
        assert_eq!(record.target, "192.168.1.3");
    }

    #[test]
    fn test_update_dns_record_update_existing() {
        let mut server = Server::new();
        let delete_mock = server
            .mock("DELETE", "/test-zone/records/123")
            .with_status(200)
            .create();

        let create_mock = server
            .mock("POST", "/test-zone/records")
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "data": {
                        "id": 126,
                        "source": "updated.example.com",
                        "target": "192.168.1.4",
                        "ttl": 300,
                        "type": "A",
                        "updated_at": 1234567890
                    }
                })
                .to_string(),
            )
            .create();

        let client = Client::new();
        let result = update_dns_record(
            &client,
            &server.url(),
            "192.168.1.4",
            Some("123"),
            "test-zone",
            "updated.example.com",
            "A",
        );

        delete_mock.assert();
        create_mock.assert();
        assert!(result.is_ok());
        let record = result.unwrap();
        assert_eq!(record.id, 126);
        assert_eq!(record.target, "192.168.1.4");
    }

    #[test]
    fn test_update_dns_record_delete_error() {
        let mut server = Server::new();
        let delete_mock = server
            .mock("DELETE", "/test-zone/records/123")
            .with_status(404)
            .create();

        let client = Client::new();
        let result = update_dns_record(
            &client,
            &server.url(),
            "192.168.1.4",
            Some("123"),
            "test-zone",
            "updated.example.com",
            "A",
        );

        delete_mock.assert();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Error updating DNS record")
        );
    }

    #[test]
    fn test_update_dns_record_create_error() {
        let mut server = Server::new();
        let create_mock = server
            .mock("POST", "/test-zone/records")
            .with_status(400)
            .with_body("Bad request")
            .create();

        let client = Client::new();
        let result = update_dns_record(
            &client,
            &server.url(),
            "192.168.1.5",
            None,
            "test-zone",
            "error.example.com",
            "A",
        );

        create_mock.assert();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Error updating DNS records")
        );
    }
}
