#![doc = include_str!("../README.md")]

mod app;
mod client;
mod error;
mod serde_helpers;
mod torrents;

pub use client::Client;
pub use error::Error;
pub use torrents::{
    AddTorrent, AddedTorrents, Filter, Hashes, State, Torrent, TorrentQuery, TorrentsApi,
};
