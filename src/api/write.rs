//! Write API

use crate::models::WriteDataPoint;
use crate::{BodyBuilding, Client, Http, RequestError, UreqProcessing};

use bytes::BufMut;
use snafu::ResultExt;
use std::io::Write;
use ureq::http::{HeaderName, HeaderValue, StatusCode};
use ureq::{AsSendBody, Body};

impl Client {
    /// Write line protocol data to the specified organization and bucket.
    /// This method writes with default timestamp precision (nanoseconds).
    /// Use write_line_protocol_with_precision if you want to write with a different precision.
    pub fn write_line_protocol(
        &self,
        org: &str,
        bucket: &str,
        body: impl AsSendBody,
    ) -> Result<(), RequestError> {
        self.write_line_protocol_with_precision(org, bucket, body, TimestampPrecision::Nanoseconds)
    }

    /// Write line protocol data to the specified organization and bucket.
    pub fn write_line_protocol_with_precision(
        &self,
        org: &str,
        bucket: &str,
        body: impl AsSendBody,
        precision: TimestampPrecision,
    ) -> Result<(), RequestError> {
        self.write_line_protocol_with_precision_headers(
            org,
            bucket,
            body,
            precision,
            [],
        )
    }

    fn write_line_protocol_with_precision_headers(
        &self,
        org: &str,
        bucket: &str,
        body: impl AsSendBody,
        precision: TimestampPrecision,
        headers: impl IntoIterator<Item = (HeaderName, HeaderValue)>,
    ) -> Result<(), RequestError> {
        let write_url = self.url("/api/v2/write")?;

        let mut request = self.post(write_url);
        for (name, value) in headers {
            request = request.header(name, value);
        }
        let response = request
            .query_pairs([
                ("bucket", bucket),
                ("org", org),
                ("precision", precision.api_short_name()),
            ])
            .send(body)
            .context(UreqProcessing)?;

        if response.status() != StatusCode::NO_CONTENT {
            let status = response.status();
            let text = response.into_body().read_to_string().context(UreqProcessing)?;
            Http { status, text }.fail()?;
        }

        Ok(())
    }

    /// Write a `Stream` of `DataPoint`s to the specified bucket.
    ///
    /// This method writes with default timestamp precision (nanoseconds).
    /// Use write_with_precision if you want to write with a different precision.
    pub fn write(
        &self,
        bucket: &str,
        body: impl IntoIterator<Item = impl WriteDataPoint> + Send + Sync + 'static,
    ) -> Result<(), RequestError> {
        self.write_with_precision(bucket, body, TimestampPrecision::Nanoseconds)

    }

    /// Write a `Stream` of `DataPoint`s to the specified organization and
    /// bucket.
    pub fn write_with_precision(
        &self,
        bucket: &str,
        body: impl IntoIterator<Item = impl WriteDataPoint> + Send + Sync + 'static,
        timestamp_precision: TimestampPrecision,
    ) -> Result<(), RequestError> {
        let mut buffer = Vec::new();

        let mut w = (&mut buffer).writer();
        for point in body {
            point.write_data_point_to(&mut w).context(BodyBuilding)?;
        }
        w.flush().context(BodyBuilding)?;

        let body = Body::builder().data(buffer);

        self.write_line_protocol_with_precision(&self.org, bucket, body, timestamp_precision)

    }
}

/// Possible timestamp precisions.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TimestampPrecision {
    /// Seconds timestamp precision
    Seconds,
    /// Milliseconds timestamp precision
    Milliseconds,
    /// Microseconds timestamp precision
    Microseconds,
    /// Nanoseconds timestamp precision
    Nanoseconds,
}

impl TimestampPrecision {
    fn api_short_name(&self) -> &str {
        match self {
            Self::Seconds => "s",
            Self::Milliseconds => "ms",
            Self::Microseconds => "us",
            Self::Nanoseconds => "ns",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::DataPoint;
    use mockito::mock;

    fn writing_points() {
        let org = "some-org";
        let bucket = "some-bucket";
        let token = "some-token";

        let mock_server = mock(
            "POST",
            format!("/api/v2/write?bucket={}&org={}&precision=ns", bucket, org).as_str(),
        )
        .match_header("Authorization", format!("Token {}", token).as_str())
        .match_body(
            "\
cpu,host=server01 usage=0.5
cpu,host=server01,region=us-west usage=0.87
",
        )
        .with_status(204)
        .create();

        let client = Client::new(mockito::server_url(), org, token);

        let points = vec![
            DataPoint::builder("cpu")
                .tag("host", "server01")
                .field("usage", 0.5)
                .build()
                .unwrap(),
            DataPoint::builder("cpu")
                .tag("host", "server01")
                .tag("region", "us-west")
                .field("usage", 0.87)
                .build()
                .unwrap(),
        ];

        // If the requests made are incorrect, Mockito returns status 501 and `write`
        // will return an error, which causes the test to fail here instead of
        // when we assert on mock_server. The error messages that Mockito
        // provides are much clearer for explaining why a test failed than just
        // that the server returned 501, so don't use `?` here.
        let result = client.write(bucket, points);
        mock_server.assert();
        assert!(result.is_ok());
    }

    fn writing_points_with_precision() {
        let org = "some-org";
        let bucket = "some-bucket";
        let token = "some-token";

        let mock_server = mock(
            "POST",
            format!("/api/v2/write?bucket={}&org={}&precision=s", bucket, org).as_str(),
        )
        .match_header("Authorization", format!("Token {}", token).as_str())
        .match_body(
            "\
cpu,host=server01 usage=0.5 1671095854
",
        )
        .with_status(204)
        .create();

        let client = Client::new(mockito::server_url(), org, token);

        let point = DataPoint::builder("cpu")
            .tag("host", "server01")
            .field("usage", 0.5)
            .timestamp(1671095854)
            .build()
            .unwrap();
        let points = vec![point];

        // If the requests made are incorrect, Mockito returns status 501 and `write`
        // will return an error, which causes the test to fail here instead of
        // when we assert on mock_server. The error messages that Mockito
        // provides are much clearer for explaining why a test failed than just
        // that the server returned 501, so don't use `?` here.
        let result = client
            .write_with_precision(bucket, points, TimestampPrecision::Seconds)
            ;
        mock_server.assert();
        assert!(result.is_ok());
    }

    fn status_code_correctly_interpreted() {
        let org = "org";
        let token = "token";
        let bucket = "bucket";

        let make_mock_server = |status| {
            mock(
                "POST",
                format!("/api/v2/write?bucket={}&org={}&precision=ns", bucket, org).as_str(),
            )
            .with_status(status)
            .create()
        };

        let write_with_status = |status| {
            let mock_server = make_mock_server(status);
            let client = Client::new(mockito::server_url(), org, token);
            let points: Vec<DataPoint> = vec![];
            let res = client.write(bucket, points);
            mock_server.assert();
            res
        };

        // success status
        assert!(write_with_status(204).is_ok());

        // failing status
        for status in [200, 201, 400, 401, 404, 413, 429, 500, 503] {
            assert!(write_with_status(status).is_err());
        }
    }
}
