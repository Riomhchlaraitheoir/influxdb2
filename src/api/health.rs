//! Health
//!
//! Get health of an InfluxDB instance

use crate::models::HealthCheck;
use crate::{Client, Http, RequestError, UreqProcessing};
use snafu::ResultExt;
use ureq::http::StatusCode;

impl Client {
    /// Get health of an instance
    pub fn health(&self) -> Result<HealthCheck, RequestError> {
        let health_url = self.url("/health")?;
        let response = self
            .get(health_url)
            .call()
            .context(UreqProcessing)?;

        match response.status() {
            StatusCode::OK => Ok(response
                .into_body()
                .read_json::<HealthCheck>()
                .context(UreqProcessing)?),
            StatusCode::SERVICE_UNAVAILABLE => Ok(response
                .into_body()
                .read_json::<HealthCheck>()
                .context(UreqProcessing)?),
            status => {
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

    fn health() {
        let mock_server = mock("GET", "/health").create();

        let client = Client::new(mockito::server_url(), "", "");

        let _result = client.health();

        mock_server.assert();
    }
}
