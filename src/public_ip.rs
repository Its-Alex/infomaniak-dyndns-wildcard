use std::error::Error;
use std::net::Ipv4Addr;

/// Retrieves the public IP address.
pub fn get_public_ip() -> Result<Ipv4Addr, Box<dyn Error>> {
    let response_ipv4 = reqwest::blocking::get("https://api.ipify.org/")?;

    Ok(response_ipv4.text()?.trim().parse::<Ipv4Addr>()?)
}
