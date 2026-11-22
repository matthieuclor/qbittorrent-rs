use std::time::Duration;

use reqwest::{Method, RequestBuilder, StatusCode};
use serde::{Serialize, de::DeserializeOwned};

use crate::Error;

const TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Clone)]
pub struct Client {
    http: reqwest::Client,
    base_url: String,
    api_key: String,
    timeout: Duration,
}

impl Client {
    pub fn new(base_url: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.into().trim_end_matches('/').to_string(),
            api_key: api_key.into(),
            timeout: TIMEOUT,
        }
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub(crate) fn request(&self, method: Method, path: &str) -> RequestBuilder {
        self.http
            .request(method, format!("{}/api/v2/{path}", self.base_url))
            .bearer_auth(&self.api_key)
            .timeout(self.timeout)
    }

    pub(crate) async fn send(&self, request: RequestBuilder) -> Result<Vec<u8>, Error> {
        let response = request.send().await?;
        let status = response.status();
        let body = response.bytes().await?.to_vec();

        if status == StatusCode::FORBIDDEN {
            return Err(Error::Unauthorized);
        }
        if !status.is_success() {
            return Err(Error::Api {
                status: status.as_u16(),
                message: String::from_utf8_lossy(&body).into_owned(),
            });
        }
        Ok(body)
    }

    pub(crate) async fn get_text(&self, path: &str) -> Result<String, Error> {
        let body = self.send(self.request(Method::GET, path)).await?;
        Ok(String::from_utf8_lossy(&body).into_owned())
    }

    pub(crate) async fn get_json<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &(impl Serialize + ?Sized),
    ) -> Result<T, Error> {
        let body = self
            .send(self.request(Method::GET, path).query(query))
            .await?;
        serde_json::from_slice(&body).map_err(Error::Decode)
    }

    pub(crate) async fn post_form(
        &self,
        path: &str,
        form: &(impl Serialize + ?Sized),
    ) -> Result<(), Error> {
        self.send(self.request(Method::POST, path).form(form))
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_trims_the_trailing_slash_and_sets_the_default_timeout() {
        let client = Client::new("http://localhost:8080/", "key");

        assert_eq!(client.base_url, "http://localhost:8080");
        assert_eq!(client.timeout, TIMEOUT);
    }
}
