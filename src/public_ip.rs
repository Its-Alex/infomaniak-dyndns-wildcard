use reqwest::blocking::Client;
use std::error::Error;
use std::net::Ipv4Addr;

/// Retrieves the public IP address.
pub fn get_public_ipv4(client: &Client) -> Result<Ipv4Addr, Box<dyn Error>> {
    Ok(client.get("https://api.ipify.org/").send()?.text()?.trim().parse::<Ipv4Addr>()?)
}
