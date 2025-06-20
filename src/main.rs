use config::Config;
use log::{error, info};
use reqwest::blocking::Client;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use std::net::Ipv6Addr;
use std::thread;
use std::time::Duration;

mod dns_record;
mod public_ip;

const IPIFY_IPV4_URL: &str = "https://api.ipify.org/";
const IPIFY_IPV6_URL: &str = "https://api64.ipify.org/";
const INFOMANIAK_ZONES_API_URL: &str = "https://api.infomaniak.com/2/zones";

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
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

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
    let records_name = config
        .get_string("records_name")
        .expect("records_name must be set");
    let ipv6_enabled = config.get_bool("ipv6_enabled").unwrap_or(false);

    let client = create_http_client(&api_token);

    loop {
        let public_ipv4 = match public_ip::get_public_ipv4_with_url(&client, IPIFY_IPV4_URL) {
            Ok(ip) => {
                info!("Public IPv4: {}", ip);
                ip
            }
            Err(e) => {
                error!("Error retrieving public IPv4: {}", e);
                thread::sleep(Duration::from_secs(time_between_updates_in_seconds));
                continue;
            }
        };

        let mut public_ipv6: Option<Ipv6Addr> = None;
        if ipv6_enabled {
            public_ipv6 = match public_ip::get_public_ipv6_with_url(&client, IPIFY_IPV6_URL) {
                Ok(ip) => {
                    info!("Public IPv4: {}", ip);
                    Some(ip)
                }
                Err(e) => {
                    error!("Error retrieving public IPv4: {}", e);
                    thread::sleep(Duration::from_secs(time_between_updates_in_seconds));
                    continue;
                }
            };
        }

        let dns_records =
            match dns_record::get_dns_records(&client, INFOMANIAK_ZONES_API_URL, &dns_zone_id) {
                Ok(records) => {
                    info!("Existing DNS record found: {:?}", records);
                    records
                }
                Err(e) => {
                    error!("Error retrieving DNS records for IPv4: {}", e);
                    thread::sleep(Duration::from_secs(time_between_updates_in_seconds));
                    continue;
                }
            };

        for record_name in records_name.split(',') {
            let mut record_a_found = false;
            let mut record_aaaa_found = false;
            for record in &dns_records {
                if record.source == record_name && record.record_type == "A" {
                    info!("Found A record: {:?}", record);
                    record_a_found = true;

                    if record.target == public_ipv4.to_string() {
                        info!("DNS record for IPv4 is already up to date.");
                    } else {
                        info!("Updating DNS record for IPv4...");
                        match dns_record::update_dns_record(
                            &client,
                            INFOMANIAK_ZONES_API_URL,
                            &public_ipv4.to_string(),
                            Some(&record.id.to_string()),
                            &dns_zone_id,
                            record_name,
                            "A",
                        ) {
                            Ok(result) => info!("Update IPv4 successful: {:?}", result),
                            Err(e) => error!("Error updating DNS for IPv4: {}", e),
                        }
                    }
                }
                if ipv6_enabled && record.source == record_name && record.record_type == "AAAA" {
                    info!("Found AAAA record: {:?}", record);
                    record_aaaa_found = true;

                    if record.target == public_ipv6.unwrap().to_string() {
                        info!("DNS record for IPv6 is already up to date.");
                    } else {
                        info!("Updating DNS record for IPv6...");
                        match dns_record::update_dns_record(
                            &client,
                            INFOMANIAK_ZONES_API_URL,
                            &public_ipv6.unwrap().to_string(),
                            Some(&record.id.to_string()),
                            &dns_zone_id,
                            record_name,
                            "AAAA",
                        ) {
                            Ok(result) => info!("Update IPv6 successful: {:?}", result),
                            Err(e) => error!("Error updating DNS for IPv6: {}", e),
                        }
                    }
                }
                if record_a_found && record_aaaa_found {
                    break; // No need to check further if both records are found
                }
            }
            if !record_a_found {
                info!("No matching A record found for {}", record_name);
                match dns_record::update_dns_record(
                    &client,
                    INFOMANIAK_ZONES_API_URL,
                    &public_ipv4.to_string(),
                    None,
                    &dns_zone_id,
                    record_name,
                    "A",
                ) {
                    Ok(result) => info!("Update IPv4 successful: {:?}", result),
                    Err(e) => error!("Error updating DNS for IPv4: {}", e),
                }
            }
            if ipv6_enabled && !record_aaaa_found {
                info!("No matching AAAA record found for {}", record_name);
                match dns_record::update_dns_record(
                    &client,
                    INFOMANIAK_ZONES_API_URL,
                    &public_ipv6.unwrap().to_string(),
                    None,
                    &dns_zone_id,
                    record_name,
                    "AAAA",
                ) {
                    Ok(result) => info!("Update IPv6 successful: {:?}", result),
                    Err(e) => error!("Error updating DNS for IPv6: {}", e),
                }
            }
        }

        thread::sleep(Duration::from_secs(time_between_updates_in_seconds));
    }
}
