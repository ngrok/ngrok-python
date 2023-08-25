use std::str::FromStr;

use bytes::Bytes;
use ngrok::config::{
    OauthOptions,
    OidcOptions,
    Scheme,
};
use pyo3::{
    pymethods,
    types::PyByteArray,
    PyRefMut,
};

use crate::tunnel_builder::NgrokHttpTunnelBuilder;

#[pymethods]
#[allow(dead_code)]
impl NgrokHttpTunnelBuilder {
    /// The scheme that this edge should use.
    /// Defaults to [Scheme::HTTPS].
    pub fn scheme(self_: PyRefMut<Self>, scheme: String) -> PyRefMut<Self> {
        self_.set(|b| {
            b.scheme(
                Scheme::from_str(scheme.as_str())
                    .unwrap_or_else(|_| panic!("Unknown scheme: {scheme:?}")),
            )
        });
        self_
    }
    /// The domain to request for this edge.
    pub fn domain(self_: PyRefMut<Self>, domain: String) -> PyRefMut<Self> {
        self_.set(|b| b.domain(domain));
        self_
    }
    /// Certificates to use for client authentication at the ngrok edge.
    pub fn mutual_tlsca<'a>(
        self_: PyRefMut<'a, Self>,
        mutual_tlsca: &PyByteArray,
    ) -> PyRefMut<'a, Self> {
        self_.set(|b| b.mutual_tlsca(Bytes::from(mutual_tlsca.to_vec())));
        self_
    }
    /// Enable gzip compression for HTTP responses.
    pub fn compression(self_: PyRefMut<Self>) -> PyRefMut<Self> {
        self_.set(|b| b.compression());
        self_
    }
    /// Convert incoming websocket connections to TCP-like streams.
    pub fn websocket_tcp_conversion(self_: PyRefMut<Self>) -> PyRefMut<Self> {
        self_.set(|b| b.websocket_tcp_conversion());
        self_
    }
    /// Reject requests when 5XX responses exceed this ratio.
    /// Disabled when 0.
    pub fn circuit_breaker(self_: PyRefMut<Self>, circuit_breaker: f64) -> PyRefMut<Self> {
        self_.set(|b| b.circuit_breaker(circuit_breaker));
        self_
    }

    /// Adds a header to all requests to this edge.
    pub fn request_header(self_: PyRefMut<Self>, name: String, value: String) -> PyRefMut<Self> {
        self_.set(|b| b.request_header(name, value));
        self_
    }
    /// Adds a header to all responses coming from this edge.
    pub fn response_header(self_: PyRefMut<Self>, name: String, value: String) -> PyRefMut<Self> {
        self_.set(|b| b.response_header(name, value));
        self_
    }
    /// Removes a header from requests to this edge.
    pub fn remove_request_header(self_: PyRefMut<Self>, name: String) -> PyRefMut<Self> {
        self_.set(|b| b.remove_request_header(name));
        self_
    }
    /// Removes a header from responses from this edge.
    pub fn remove_response_header(self_: PyRefMut<Self>, name: String) -> PyRefMut<Self> {
        self_.set(|b| b.remove_response_header(name));
        self_
    }

    /// Credentials for basic authentication.
    /// If not called, basic authentication is disabled.
    pub fn basic_auth(self_: PyRefMut<Self>, username: String, password: String) -> PyRefMut<Self> {
        self_.set(|b| b.basic_auth(username, password));
        self_
    }

    /// OAuth configuration.
    /// If not called, OAuth is disabled.
    ///
    /// :param str provider: The name of the OAuth provider to use.
    /// :param list or None allow_emails: A list of email addresses to allow.
    /// :param list or None allow_domains: A list of domain names to allow.
    /// :param list or None scopes: A list of scopes.
    #[pyo3(text_signature = "(provider, allow_emails=None, allow_domains=None, scopes=None)")]
    pub fn oauth(
        self_: PyRefMut<Self>,
        provider: String,
        allow_emails: Option<Vec<String>>,
        allow_domains: Option<Vec<String>>,
        scopes: Option<Vec<String>>,
    ) -> PyRefMut<Self> {
        let mut oauth = OauthOptions::new(provider);
        if let Some(allow_emails) = allow_emails {
            for v in allow_emails.iter() {
                oauth = oauth.allow_email(v);
            }
        }
        if let Some(allow_domains) = allow_domains {
            for v in allow_domains.iter() {
                oauth = oauth.allow_domain(v);
            }
        }
        if let Some(scopes) = scopes {
            for v in scopes.iter() {
                oauth = oauth.scope(v);
            }
        }

        self_.set(|b| b.oauth(oauth));
        self_
    }

    /// OIDC configuration.
    /// If not called, OIDC is disabled.
    ///
    /// :param str issuer_url: The name of the OIDC issuer URL to use.
    /// :param str client_id: The OIDC client ID.
    /// :param str client_secret: The OIDC client secret.
    /// :param list or None allow_emails: A list of email addresses to allow.
    /// :param list or None allow_domains: A list of domain names to allow.
    /// :param list or None scopes: A list of scopes.
    #[pyo3(
        text_signature = "(issuer_url, client_id, client_secret, allow_emails=None, allow_domains=None, scopes=None)"
    )]
    pub fn oidc(
        self_: PyRefMut<Self>,
        issuer_url: String,
        client_id: String,
        client_secret: String,
        allow_emails: Option<Vec<String>>,
        allow_domains: Option<Vec<String>>,
        scopes: Option<Vec<String>>,
    ) -> PyRefMut<Self> {
        let mut oidc = OidcOptions::new(issuer_url, client_id, client_secret);
        if let Some(allow_emails) = allow_emails {
            for v in allow_emails.iter() {
                oidc = oidc.allow_email(v);
            }
        }
        if let Some(allow_domains) = allow_domains {
            for v in allow_domains.iter() {
                oidc = oidc.allow_domain(v);
            }
        }
        if let Some(scopes) = scopes {
            for v in scopes.iter() {
                oidc = oidc.scope(v);
            }
        }

        self_.set(|b| b.oidc(oidc));
        self_
    }

    /// WebhookVerification configuration.
    /// If not called, WebhookVerification is disabled.
    pub fn webhook_verification(
        self_: PyRefMut<Self>,
        provider: String,
        secret: String,
    ) -> PyRefMut<Self> {
        self_.set(|b| b.webhook_verification(provider, secret));
        self_
    }
}
