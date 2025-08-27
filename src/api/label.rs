//! Labels

use crate::models::{LabelCreateRequest, LabelResponse, LabelUpdate, LabelsResponse};
use crate::{Client, Http, RequestError, UreqProcessing};
use snafu::ResultExt;
use std::collections::HashMap;
use ureq::http::StatusCode;

impl Client {
    /// List all Labels
    pub fn labels(&self) -> Result<LabelsResponse, RequestError> {
        self.get_labels(None)
    }

    /// List all Labels by organization ID
    pub fn labels_by_org(&self, org_id: &str) -> Result<LabelsResponse, RequestError> {
        self.get_labels(Some(org_id))
    }

    fn get_labels(&self, org_id: Option<&str>) -> Result<LabelsResponse, RequestError> {
        let labels_url = self.url("/api/v2/labels")?;
        let mut request = self.get(labels_url);

        if let Some(id) = org_id {
            request = request.query("orgID", id);
        }

        let response = request.call().context(UreqProcessing)?;
        match response.status() {
            StatusCode::OK => Ok(response
                .into_body()
                .read_json::<LabelsResponse>()
                .context(UreqProcessing)?),
            status => {
                let text = response.into_body().read_to_string().context(UreqProcessing)?;
                Http { status, text }.fail()?
            }
        }
    }

    /// Retrieve a label by ID
    pub fn find_label(&self, label_id: &str) -> Result<LabelResponse, RequestError> {
        let labels_by_id_url = self.url(&format!("/api/v2/labels/{}", label_id))?;
        let response = self
            .get(labels_by_id_url)
            .call()
            .context(UreqProcessing)?;
        match response.status() {
            StatusCode::OK => Ok(response
                .into_body()
                .read_json::<LabelResponse>()
                .context(UreqProcessing)?),
            status => {
                let text = response.into_body().read_to_string().context(UreqProcessing)?;
                Http { status, text }.fail()?
            }
        }
    }

    /// Create a Label
    pub fn create_label(
        &self,
        org_id: &str,
        name: &str,
        properties: Option<HashMap<String, String>>,
    ) -> Result<LabelResponse, RequestError> {
        let create_label_url = self.url("/api/v2/labels")?;
        let body = LabelCreateRequest {
            org_id: org_id.into(),
            name: name.into(),
            properties,
        };
        let response = self
            .post(create_label_url)
            .send_json(body)
            .context(UreqProcessing)?;
        match response.status() {
            StatusCode::CREATED => Ok(response
                .into_body()
                .read_json::<LabelResponse>()
                .context(UreqProcessing)?),
            status => {
                let text = response.into_body().read_to_string().context(UreqProcessing)?;
                Http { status, text }.fail()?
            }
        }
    }

    /// Update a Label
    pub fn update_label(
        &self,
        name: Option<String>,
        properties: Option<HashMap<String, String>>,
        label_id: &str,
    ) -> Result<LabelResponse, RequestError> {
        let update_label_url = self.url(&format!("/api/v2/labels/{}", label_id))?;
        let body = LabelUpdate { name, properties };
        let response = self
            .patch(update_label_url)
            .send_json(body)
            .context(UreqProcessing)?;
        match response.status() {
            StatusCode::OK => Ok(response
                .into_body()
                .read_json::<LabelResponse>()
                .context(UreqProcessing)?),
            status => {
                let text = response.into_body().read_to_string().context(UreqProcessing)?;
                Http { status, text }.fail()?
            }
        }
    }

