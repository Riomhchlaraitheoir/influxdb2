//! Delete API

use chrono::NaiveDateTime;
use snafu::ResultExt;

use crate::{Client, Http, RequestError, UreqProcessing};

impl Client {
    /// Delete data points from a bucket matching specified parameters.
    ///
    /// Usage:
    ///
    /// ```
    /// use chrono::NaiveDate;
    /// use influxdb2::Client;
    ///
    /// fn foo() {
    ///     let client = Client::new("some-host", "some-org", "some-token");
    ///     let start = NaiveDate::from_ymd(2020, 1, 1).and_hms(0, 0, 0);
    ///     let stop = NaiveDate::from_ymd(2020, 12, 31).and_hms(23, 59, 59);
    ///     let predicate = Some("_measurement=\"some-measurement\"".to_owned());
    ///     client.delete("some-bucket", start, stop, predicate).unwrap();
    /// }
    /// ```
    ///
    pub fn delete(
        &self,
        bucket: &str,
        start: NaiveDateTime,
        stop: NaiveDateTime,
        predicate: Option<String>,
    ) -> Result<(), RequestError> {
        let delete_url = self.url("/api/v2/delete")?;

        let body = serde_json
        ::json!({
            "start": start.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            "stop": stop.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            "predicate": predicate,
        })
        .to_string();

        let response = self
            .post(delete_url)
            .query_pairs([("bucket", bucket), ("org", &self.org)])
            .send(body)
            .context(UreqProcessing)?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.into_body().read_to_string().context(UreqProcessing)?;
            Http { status, text }.fail()?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use mockito::mock;

    fn delete_points() {
        let org = "some-org";
        let bucket = "some-bucket";
        let token = "some-token";

        let mock_server = mock(
                "POST",
                format!("/api/v2/delete?bucket={}&org={}", bucket, org).as_str(),
            )
            .match_header("Authorization", format!("Token {}", token).as_str())
            .match_body(
                "{\"predicate\":null,\"start\":\"2020-01-01T00:00:00Z\",\"stop\":\"2021-01-01T00:00:00Z\"}"
            )
            .create();

        let client = Client::new(mockito::server_url(), org, token);

        let start = NaiveDate::from_ymd(2020, 1, 1).and_hms(0, 0, 0);
        let stop = NaiveDate::from_ymd(2021, 1, 1).and_hms(0, 0, 0);
        let _result = client.delete(bucket, start, stop, None);

        mock_server.assert();
    }
}
