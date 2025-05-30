use config::Config;
use reqwest::blocking::Client;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
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
    let ipv6_enabled = config.get_bool("ipv6_enabled").unwrap_or(false);

    let client = create_http_client(&api_token);

    loop {
        match public_ip::get_public_ipv4_with_url(&client, IPIFY_IPV4_URL) {
            Ok(ip) => {
                println!("Public IP: {}", ip);
                match dns_record::get_dns_records(
                    &client,
                    INFOMANIAK_ZONES_API_URL,
                    &dns_zone_id,
                    &record_name,
                    "A",
                ) {
                    Ok(Some(record)) => {
                        if record.target == ip.to_string() {
                            println!("DNS record for IPv4 is already up to date.");
                        } else {
                            match dns_record::update_dns_record(
                                &client,
                                INFOMANIAK_ZONES_API_URL,
                                &ip.to_string(),
                                Some(&record.id.to_string()),
                                &dns_zone_id,
                                &record_name,
                                "A",
                            ) {
                                Ok(result) => println!("Update IPv4 successful: {:?}", result),
                                Err(e) => eprintln!("Error updating DNS for IPv4: {}", e),
                            }
                        }
                    }
                    Ok(None) => {
                        match dns_record::update_dns_record(
                            &client,
                            INFOMANIAK_ZONES_API_URL,
                            &ip.to_string(),
                            None,
                            &dns_zone_id,
                            &record_name,
                            "A",
                        ) {
                            Ok(result) => println!("Update IPv4 successful: {:?}", result),
                            Err(e) => eprintln!("Error updating DNS for IPv4: {}", e),
                        }
                    }
                    Err(e) => eprintln!("Error retrieving DNS records for IPv4: {}", e),
                }
            }
            Err(e) => eprintln!("Error retrieving public IPv4: {}", e),
        }
        if ipv6_enabled {
            match public_ip::get_public_ipv6_with_url(&client, IPIFY_IPV6_URL) {
                Ok(ip) => {
                    println!("Public IPv6: {}", ip);
                    match dns_record::get_dns_records(
                        &client,
                        INFOMANIAK_ZONES_API_URL,
                        &dns_zone_id,
                        &record_name,
                        "AAAA",
                    ) {
                        Ok(Some(record)) => {
                            if record.target == ip.to_string() {
                                println!("DNS record for IPv6 is already up to date.");
                            } else {
                                match dns_record::update_dns_record(
                                    &client,
                                    INFOMANIAK_ZONES_API_URL,
                                    &ip.to_string(),
                                    Some(&record.id.to_string()),
                                    &dns_zone_id,
                                    &record_name,
                                    "AAAA",
                                ) {
                                    Ok(result) => println!("Update IPv6 successful: {:?}", result),
                                    Err(e) => eprintln!("Error updating DNS for IPv6: {}", e),
                                }
                            }
                        }
                        Ok(None) => {
                            match dns_record::update_dns_record(
                                &client,
                                INFOMANIAK_ZONES_API_URL,
                                &ip.to_string(),
                                None,
                                &dns_zone_id,
                                &record_name,
                                "AAAA",
                            ) {
                                Ok(result) => println!("Update IPv6 successful: {:?}", result),
                                Err(e) => eprintln!("Error updating DNS for IPv6: {}", e),
                            }
                        }
                        Err(e) => eprintln!("Error retrieving DNS records for IPv6: {}", e),
                    }
                }
                Err(e) => eprintln!("Error retrieving public IPv6: {}", e),
            }
        }
        thread::sleep(Duration::from_secs(time_between_updates_in_seconds));
    }
}