    /// Delete a Label
    pub fn delete_label(&self, label_id: &str) -> Result<(), RequestError> {
        let delete_label_url = self.url(&format!("/api/v2/labels/{}", label_id))?;
        let response = self
            .delete_req(delete_label_url)
            .call()
            .context(UreqProcessing)?;
        match response.status() {
            StatusCode::NO_CONTENT => Ok(()),
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

    const BASE_PATH: &str = "/api/v2/labels";

    fn labels() {
        let token = "some-token";

        let mock_server = mock("GET", BASE_PATH)
            .match_header("Authorization", format!("Token {}", token).as_str())
            .create();

        let client = Client::new(mockito::server_url(), "", token);

        let _result = client.labels();

        mock_server.assert();
    }

    fn labels_by_org() {
        let token = "some-token";
        let org_id = "some-org_id";

        let mock_server = mock("GET", format!("{}?orgID={}", BASE_PATH, org_id).as_str())
            .match_header("Authorization", format!("Token {}", token).as_str())
            .create();

        let client = Client::new(mockito::server_url(), "", token);

        let _result = client.labels_by_org(org_id);

        mock_server.assert();
    }

    fn find_label() {
        let token = "some-token";
        let label_id = "some-id";

        let mock_server = mock("GET", format!("{}/{}", BASE_PATH, label_id).as_str())
            .match_header("Authorization", format!("Token {}", token).as_str())
            .create();

        let client = Client::new(mockito::server_url(), "", token);

        let _result = client.find_label(label_id);

        mock_server.assert();
    }

    fn create_label() {
        let token = "some-token";
        let org_id = "some-org";
        let name = "some-user";
        let mut properties = HashMap::new();
        properties.insert("some-key".to_string(), "some-value".to_string());

        let mock_server = mock("POST", BASE_PATH)
            .match_header("Authorization", format!("Token {}", token).as_str())
            .match_body(
                format!(
                    r#"{{"orgID":"{}","name":"{}","properties":{{"some-key":"some-value"}}}}"#,
                    org_id, name
                )
                .as_str(),
            )
            .create();

        let client = Client::new(mockito::server_url(), org_id, token);

        let _result = client.create_label(org_id, name, Some(properties));

        mock_server.assert();
    }

    fn create_label_opt() {
        let token = "some-token";
        let org_id = "some-org_id";
        let name = "some-user";

        let mock_server = mock("POST", BASE_PATH)
            .match_header("Authorization", format!("Token {}", token).as_str())
            .match_body(format!(r#"{{"orgID":"{}","name":"{}"}}"#, org_id, name).as_str())
            .create();

        let client = Client::new(mockito::server_url(), org_id, token);

        let _result = client.create_label(org_id, name, None);

        mock_server.assert();
    }

    fn update_label() {
        let token = "some-token";
        let name = "some-user";
        let label_id = "some-label_id";
        let mut properties = HashMap::new();
        properties.insert("some-key".to_string(), "some-value".to_string());

        let mock_server = mock("PATCH", format!("{}/{}", BASE_PATH, label_id).as_str())
            .match_header("Authorization", format!("Token {}", token).as_str())
            .match_body(
                format!(
                    r#"{{"name":"{}","properties":{{"some-key":"some-value"}}}}"#,
                    name
                )
                .as_str(),
            )
            .create();

        let client = Client::new(mockito::server_url(), "", token);

        let _result = client
            .update_label(Some(name.to_string()), Some(properties), label_id)
            ;

        mock_server.assert();
    }

    fn update_label_opt() {
        let token = "some-token";
        let label_id = "some-label_id";

        let mock_server = mock("PATCH", format!("{}/{}", BASE_PATH, label_id).as_str())
            .match_header("Authorization", format!("Token {}", token).as_str())
            .match_body("{}")
            .create();

        let client = Client::new(mockito::server_url(), "", token);

        let _result = client.update_label(None, None, label_id);

        mock_server.assert();
    }

    fn delete_label() {
        let token = "some-token";
        let label_id = "some-label_id";

        let mock_server = mock("DELETE", format!("{}/{}", BASE_PATH, label_id).as_str())
            .match_header("Authorization", format!("Token {}", token).as_str())
            .create();

        let client = Client::new(mockito::server_url(), "", token);

        let _result = client.delete_label(label_id);

        mock_server.assert();
    }
}
