//! Ready
//!
//! Check readiness of an InfluxDB instance at startup

use snafu::ResultExt;
use ureq::http::StatusCode;
use crate::{Client, Http, RequestError, UreqProcessing};

impl Client {
    /// Get the readiness of an instance at startup
    pub fn ready(&self) -> Result<bool, RequestError> {
        let ready_url = self.url("/ready")?;
        let response = self
            .get(ready_url)
            .call()
            .context(UreqProcessing)?;

        match response.status() {
            StatusCode::OK => Ok(true),
            _ => {
                let status = response.status();
                let text = response.into_body().read_to_string().context(UreqProcessing)?;
                Http { status, text }.fail()?
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::mock;

    fn ready() {
        let mock_server = mock("GET", "/ready").create();

        let client = Client::new(mockito::server_url(), "org", "");

        let _result = client.ready();

        mock_server.assert();
    }
}
