use std::time::Duration;

use reqwest::{
    Method,
    multipart::{Form, Part},
};
use serde::{Deserialize, Serialize};

use crate::{Client, Error, serde_helpers};

#[derive(Debug, Clone, Copy)]
pub struct TorrentsApi<'a> {
    client: &'a Client,
}

impl Client {
    pub fn torrents(&self) -> TorrentsApi<'_> {
        TorrentsApi { client: self }
    }
}

impl TorrentsApi<'_> {
    pub async fn list(&self, query: &TorrentQuery) -> Result<Vec<Torrent>, Error> {
        self.client.get_json("torrents/info", query).await
    }

    pub async fn add(&self, torrent: AddTorrent) -> Result<AddedTorrents, Error> {
        let request = self
            .client
            .request(Method::POST, "torrents/add")
            .multipart(torrent.into_form());
        let body = self.client.send(request).await?;
        serde_json::from_slice(&body).map_err(Error::Decode)
    }

    pub async fn stop(&self, hashes: impl Into<Hashes>) -> Result<(), Error> {
        let hashes = hashes.into().to_param();
        self.client
            .post_form("torrents/stop", &[("hashes", hashes)])
            .await
    }

    pub async fn start(&self, hashes: impl Into<Hashes>) -> Result<(), Error> {
        let hashes = hashes.into().to_param();
        self.client
            .post_form("torrents/start", &[("hashes", hashes)])
            .await
    }

    pub async fn delete(&self, hashes: impl Into<Hashes>, delete_files: bool) -> Result<(), Error> {
        let hashes = hashes.into().to_param();
        let delete_files = delete_files.to_string();
        self.client
            .post_form(
                "torrents/delete",
                &[("hashes", hashes), ("deleteFiles", delete_files)],
            )
            .await
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Hashes {
    All,
    Only(Vec<String>),
}

impl Hashes {
    fn to_param(&self) -> String {
        match self {
            Self::All => "all".to_string(),
            Self::Only(hashes) => hashes.join("|"),
        }
    }
}

impl From<&str> for Hashes {
    fn from(hash: &str) -> Self {
        Self::Only(vec![hash.to_string()])
    }
}

impl<S: Into<String>> FromIterator<S> for Hashes {
    fn from_iter<I: IntoIterator<Item = S>>(hashes: I) -> Self {
        Self::Only(hashes.into_iter().map(Into::into).collect())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Filter {
    All,
    Downloading,
    Seeding,
    Completed,
    Stopped,
    Running,
    Active,
    Inactive,
    Stalled,
    StalledUploading,
    StalledDownloading,
    Errored,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct TorrentQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    filter: Option<Filter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hashes: Option<String>,
}

impl TorrentQuery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn filter(mut self, filter: Filter) -> Self {
        self.filter = Some(filter);
        self
    }

    pub fn category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = Some(tag.into());
        self
    }

    pub fn hashes(mut self, hashes: impl Into<Hashes>) -> Self {
        self.hashes = Some(hashes.into().to_param());
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AddTorrent {
    file_name: String,
    data: Vec<u8>,
    save_path: Option<String>,
    category: Option<String>,
    tags: Vec<String>,
    stopped: bool,
}

impl AddTorrent {
    pub fn file(file_name: impl Into<String>, data: impl Into<Vec<u8>>) -> Self {
        Self {
            file_name: file_name.into(),
            data: data.into(),
            save_path: None,
            category: None,
            tags: Vec::new(),
            stopped: false,
        }
    }

    pub fn save_path(mut self, save_path: impl Into<String>) -> Self {
        self.save_path = Some(save_path.into());
        self
    }

    pub fn category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    pub fn stopped(mut self, stopped: bool) -> Self {
        self.stopped = stopped;
        self
    }

    fn into_form(self) -> Form {
        let mut form = Form::new()
            .part("torrents", Part::bytes(self.data).file_name(self.file_name))
            .text("stopped", self.stopped.to_string());
        if let Some(save_path) = self.save_path {
            form = form.text("savepath", save_path);
        }
        if let Some(category) = self.category {
            form = form.text("category", category);
        }
        if !self.tags.is_empty() {
            form = form.text("tags", self.tags.join(","));
        }
        form
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AddedTorrents {
    pub added_torrent_ids: Vec<String>,
    pub success_count: u32,
    pub failure_count: u32,
    pub pending_count: u32,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Torrent {
    pub hash: String,
    pub name: String,
    pub state: State,
    pub progress: f64,
    pub size: u64,
    pub total_size: u64,
    pub downloaded: u64,
    pub uploaded: u64,
    pub amount_left: u64,
    pub dlspeed: u64,
    pub upspeed: u64,
    #[serde(deserialize_with = "serde_helpers::eta")]
    pub eta: Option<Duration>,
    pub ratio: f64,
    pub num_seeds: u32,
    pub num_leechs: u32,
    pub save_path: String,
    pub content_path: String,
    pub category: String,
    #[serde(deserialize_with = "serde_helpers::tags")]
    pub tags: Vec<String>,
    pub added_on: i64,
    #[serde(deserialize_with = "serde_helpers::timestamp")]
    pub completion_on: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum State {
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "missingFiles")]
    MissingFiles,
    #[serde(rename = "uploading")]
    Uploading,
    #[serde(rename = "stoppedUP")]
    StoppedUp,
    #[serde(rename = "queuedUP")]
    QueuedUp,
    #[serde(rename = "stalledUP")]
    StalledUp,
    #[serde(rename = "checkingUP")]
    CheckingUp,
    #[serde(rename = "forcedUP")]
    ForcedUp,
    #[serde(rename = "allocating")]
    Allocating,
    #[serde(rename = "downloading")]
    Downloading,
    #[serde(rename = "metaDL")]
    MetaDl,
    #[serde(rename = "forcedMetaDL")]
    ForcedMetaDl,
    #[serde(rename = "stoppedDL")]
    StoppedDl,
    #[serde(rename = "queuedDL")]
    QueuedDl,
    #[serde(rename = "stalledDL")]
    StalledDl,
    #[serde(rename = "checkingDL")]
    CheckingDl,
    #[serde(rename = "forcedDL")]
    ForcedDl,
    #[serde(rename = "checkingResumeData")]
    CheckingResumeData,
    #[serde(rename = "moving")]
    Moving,
    #[serde(other)]
    Unknown,
}

impl State {
    pub fn is_complete(self) -> bool {
        matches!(
            self,
            Self::Uploading
                | Self::StoppedUp
                | Self::QueuedUp
                | Self::StalledUp
                | Self::CheckingUp
                | Self::ForcedUp
        )
    }
}
