//! Authorizations (tokens) API.

use serde::{Deserialize, Serialize};
use snafu::ResultExt;

use crate::models::authorization::{Authorization, Authorizations, Status};
use crate::models::permission::Permission;
use crate::{Client, Http, RequestError, UreqProcessing};

impl Client {
    /// List all authorization matching specified parameters
    pub fn list_authorizations(
        &self,
        request: ListAuthorizationsRequest,
    ) -> Result<Authorizations, RequestError> {
        let url = self.url_with_params("/api/v2/authorizations", request)?;

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

        response.into_body().read_json().context(UreqProcessing)
    }

    /// Create a new authorization in the organization.
    pub fn create_authorization(
        &self,
        request: CreateAuthorizationRequest,
    ) -> Result<Authorization, RequestError> {
        let create_bucket_url = self.url("/api/v2/authorizations")?;

        let response = self
            .post(create_bucket_url)
            .send_json(request)
            .context(UreqProcessing)?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.into_body().read_to_string().context(UreqProcessing)?;
            let res = Http { status, text }.fail();
            return res;
        }

        response.into_body().read_json::<Authorization>()
            .context(UreqProcessing)
    }
}

/// Request for listing authorizations.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct ListAuthorizationsRequest {
    /// Only returns authorizations that belong to the specified organization.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub org: Option<String>,
    /// Only returns authorizations that belong to the specified organization ID.
    #[serde(rename = "orgID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub org_id: Option<String>,
    /// Only returns the authorization that match the token value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    /// Only returns the authorization scoped to the specified user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// Only returns the authorization scoped to the specified user ID.
    #[serde(rename = "userID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
}

/// Request for creating an authorization.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAuthorizationRequest {
    /// A description of the token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// An organization ID. Specifies the organization that owns the authorization.
    #[serde(rename = "orgID")]
    pub org_id: String,
    /// A list of permissions for an authorization (at least 1 required).
    pub permissions: Vec<Permission>,
    /// Status of the token after creation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<Status>,
    /// A user ID. Specifies the user that the authorization is scoped to.
    #[serde(rename = "userID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
}
