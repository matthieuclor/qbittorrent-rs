use crate::{Client, Error};

impl Client {
    pub async fn version(&self) -> Result<String, Error> {
        self.get_text("app/version").await
    }

    pub async fn webapi_version(&self) -> Result<String, Error> {
        self.get_text("app/webapiVersion").await
    }
}
