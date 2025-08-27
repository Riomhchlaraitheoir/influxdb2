//! Buckets API

use serde::{Deserialize, Serialize};
use snafu::ResultExt;

use crate::models::{Buckets, PostBucketRequest};
use crate::{Client, Http, RequestError, UreqProcessing};

impl Client {
    /// List all buckets matching specified parameters
    pub fn list_buckets(
        &self,
        request: Option<ListBucketsRequest>,
    ) -> Result<Buckets, RequestError> {
        let url = self.url_with_params("/api/v2/buckets", request)?;

        let response = self
            .get(url)
            .call()
            .context(UreqProcessing)?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.into_body().read_to_string().context(UreqProcessing)?;
            let res = Http { status, text }.fail();
            return res;
        }

        let res = response
            .into_body()
            .read_json::<Buckets>()
            .context(UreqProcessing)?;

        Ok(res)
    }

    /// Create a new bucket in the organization specified by the 16-digit
    /// hexadecimal `org_id` and with the bucket name `bucket`.
    pub fn create_bucket(
        &self,
        post_bucket_request: Option<PostBucketRequest>,
    ) -> Result<(), RequestError> {
        let create_bucket_url = self.url("/api/v2/buckets")?;

        let response = self
            .post(create_bucket_url)
            .send_json(post_bucket_request.unwrap_or_default())
            .context(UreqProcessing)?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.into_body().read_to_string().context(UreqProcessing)?;
            Http { status, text }.fail()?;
        }

        Ok(())
    }

    /// Delete a bucket specified by bucket id.
    pub fn delete_bucket(&self, bucket_id: &str) -> Result<(), RequestError> {
        let url = self.url(&format!("/api/v2/buckets/{}", bucket_id))?;

        let response = self
            .delete_req(url)
            .call()
            .context(UreqProcessing)?;
        if !response.status().is_success() {
            let status = response.status();
            let text = response.into_body().read_to_string().context(UreqProcessing)?;
            Http { status, text }.fail()?;
        }
        Ok(())
    }
}

/// Request for list buckets API
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListBucketsRequest {
    /// The last bucket ID from which to seek from (but not including). This
    /// is to be used instead of `offset`.
    pub after: Option<String>,
    /// Only returns buckets with a specific ID.
    pub id: Option<String>,
    /// Number of buckets to return. Default: 20. Valid values: [1..100]
    pub limit: Option<u8>,
    /// Only returns buckets with a specific name.
    pub name: Option<String>,
    /// Offset from which to return buckets.
    pub offset: Option<u64>,
    /// The name of the organization.
    pub org: Option<String>,
    #[serde(rename = "orgID")]
    /// The organization ID.
    pub org_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::mock;

    fn setup() -> (Client, String, String) {
        let org_id = "0000111100001111".to_string();
        let token = "some-token".to_string();
        let client = Client::new(mockito::server_url(), &org_id, token.clone());

        (client, org_id, token)
    }

    fn create_bucket() {
        let (client, org_id, token) = setup();

        let bucket = "some-bucket".to_string();

        let mock_server = mock("POST", "/api/v2/buckets")
            .match_header("Authorization", format!("Token {}", token).as_str())
            .match_body(
                format!(
                    r#"{{"orgID":"{}","name":"{}","retentionRules":[]}}"#,
                    org_id, bucket
                )
                .as_str(),
            )
            .create();

        let _result = client
            .create_bucket(Some(PostBucketRequest::new(org_id, bucket)))
            ;

        mock_server.assert();
    }

    fn list_buckets_with_params() {
        let (client, _, token) = setup();

        let limit = 1;
        let bucket = "some-bucket".to_string();

        let mock_server = mock(
            "GET",
            format!("/api/v2/buckets?limit={limit}&name={bucket}").as_str(),
        )
        .match_header("Authorization", format!("Token {}", token).as_str())
        .create();

        let request = ListBucketsRequest {
            limit: Some(limit),
            name: Some(bucket),
            ..ListBucketsRequest::default()
        };

        let _result = client.list_buckets(Some(request));

        mock_server.assert();
    }

    fn list_buckets_without_params() {
        let (client, _, token) = setup();

        let mock_server = mock("GET", "/api/v2/buckets")
            .match_header("Authorization", format!("Token {}", token).as_str())
            .create();

        let _result = client.list_buckets(None);

        mock_server.assert();
    }
}
