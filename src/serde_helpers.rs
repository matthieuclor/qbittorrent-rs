use std::time::Duration;

use serde::{Deserialize, Deserializer};

const UNKNOWN_ETA: u64 = 8_640_000;

pub(crate) fn tags<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<String>, D::Error> {
    let tags = String::deserialize(deserializer)?;
    Ok(tags
        .split(',')
        .map(str::trim)
        .filter(|tag| !tag.is_empty())
        .map(str::to_string)
        .collect())
}

pub(crate) fn eta<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Duration>, D::Error> {
    let seconds = u64::deserialize(deserializer)?;
    Ok((seconds < UNKNOWN_ETA).then(|| Duration::from_secs(seconds)))
}

pub(crate) fn timestamp<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<i64>, D::Error> {
    let timestamp = i64::deserialize(deserializer)?;
    Ok((timestamp > 0).then_some(timestamp))
}
