use reqwest::blocking::Client;
use std::error::Error;
use std::net::Ipv4Addr;

/// Function that get public IPv4 address from a given URL.
pub fn get_public_ipv4_with_url(client: &Client, url: &str) -> Result<Ipv4Addr, Box<dyn Error>> {
    Ok(client.get(url).send()?.text()?.trim().parse::<Ipv4Addr>()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::blocking::Client;

    #[test]
    fn test_get_public_ipv4_success() {
        let mut server = mockito::Server::new();

        // Mock the API response
        let _m = server
            .mock("GET", "/")
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_body("192.168.1.1")
            .create();

        let client = Client::new();
        let result = get_public_ipv4_with_url(&client, &server.url());

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "192.168.1.1".parse::<Ipv4Addr>().unwrap());
    }

    #[test]
    fn test_get_public_ipv4_invalid_ip() {
        let mut server = mockito::Server::new();

        // Mock the API response with invalid IP
        let _m = server
            .mock("GET", "/")
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_body("invalid_ip")
            .create();

        let client = Client::new();
        let result = get_public_ipv4_with_url(&client, &server.url());

        assert!(result.is_err());
    }

    #[test]
    fn test_get_public_ipv4_with_whitespace() {
        let mut server = mockito::Server::new();

        // Mock the API response with whitespace
        let _m = server
            .mock("GET", "/")
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_body("  10.0.0.1  \n")
            .create();

        let client = Client::new();
        let result = get_public_ipv4_with_url(&client, &server.url());

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "10.0.0.1".parse::<Ipv4Addr>().unwrap());
    }

    #[test]
    fn test_get_public_ipv4_server_error() {
        let mut server = mockito::Server::new();

        // Mock a server error
        let _m = server.mock("GET", "/").with_status(500).create();

        let client = Client::new();
        let result = get_public_ipv4_with_url(&client, &server.url());

        assert!(result.is_err());
    }

    #[test]
    fn test_get_public_ipv4_network_error() {
        let client = Client::new();
        // Test with invalid URL to simulate network error
        let result =
            get_public_ipv4_with_url(&client, "http://invalid-url-that-does-not-exist.invalid");

        assert!(result.is_err());
    }
}
