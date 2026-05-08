// Network-effect patterns.

use reqwest::Client;
use std::net::{TcpListener, TcpStream};

use crate::errors::raise_if_invalid;

pub async fn fetch_user(client: &Client, id: u64) -> Result<String, reqwest::Error> {
    // expect: Net=definite, Async=definite, Throw=probable
    raise_if_invalid(id);
    let url = format!("https://api.example/users/{id}");
    let resp = client.get(&url).send().await?;
    resp.text().await
}

pub async fn post_event(client: &Client, payload: &str) -> Result<(), reqwest::Error> {
    // expect: Net=definite, Async=definite
    client.post("https://api.example/events").body(payload.to_string()).send().await?;
    Ok(())
}

pub fn open_tcp(addr: &str) -> std::io::Result<TcpStream> {
    // expect: Net=definite
    TcpStream::connect(addr)
}

pub fn bind_listener(addr: &str) -> std::io::Result<TcpListener> {
    // expect: Net=definite
    TcpListener::bind(addr)
}

pub fn make_request_helper(url: &str) -> String {
    // expect: Net=possible
    // Pure body; name pattern only.
    url.trim_end_matches('/').to_string()
}
