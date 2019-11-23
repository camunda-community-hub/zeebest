use std::path::PathBuf;
use std::time::SystemTime;
use crate::Error;
use dirs::home_dir;

const CAMUNDA_CLOUD_TOKEN_NAME: &str = "cloud.token";
const CAMUNDA_CLOUD_DIR: &str = ".zeebe";

#[derive(Clone, Serialize, Debug, Default)]
pub struct AccessRequest {
    client_id: String,
    client_secret: String,
    audience: String,
    grant_type: String,
}

impl AccessRequest {
    pub fn new(client_id: String, client_secret: String, audience: String) -> Self {
        Self {
            client_id,
            client_secret,
            audience,
            grant_type: "client_credentials".to_string(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct AccessResponse {
    pub access_token: Option<String>,
    pub expires_in: Option<u64>,
    pub token_type: Option<String>,
    pub scope: Option<String>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct CloudToken {
    #[serde(rename = "Token")]
    pub token: String,
    #[serde(rename = "DueDate")]
    pub due_date: u128,
}

impl CloudToken {
    pub fn new(token: String, due_date: u128) -> Self {
        Self {
            token,
            due_date,
        }
    }

    /// Returns true if due date is less than now-utc
    pub fn is_overdue(&self) -> bool {
        let due_date = self.due_date as u128;
        let since_the_epoch = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let current_time_in_millis = since_the_epoch.as_millis();
        due_date <= current_time_in_millis
    }
}

pub struct CamundaCloudTokenProvider {
    access_request: AccessRequest,
    cloud_token_path: PathBuf,
}

impl CamundaCloudTokenProvider {
    pub fn new(client_id: String, client_secret: String, audience: String, custom_cloud_token_path: Option<PathBuf>) -> Self {
        let cloud_token_path: PathBuf = custom_cloud_token_path.unwrap_or_else(|| {
            let mut cloud_token_path = home_dir().unwrap();
            cloud_token_path.push(CAMUNDA_CLOUD_DIR);
            cloud_token_path.push(CAMUNDA_CLOUD_TOKEN_NAME);
            cloud_token_path
        });
        Self {
            access_request: AccessRequest::new(client_id, client_secret, audience),
            cloud_token_path,
        }
    }

    async fn authenticate(&self) -> Result<AccessResponse, Error> {
        let request = surf::post("https://login.cloud.camunda.io/oauth/token")
            .body_json(&self.access_request)?;
        let result: AccessResponse = request.recv_json().await?;
        Ok(result)
    }

    fn response_to_cloud_token(&self, access_response: AccessResponse) -> Result<CloudToken, Error> {
        let AccessResponse { access_token, expires_in, .. } = access_response;
        let cloud_token = match (access_token, expires_in) {
            (Some(access_token), Some(expires_in)) => {
                let expires_in_millis = (expires_in as u128) * 1000;
                let since_the_epoch = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap();
                let due_date = since_the_epoch.as_millis() + expires_in_millis;
                Ok(CloudToken::new(access_token, due_date))
            },
            _ => Err(Error::InvalidCloudToken)
        }?;
        Ok(cloud_token)
    }

    async fn persist_token(&self, cloud_token: &CloudToken) -> Result<(), Error> {
        use async_std::fs::File;
        use async_std::prelude::*;
        let json = serde_json::to_string(cloud_token)?;
        let path = self.cloud_token_path.as_path();
        let mut file = File::create(path).await?;
        file.write_all(json.as_bytes()).await?;
        Ok(())
    }

    pub async fn get_token(&self) -> Result<CloudToken, Error> {
        let path = self.cloud_token_path.as_path();
        let cloud_token = match async_std::fs::metadata(path).await {
            Ok(metadata) => {
                if !metadata.is_file() {
                    let access_response = self.authenticate().await?;
                    let cloud_token = self.response_to_cloud_token(access_response)?;
                    self.persist_token(&cloud_token).await?;
                    cloud_token
                }
                else {
                    let json = async_std::fs::read_to_string(path).await?;
                    let mut cloud_token: CloudToken = serde_json::from_str(&json)?;
                    if cloud_token.is_overdue() {
                        // authenticate again and get new token
                        let access_response = self.authenticate().await?;
                        cloud_token = self.response_to_cloud_token(access_response)?;
                        self.persist_token(&cloud_token).await?;
                    }
                    cloud_token
                }
            },
            Err(_e) => {
                let access_response = self.authenticate().await?;
                let cloud_token = self.response_to_cloud_token(access_response)?;
                self.persist_token(&cloud_token).await?;
                cloud_token
            },
        };

        Ok(cloud_token)
    }
}
