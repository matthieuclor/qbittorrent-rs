use std::time::Duration;

use qbittorrent_rs::{AddTorrent, AddedTorrents, Error, Filter, Hashes, State, TorrentQuery};
use serde_json::json;
use wiremock::{
    Mock, ResponseTemplate,
    matchers::{body_string, body_string_contains, method, path, query_param},
};

use crate::common::{fixture, mock_client};

fn added() -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({
        "added_torrent_ids": ["c11088ea0fc66a8547eae189a0ae4c31ee916f1c"],
        "failure_count": 0,
        "pending_count": 0,
        "success_count": 1
    }))
}

#[tokio::test]
async fn list_decodes_torrents() {
    let (server, client) = mock_client().await;
    Mock::given(method("GET"))
        .and(path("/api/v2/torrents/info"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(fixture("torrents-info"), "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let torrents = client.torrents().list(&TorrentQuery::new()).await.unwrap();
    let [stopped, downloading] = torrents.as_slice() else {
        panic!("expected two torrents");
    };

    assert_eq!(stopped.hash, "c11088ea0fc66a8547eae189a0ae4c31ee916f1c");
    assert_eq!(stopped.state, State::StoppedDl);
    assert!(!stopped.state.is_complete());
    assert_eq!(stopped.eta, None);
    assert_eq!(stopped.completion_on, None);
    assert!(stopped.tags.is_empty());
    assert_eq!(downloading.state, State::Unknown);
    assert_eq!(downloading.progress, 0.75);
    assert_eq!(downloading.eta, Some(Duration::from_secs(120)));
    assert_eq!(downloading.completion_on, Some(1_795_337_000));
    assert_eq!(downloading.tags, ["linux", "iso"]);
    assert_eq!(downloading.dlspeed, 2_097_152);
    assert_eq!(downloading.category, "movies");
}

#[tokio::test]
async fn list_sends_the_query() {
    let (server, client) = mock_client().await;
    Mock::given(path("/api/v2/torrents/info"))
        .and(query_param("filter", "stalled_downloading"))
        .and(query_param("category", "movies"))
        .and(query_param("tag", "my_vod"))
        .and(query_param("hashes", "a|b"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .expect(1)
        .mount(&server)
        .await;
    let query = TorrentQuery::new()
        .filter(Filter::StalledDownloading)
        .category("movies")
        .tag("my_vod")
        .hashes(["a", "b"].into_iter().collect::<Hashes>());

    let torrents = client.torrents().list(&query).await.unwrap();

    assert!(torrents.is_empty());
}

#[test]
fn completed_states_are_complete() {
    let states: Vec<State> = serde_json::from_value(json!([
        "uploading",
        "stoppedUP",
        "queuedUP",
        "stalledUP",
        "checkingUP",
        "forcedUP",
        "error",
        "missingFiles",
        "allocating",
        "downloading",
        "metaDL",
        "forcedMetaDL",
        "stoppedDL",
        "queuedDL",
        "stalledDL",
        "checkingDL",
        "forcedDL",
        "checkingResumeData",
        "moving",
        "unknown"
    ]))
    .unwrap();

    let complete: Vec<bool> = states.iter().map(|state| state.is_complete()).collect();

    assert_eq!(complete[..6], [true; 6]);
    assert_eq!(complete[6..], [false; 14]);
    assert_eq!(states[19], State::Unknown);
}

#[tokio::test]
async fn add_sends_the_file_and_options() {
    let (server, client) = mock_client().await;
    Mock::given(method("POST"))
        .and(path("/api/v2/torrents/add"))
        .and(body_string_contains("filename=\"movie.torrent\""))
        .and(body_string_contains("d4:infoe"))
        .and(body_string_contains("name=\"savepath\"\r\n\r\n/downloads"))
        .and(body_string_contains("name=\"category\"\r\n\r\nmovies"))
        .and(body_string_contains("name=\"tags\"\r\n\r\nmy_vod,my_vod-1"))
        .and(body_string_contains("name=\"stopped\"\r\n\r\ntrue"))
        .respond_with(added())
        .expect(1)
        .mount(&server)
        .await;
    let torrent = AddTorrent::file("movie.torrent", b"d4:infoe".to_vec())
        .save_path("/downloads")
        .category("movies")
        .tag("my_vod")
        .tag("my_vod-1")
        .stopped(true);

    let added = client.torrents().add(torrent).await.unwrap();

    assert_eq!(
        added,
        AddedTorrents {
            added_torrent_ids: vec!["c11088ea0fc66a8547eae189a0ae4c31ee916f1c".to_string()],
            success_count: 1,
            failure_count: 0,
            pending_count: 0,
        }
    );
}

#[tokio::test]
async fn add_omits_unset_options() {
    let (server, client) = mock_client().await;
    Mock::given(path("/api/v2/torrents/add"))
        .and(body_string_contains("name=\"stopped\"\r\n\r\nfalse"))
        .respond_with(added())
        .expect(1)
        .mount(&server)
        .await;

    client
        .torrents()
        .add(AddTorrent::file("movie.torrent", b"d4:infoe".to_vec()))
        .await
        .unwrap();
    let requests = server.received_requests().await.unwrap();
    let body = String::from_utf8_lossy(&requests[0].body);

    assert!(!body.contains("savepath"));
    assert!(!body.contains("category"));
    assert!(!body.contains("tags"));
}

#[tokio::test]
async fn add_reports_an_invalid_torrent() {
    let (server, client) = mock_client().await;
    Mock::given(path("/api/v2/torrents/add"))
        .respond_with(ResponseTemplate::new(415).set_body_string("invalid"))
        .mount(&server)
        .await;

    let error = client
        .torrents()
        .add(AddTorrent::file("bad.torrent", b"bad".to_vec()))
        .await
        .unwrap_err();

    assert!(matches!(error, Error::Api { status: 415, .. }));
}

#[tokio::test]
async fn add_rejects_an_unexpected_body() {
    let (server, client) = mock_client().await;
    Mock::given(path("/api/v2/torrents/add"))
        .respond_with(ResponseTemplate::new(200).set_body_string("Ok."))
        .mount(&server)
        .await;

    let error = client
        .torrents()
        .add(AddTorrent::file("movie.torrent", b"d4:infoe".to_vec()))
        .await
        .unwrap_err();

    assert!(matches!(error, Error::Decode(_)));
}

#[tokio::test]
async fn stop_sends_the_hash() {
    let (server, client) = mock_client().await;
    Mock::given(method("POST"))
        .and(path("/api/v2/torrents/stop"))
        .and(body_string("hashes=abc"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;

    client.torrents().stop("abc").await.unwrap();
}

#[tokio::test]
async fn start_sends_every_hash() {
    let (server, client) = mock_client().await;
    Mock::given(method("POST"))
        .and(path("/api/v2/torrents/start"))
        .and(body_string("hashes=all"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;

    client.torrents().start(Hashes::All).await.unwrap();
}

#[tokio::test]
async fn delete_sends_the_hashes_and_the_files_flag() {
    let (server, client) = mock_client().await;
    Mock::given(method("POST"))
        .and(path("/api/v2/torrents/delete"))
        .and(body_string("hashes=a%7Cb&deleteFiles=true"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;
    let hashes: Hashes = ["a".to_string(), "b".to_string()].into_iter().collect();

    client.torrents().delete(hashes, true).await.unwrap();
}

#[tokio::test]
async fn post_errors_are_reported() {
    let (server, client) = mock_client().await;
    Mock::given(path("/api/v2/torrents/stop"))
        .respond_with(ResponseTemplate::new(403))
        .mount(&server)
        .await;

    let error = client.torrents().stop("abc").await.unwrap_err();

    assert!(matches!(error, Error::Unauthorized));
}
