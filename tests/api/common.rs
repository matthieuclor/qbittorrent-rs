use qbittorrent_rs::Client;
use wiremock::MockServer;

pub async fn mock_client() -> (MockServer, Client) {
    let server = MockServer::start().await;
    let client = Client::new(server.uri(), "key");
    (server, client)
}

pub fn fixture(name: &str) -> String {
    std::fs::read_to_string(format!(
        "{}/tests/fixtures/{name}.json",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
}
