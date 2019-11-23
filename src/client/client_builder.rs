use tonic::codegen::http;
use crate::client::{ client::Client, camunda_cloud_token_provider::{ CloudToken, CamundaCloudTokenProvider } };
use crate::Error;
use tonic::transport::ClientTlsConfig;
use async_std::sync::Arc;

#[derive(Default)]
pub struct ClientBuilder {
    uri: Option<http::Uri>,
    token_provider: Option<CamundaCloudTokenProvider>,
}

impl ClientBuilder {
    pub async fn connect(&self) -> Result<Client, Error> {
        let uri: http::Uri = self.uri.clone().expect("URI is required");
        let mut endpoint = tonic::transport::Channel::builder(uri.clone());

        match uri.scheme_str() {
            Some("http") => {
            },
            Some("https") => {
                let domain_name = uri.host().unwrap().to_string();
                let mut tls_config = ClientTlsConfig::with_rustls().domain_name(domain_name).clone();
                endpoint.tls_config(&mut tls_config);
            },
            Some(s) => {
                return Err(Error::InvalidSchemeError(s.to_string()))
            }
            None => {
                return Err(Error::SchemeMissingError)
            },
        }

        if let Some(provider) = &self.token_provider {
            let token: CloudToken = provider.get_token().await?;
            endpoint.intercept_headers(move |headers| {
                let value = format!("Bearer {}", token.token).parse().unwrap();
                headers.insert("Authorization", value);
                println!("{:?}", headers);

            });
        }
        let internal_channel: tonic::transport::Channel = endpoint.connect().await?;
        let internal_client = crate::gateway::client::GatewayClient::new(internal_channel);
        let internal_client = Arc::new(async_std::sync::RwLock::new(internal_client));
        Ok(Client {
            internal_client
        })
    }

    pub fn camunda_cloud_token_provider(
        &mut self,
        token_provider: CamundaCloudTokenProvider,
    ) -> &mut Self {
        self.token_provider = Some(token_provider);
        self
    }

    pub fn uri(&mut self, uri: http::Uri) -> &mut Self {
        self.uri = Some(uri);
        self
    }
}