use crate::Error;
use httpbis::ClientTlsOption;
use std::sync::Arc;
use tls_api::TlsConnectorBuilder;
use tls_api_native_tls::TlsConnector;

pub struct TlsCertificate {
    certificate: tls_api::Certificate,
}

impl TlsCertificate {
    pub fn new(der_certificate_bytes: Vec<u8>) -> Self {
        TlsCertificate {
            certificate: tls_api::Certificate::from_der(der_certificate_bytes),
        }
    }

    pub fn into_tls_option(self, host: String) -> Result<ClientTlsOption<TlsConnector>, Error> {
        let mut builder = <TlsConnector as tls_api::TlsConnector>::builder()
            .map_err(|e| Error::TlsCertificateError(e))?;
        builder
            .add_root_certificate(self.certificate)
            .map_err(|e| Error::TlsCertificateError(e))?;
        builder
            .build()
            .map(|connector| ClientTlsOption::Tls(host, Arc::new(connector)))
            .map_err(|e| Error::TlsCertificateError(e))
    }
}
