#![recursion_limit = "1024"]
#![deny(rustdoc::broken_intra_doc_links, rust_2018_idioms)]
#![warn(
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    clippy::explicit_iter_loop,
    clippy::use_self,
    clippy::clone_on_ref_ptr,
    clippy::future_not_send
)]

//! # influxdb2
//!
//! This is a Rust client to InfluxDB using the [2.0 API][2api].
//!
//! [2api]: https://v2.docs.influxdata.com/v2.0/reference/api/
//!
//! This project is a fork from the
//! https://github.com/influxdata/influxdb_iox/tree/main/influxdb2_client project.
//! At the time of this writing, the query functionality of the influxdb2 client
//! from the official repository isn't working. So, I created this client to use
//! it in my project.
//!
//! ## Usage
//!
//! ### Querying
//!
//! ```rust
//! use chrono::{DateTime, FixedOffset};
//! use influxdb2::{Client, FromDataPoint};
//! use influxdb2::models::Query;
//!
//! #[derive(Debug, FromDataPoint)]
//! pub struct StockPrice {
//!     ticker: String,
//!     value: f64,
//!     time: DateTime<FixedOffset>,
//! }
//!
//! impl Default for StockPrice {
//!     fn default() -> Self {
//!         Self {
//!             ticker: "".to_string(),
//!             value: 0_f64,
//!             time: chrono::MIN_DATETIME.with_timezone(&chrono::FixedOffset::east(7 * 3600)),
//!         }
//!     }
//! }
//!
//! fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     let host = std::env::var("INFLUXDB_HOST").unwrap();
//!     let org = std::env::var("INFLUXDB_ORG").unwrap();
//!     let token = std::env::var("INFLUXDB_TOKEN").unwrap();
//!     let client = Client::new(host, org, token);
//!
//!     let qs = format!("from(bucket: \"stock-prices\")
//!         |> range(start: -1w)
//!         |> filter(fn: (r) => r.ticker == \"{}\")
//!         |> last()
//!     ", "AAPL");
//!     let query = Query::new(qs.to_string());
//!     let res: Vec<StockPrice> = client.query::<StockPrice>(Some(query))
//!         ?;
//!     println!("{:?}", res);
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Writing
//!
//! ```rust
//! fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     use influxdb2::models::DataPoint;
//!     use influxdb2::Client;
//!
//!     let host = std::env::var("INFLUXDB_HOST").unwrap();
//!     let org = std::env::var("INFLUXDB_ORG").unwrap();
//!     let token = std::env::var("INFLUXDB_TOKEN").unwrap();
//!     let bucket = "bucket";
//!     let client = Client::new(host, org, token);
//!     
//!     let points = vec![
//!         DataPoint::builder("cpu")
//!             .tag("host", "server01")
//!             .field("usage", 0.5)
//!             .build()?,
//!         DataPoint::builder("cpu")
//!             .tag("host", "server01")
//!             .tag("region", "us-west")
//!             .field("usage", 0.87)
//!             .build()?,
//!     ];
//!                                                             
//!     client.write(bucket, points)?;
//!     
//!     Ok(())
//! }
//! ```

use std::io;
use secrecy::{ExposeSecret, Secret};
use snafu::{ResultExt, Snafu};
use serde::Serialize;
use ureq::http::uri::{InvalidUriParts};
use ureq::http::{StatusCode, Uri};
use ureq::typestate::{WithBody, WithoutBody};
use ureq::RequestBuilder;

/// Errors that occur while making requests to the Influx server.
#[derive(Debug, Snafu)]
pub enum RequestError {
    /// failed to serialise the request query parameters
    UriBuilding {
        source: serde_urlencoded::ser::Error,
    },
    /// failed to build the request url
    RequestBuilding {
        source: InvalidUriParts,
    },
    /// While building the request body encountered an IO error
    BodyBuilding {
        source: io::Error
    },
    /// While making a request to the Influx server, the underlying `reqwest`
    /// library returned an error that was not an HTTP 400 or 500.
    #[snafu(display("Error while processing the HTTP request: {}", source))]
    UreqProcessing {
        /// The underlying error object from `reqwest`.
        source: ureq::Error,
    },
    /// The underlying `reqwest` library returned an HTTP error with code 400
    /// (meaning a client error) or 500 (meaning a server error).
    #[snafu(display("HTTP request returned an error: {}, `{}`", status, text))]
    Http {
        /// The `StatusCode` returned from the request
        status: StatusCode,
        /// Any text data returned from the request
        text: String,
    },

    /// While serializing data as JSON to send in a request, the underlying
    /// `serde_json` library returned an error.
    #[snafu(display("Error while serializing to JSON: {}", source))]
    Serializing {
        /// The underlying error object from `serde_json`.
        source: serde_json::error::Error,
    },

    /// While deserializing response from the Influx server, the underlying
    /// parsing library returned an error.
    #[snafu(display("Error while parsing response: {}", text))]
    Deserializing {
        /// Error description.
        text: String,
    },
}

