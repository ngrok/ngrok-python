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

use crate::listener_builder::HttpListenerBuilder;

#[pymethods]
#[allow(dead_code)]
impl HttpListenerBuilder {
    /// The L7 protocol to use for this edge: "http1" or "http2".
    pub fn app_protocol(self_: PyRefMut<Self>, app_protocol: String) -> PyRefMut<Self> {
        self_.set(|b| {
            b.app_protocol(app_protocol);
        });
        self_
    }

    /// The scheme that this edge should use.
    /// "HTTPS" or "HTTP", defaults to "HTTPS".
    pub fn scheme(self_: PyRefMut<Self>, scheme: String) -> PyRefMut<Self> {
        self_.set(|b| {
            b.scheme(
                Scheme::from_str(scheme.as_str())
                    .unwrap_or_else(|_| panic!("Unknown scheme: {scheme:?}")),
            );
        });
        self_
    }
    /// The domain to request for this edge, any valid domain or hostname that you have
    /// previously registered with ngrok. If using a custom domain, this requires
    /// registering in the `ngrok dashboard`_ and setting a DNS CNAME value.
    ///
    /// .. _ngrok dashboard: https://dashboard.ngrok.com/cloud-edge/domains
    pub fn domain(self_: PyRefMut<Self>, domain: String) -> PyRefMut<Self> {
        self_.set(|b| {
            b.domain(domain);
        });
        self_
    }
    /// Certificates to use for client authentication at the ngrok edge.
    /// See `Mutual TLS`_ in the ngrok docs for additional details.
    ///
    /// .. _Mutual TLS: https://ngrok.com/docs/cloud-edge/modules/mutual-tls/
    pub fn mutual_tlsca<'a>(
        self_: PyRefMut<'a, Self>,
        mutual_tlsca: &PyByteArray,
    ) -> PyRefMut<'a, Self> {
        self_.set(|b| {
            b.mutual_tlsca(Bytes::from(mutual_tlsca.to_vec()));
        });
        self_
    }
    /// Enable gzip compression for HTTP responses.
    /// See `Compression`_ in the ngrok docs for additional details.
    ///
    /// .. _Compression: https://ngrok.com/docs/cloud-edge/modules/compression/
    pub fn compression(self_: PyRefMut<Self>) -> PyRefMut<Self> {
        self_.set(|b| {
            b.compression();
        });
        self_
    }
    /// Convert incoming websocket connections to TCP-like streams.
    pub fn websocket_tcp_conversion(self_: PyRefMut<Self>) -> PyRefMut<Self> {
        self_.set(|b| {
            b.websocket_tcp_conversion();
        });
        self_
    }
    /// Reject requests when 5XX responses exceed this ratio.
    /// Disabled when 0.
    /// See `Circuit Breaker`_ in the ngrok docs for additional details.
    ///
    /// .. _Circuit Breaker: https://ngrok.com/docs/cloud-edge/modules/circuit-breaker/
    pub fn circuit_breaker(self_: PyRefMut<Self>, circuit_breaker: f64) -> PyRefMut<Self> {
        self_.set(|b| {
            b.circuit_breaker(circuit_breaker);
        });
        self_
    }

    /// Adds a header to all requests to this edge.
    /// See `Request Headers`_ in the ngrok docs for additional details.
    ///
    /// .. _Request Headers: https://ngrok.com/docs/cloud-edge/modules/request-headers/
    pub fn request_header(self_: PyRefMut<Self>, name: String, value: String) -> PyRefMut<Self> {
        self_.set(|b| {
            b.request_header(name, value);
        });
        self_
    }
    /// Adds a header to all responses coming from this edge.
    /// See `Response Headers`_ in the ngrok docs for additional details.
    ///
    /// .. _Response Headers: https://ngrok.com/docs/cloud-edge/modules/response-headers/
    pub fn response_header(self_: PyRefMut<Self>, name: String, value: String) -> PyRefMut<Self> {
        self_.set(|b| {
            b.response_header(name, value);
        });
        self_
    }
    /// Removes a header from requests to this edge.
    /// See `Request Headers`_ in the ngrok docs for additional details.
    ///
    /// .. _Request Headers: https://ngrok.com/docs/cloud-edge/modules/request-headers/
    pub fn remove_request_header(self_: PyRefMut<Self>, name: String) -> PyRefMut<Self> {
        self_.set(|b| {
            b.remove_request_header(name);
        });
        self_
    }
    /// Removes a header from responses from this edge.
    /// See `Response Headers`_ in the ngrok docs for additional details.
    ///
    /// .. _Response Headers: https://ngrok.com/docs/cloud-edge/modules/response-headers/
    pub fn remove_response_header(self_: PyRefMut<Self>, name: String) -> PyRefMut<Self> {
        self_.set(|b| {
            b.remove_response_header(name);
        });
        self_
    }

    /// Credentials for basic authentication.
    /// If not called, basic authentication is disabled.
    pub fn basic_auth(self_: PyRefMut<Self>, username: String, password: String) -> PyRefMut<Self> {
        self_.set(|b| {
            b.basic_auth(username, password);
        });
        self_
    }

    /// A set of regular expressions used to match User-Agents that will be allowed.
    /// On request, the User Agent Filter module will check the incoming User-Agent header value
    /// against the list of defined allow and deny regular expression rules.
    /// See `User Agent Filter`_ in the ngrok docs for additional details.
    ///
    /// .. _User Agent Filter: https://ngrok.com/docs/cloud-edge/modules/user-agent-filter/
    pub fn allow_user_agent(self_: PyRefMut<Self>, regex: String) -> PyRefMut<Self> {
        self_.set(|b| {
            b.allow_user_agent(regex);
        });
        self_
    }
    /// A set of regular expressions used to match User-Agents that will be denied.
    /// On request, the User Agent Filter module will check the incoming User-Agent header value
    /// against the list of defined allow and deny regular expression rules.
    /// See `User Agent Filter`_ in the ngrok docs for additional details.
    ///
    /// .. _User Agent Filter: https://ngrok.com/docs/cloud-edge/modules/user-agent-filter/
    pub fn deny_user_agent(self_: PyRefMut<Self>, regex: String) -> PyRefMut<Self> {
        self_.set(|b| {
            b.deny_user_agent(regex);
        });
        self_
    }

    /// OAuth configuration.
    /// If not called, OAuth is disabled.
    /// See `OAuth`_ in the ngrok docs for additional details.
    ///
    /// .. _OAuth: https://ngrok.com/docs/cloud-edge/modules/oauth/
    ///
    /// :param str provider: The name of the OAuth provider to use.
    /// :param list or None allow_emails: A list of email addresses to allow.
    /// :param list or None allow_domains: A list of domain names to allow.
    /// :param list or None scopes: A list of scopes.
    /// :param str or None client_id: The optional OAuth client ID, required for scopes.
    /// :param str or None client_secret: The optional OAuth client secret, required for scopes.
    #[pyo3(
        text_signature = "(provider, allow_emails=None, allow_domains=None, scopes=None, client_id=None, client_secret=None)"
    )]
    pub fn oauth(
        self_: PyRefMut<Self>,
        provider: String,
        allow_emails: Option<Vec<String>>,
        allow_domains: Option<Vec<String>>,
        scopes: Option<Vec<String>>,
        client_id: Option<String>,
        client_secret: Option<String>,
    ) -> PyRefMut<Self> {
        let mut oauth = OauthOptions::new(provider);
        if let Some(allow_emails) = allow_emails {
            for v in allow_emails.iter() {
                oauth.allow_email(v);
            }
        }
        if let Some(allow_domains) = allow_domains {
            for v in allow_domains.iter() {
                oauth.allow_domain(v);
            }
        }
        if let Some(scopes) = scopes {
            for v in scopes.iter() {
                oauth.scope(v);
            }
        }
        if let Some(client_id) = client_id {
            oauth.client_id(client_id);
        }
        if let Some(client_secret) = client_secret {
            oauth.client_secret(client_secret);
        }

        self_.set(|b| {
            b.oauth(oauth);
        });
        self_
    }

    /// OIDC configuration.
    /// If not called, OIDC is disabled.
    /// See `OpenID Connect`_ in the ngrok docs for additional details.
    ///
    /// .. _OpenID Connect: https://ngrok.com/docs/cloud-edge/modules/openid-connect/
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
                oidc.allow_email(v);
            }
        }
        if let Some(allow_domains) = allow_domains {
            for v in allow_domains.iter() {
                oidc.allow_domain(v);
            }
        }
        if let Some(scopes) = scopes {
            for v in scopes.iter() {
                oidc.scope(v);
            }
        }

        self_.set(|b| {
            b.oidc(oidc);
        });
        self_
    }

    /// WebhookVerification configuration.
    /// If not called, WebhookVerification is disabled.
    /// See `Webhook Verification`_ in the ngrok docs for additional details.
    ///
    /// .. _Webhook Verification: https://ngrok.com/docs/cloud-edge/modules/webhook-verification/
    pub fn webhook_verification(
        self_: PyRefMut<Self>,
        provider: String,
        secret: String,
    ) -> PyRefMut<Self> {
        self_.set(|b| {
            b.webhook_verification(provider, secret);
        });
        self_
    }
}
