use std::{error::Error as _, time::Duration};

use qbittorrent_rs::{Client, Error};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{header, path},
};

use crate::common::mock_client;

#[tokio::test]
async fn sends_the_api_key_as_bearer() {
    let (server, client) = mock_client().await;
    Mock::given(header("authorization", "Bearer key"))
        .and(path("/api/v2/app/version"))
        .respond_with(ResponseTemplate::new(200).set_body_string("v5.2.3"))
        .expect(1)
        .mount(&server)
        .await;

    client.version().await.unwrap();
}

#[tokio::test]
async fn accepts_a_base_url_with_a_trailing_slash() {
    let server = MockServer::start().await;
    let client = Client::new(format!("{}/", server.uri()), "key");
    Mock::given(path("/api/v2/app/version"))
        .respond_with(ResponseTemplate::new(200).set_body_string("v5.2.3"))
        .expect(1)
        .mount(&server)
        .await;

    client.version().await.unwrap();
}

#[tokio::test]
async fn forbidden_is_unauthorized() {
    let (server, client) = mock_client().await;
    Mock::given(path("/api/v2/app/version"))
        .respond_with(ResponseTemplate::new(403).set_body_string("Forbidden"))
        .mount(&server)
        .await;

    let error = client.version().await.unwrap_err();

    assert!(matches!(error, Error::Unauthorized));
    assert_eq!(error.to_string(), "qBittorrent rejected the API key");
    assert!(error.source().is_none());
}

#[tokio::test]
async fn other_error_statuses_keep_the_message() {
    let (server, client) = mock_client().await;
    Mock::given(path("/api/v2/app/version"))
        .respond_with(ResponseTemplate::new(409).set_body_string("Conflict"))
        .mount(&server)
        .await;

    let error = client.version().await.unwrap_err();

    assert!(matches!(&error, Error::Api { status: 409, message } if message == "Conflict"));
    assert_eq!(error.to_string(), "qBittorrent returned 409: Conflict");
}

#[tokio::test]
async fn invalid_json_is_a_decode_error() {
    let (server, client) = mock_client().await;
    Mock::given(path("/api/v2/torrents/info"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
        .mount(&server)
        .await;

    let error = client
        .torrents()
        .list(&Default::default())
        .await
        .unwrap_err();

    assert!(matches!(error, Error::Decode(_)));
    assert!(
        error
            .to_string()
            .starts_with("qBittorrent response could not be decoded")
    );
    assert!(error.source().is_some());
}

#[tokio::test]
async fn slow_responses_time_out() {
    let (server, client) = mock_client().await;
    let client = client.timeout(Duration::from_millis(50));
    Mock::given(path("/api/v2/app/version"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(2)))
        .mount(&server)
        .await;

    let error = client.version().await.unwrap_err();

    assert!(matches!(error, Error::Timeout));
    assert_eq!(error.to_string(), "qBittorrent request timed out");
}

#[tokio::test]
async fn unreachable_server_is_a_network_error() {
    let client = Client::new("http://127.0.0.1:9", "key");

    let error = client.version().await.unwrap_err();

    assert!(matches!(error, Error::Network(_)));
    assert!(error.to_string().starts_with("qBittorrent request failed"));
    assert!(error.source().is_some());
}