/// Client to a server supporting the InfluxData 2.0 API.
#[derive(Debug, Clone)]
pub struct Client {
    /// The base URL this client sends requests to
    pub base: Uri,
    /// The organization tied to this client
    pub org: String,
    auth_header: Option<Secret<String>>,
}

impl Client {
    /// Create a new client pointing to the URL specified in
    /// `protocol://server:port` format and using the specified token for
    /// authorization.
    ///
    /// # Example
    ///
    /// ```
    /// let client = influxdb2::Client::new("http://localhost:8888", "org", "my-token");
    /// ```
    pub fn new(
        url: impl Into<String>,
        org: impl Into<String>,
        auth_token: impl Into<String>,
    ) -> Self {
        // unwrap is used to maintain backwards compatibility
        // panicking was present earlier as well inside of reqwest
        ClientBuilder::new(url, org, auth_token).build().unwrap()
    }

    /// Consolidate common request building code
    fn with_auth<Any>(&self, mut req: RequestBuilder<Any>) -> RequestBuilder<Any> {
        if let Some(auth) = &self.auth_header {
            req = req.header("Authorization", auth.expose_secret());
        }
        req
    }

    fn get(&self, url: Uri) -> RequestBuilder<WithoutBody> {
        self.with_auth(ureq::get(url))
    }

    fn post(&self, url: Uri) -> RequestBuilder<WithBody> {
        self.with_auth(ureq::post(url))
    }

    fn patch(&self, url: Uri) -> RequestBuilder<WithBody> {
        self.with_auth(ureq::patch(url))
    }

    fn delete_req(&self, url: Uri) -> RequestBuilder<WithoutBody> {
        self.with_auth(ureq::delete(url))
    }

    /// Join base Url of the client to target API endpoint into valid Url
    fn url(&self, mut endpoint: &str) -> Result<Uri, RequestError> {
        let mut parts = self.base.clone().into_parts();
        let endpoint = if endpoint.starts_with('/') {
            endpoint.to_string()
        } else {
            format!("/{endpoint}")
        };
        parts.path_and_query = Some(endpoint.parse().unwrap());
        Uri::from_parts(parts).context(RequestBuilding)
    }
    fn url_with_params(&self, endpoint: &str, query: impl Serialize) -> Result<Uri, RequestError> {
        let mut parts = self.base.clone().into_parts();
        let query = serde_urlencoded::to_string(query).context(UriBuilding)?;
        parts.path_and_query = Some(format!("{endpoint}?{query}").parse().unwrap());
        Uri::from_parts(parts).context(RequestBuilding)
    }
}

/// Errors that occur when building the client
#[derive(Debug, Snafu)]
pub enum BuildError {
    /// While constructing the reqwest client an error occurred
    #[snafu(display("Error while building the client: {}", source))]
    UreqClientError {
        /// Reqwest internal error
        source: ureq::Error,
    },
}
/// ClientBuilder builds the `Client`
#[derive(Debug)]
pub struct ClientBuilder {
    /// The base URL this client sends requests to
    pub base: Uri,
    /// The organization tied to this client
    pub org: String,
    auth_header: Option<Secret<String>>,
}

impl ClientBuilder {
    /// Construct a new `ClientBuilder`.
    pub fn new(
        url: impl Into<String>,
        org: impl Into<String>,
        auth_token: impl Into<String>,
    ) -> Self {
        let token = auth_token.into();
        let auth_header = if token.is_empty() {
            None
        } else {
            Some(format!("Token {}", token).into())
        };

        let url: String = url.into();
        let url = url.strip_suffix("/").unwrap_or(&url).to_string();
        let base = url
            .parse()
            .unwrap_or_else(|_| panic!("Invalid url was provided: {}", &url));

        Self {
            base,
            org: org.into(),
            auth_header,
        }
    }

    /// Build returns the influx client
    pub fn build(self) -> Result<Client, BuildError> {
        Ok(Client {
            base: self.base,
            org: self.org,
            auth_header: self.auth_header,
        })
    }
}

pub mod common;

pub mod api;
pub mod models;
pub mod writable;

// Re-exports
pub use influxdb2_derive::FromDataPoint;
pub use influxdb2_structmap::FromMap;

#[cfg(test)]
mod tests {
    use crate::Client;

    #[test]
    fn url_invalid_panic() {
        let result = std::panic::catch_unwind(|| Client::new("\\/3242/23", "some-org", "some-token"));
        assert!(result.is_err());
    }

    #[test]
    /// Reproduction of https://github.com/aprimadi/influxdb2/issues/6
    fn url_ignores_double_slashes() {
        let base = "http://influxdb.com/";
        let client = Client::new(base, "some-org", "some-token");

        assert_eq!(format!("{}api/v2/write", base), client.url("/api/v2/write").unwrap().to_string());

        assert_eq!(client.url("/api/v2/write").unwrap(), client.url("api/v2/write").unwrap());
    }
}
