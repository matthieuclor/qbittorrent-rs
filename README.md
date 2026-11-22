# qbittorrent-rs

A modern, asynchronous, and strongly typed Rust client for the [qBittorrent](https://www.qbittorrent.org/) WebUI API, designed for simplicity, performance, and ease of use.

## Features

- API key authentication (`Authorization: Bearer`), no login, no session cookie
- Application versions: `version`, `webapi_version`
- Torrents: list with filters, add from a `.torrent` file, stop, start, delete
- Typed torrent states (`stoppedDL`, `stalledUP`...), tags as a list, unknown ETA as `None`
- Configurable timeout and a single error type
- `reqwest` with `rustls`, no OpenSSL, and no `reqwest` type in the public API

Targets qBittorrent 5.x. Tested against qBittorrent 5.2.3 (WebUI API 2.15.1).

## Installation

The crate is not published on crates.io yet. Add it from Git:

```toml
[dependencies]
qbittorrent-rs = { git = "https://github.com/matthieuclor/qbittorrent-rs" }
```

or from a local checkout:

```bash
cargo add --path ../qbittorrent-rs
```

The client is async and needs a Tokio runtime.

## Authentication

In qBittorrent, open **Settings > WebUI**, enable the WebUI and generate an **API key**. The client sends it as `Authorization: Bearer <key>` on every request.

Never commit this key. Load it from the environment instead.

## Usage

```rust,no_run
use std::time::Duration;

use qbittorrent_rs::{AddTorrent, Client, Filter, Hashes, TorrentQuery};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new("http://localhost:8080", std::env::var("QBITTORRENT_API_KEY")?)
        .timeout(Duration::from_secs(10));

    println!("WebUI API {}", client.webapi_version().await?);

    let file = std::fs::read("debian.torrent")?;
    let added = client
        .torrents()
        .add(
            AddTorrent::file("debian.torrent", file)
                .save_path("/downloads")
                .tag("linux")
                .stopped(true),
        )
        .await?;
    let hash = &added.added_torrent_ids[0];

    client.torrents().start(hash.as_str()).await?;

    let query = TorrentQuery::new().tag("linux").filter(Filter::Downloading);
    for torrent in client.torrents().list(&query).await? {
        println!("{} {:.0}% {:?}", torrent.name, torrent.progress * 100.0, torrent.eta);
    }

    client.torrents().stop(Hashes::All).await?;
    client.torrents().delete(hash.as_str(), false).await?;

    Ok(())
}
```

## Errors

Every call returns `Result<T, qbittorrent_rs::Error>`:

| Variant | Cause |
| --- | --- |
| `Network` | Connection failure |
| `Timeout` | No response before the configured timeout |
| `Decode` | Unexpected response body |
| `Unauthorized` | Missing or invalid API key (HTTP 403) |
| `Api { status, message }` | Any other error status, for example 409 when the torrent already exists or 415 for an invalid `.torrent` file |

## Development

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo cov
cargo test --doc
```

`cargo cov` runs the tests with `cargo-llvm-cov` and fails under 100% line coverage.
