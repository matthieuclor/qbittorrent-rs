use wiremock::{
    Mock, ResponseTemplate,
    matchers::{method, path},
};

use crate::common::mock_client;

#[tokio::test]
async fn version_returns_the_application_version() {
    let (server, client) = mock_client().await;
    Mock::given(method("GET"))
        .and(path("/api/v2/app/version"))
        .respond_with(ResponseTemplate::new(200).set_body_string("v5.2.3"))
        .expect(1)
        .mount(&server)
        .await;

    assert_eq!(client.version().await.unwrap(), "v5.2.3");
}

#[tokio::test]
async fn webapi_version_returns_the_api_version() {
    let (server, client) = mock_client().await;
    Mock::given(method("GET"))
        .and(path("/api/v2/app/webapiVersion"))
        .respond_with(ResponseTemplate::new(200).set_body_string("2.15.1"))
        .expect(1)
        .mount(&server)
        .await;

    assert_eq!(client.webapi_version().await.unwrap(), "2.15.1");
}
