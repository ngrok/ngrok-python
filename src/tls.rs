use bytes::Bytes;
use pyo3::{
    pymethods,
    types::PyByteArray,
    PyRefMut,
};

use crate::listener_builder::TlsListenerBuilder;

#[pymethods]
#[allow(dead_code)]
impl TlsListenerBuilder {
    /// The domain to request for this edge.
    pub fn domain(self_: PyRefMut<Self>, domain: String) -> PyRefMut<Self> {
        self_.set(|b| {
            b.domain(domain);
        });
        self_
    }
    /// Certificates to use for client authentication at the ngrok edge.
    pub fn mutual_tlsca<'a>(
        self_: PyRefMut<'a, Self>,
        mutual_tlsca: &PyByteArray,
    ) -> PyRefMut<'a, Self> {
        self_.set(|b| {
            b.mutual_tlsca(Bytes::from(mutual_tlsca.to_vec()));
        });
        self_
    }
    /// The key to use for TLS termination at the ngrok edge in PEM format.
    pub fn termination<'a>(
        self_: PyRefMut<'a, Self>,
        cert_pem: &PyByteArray,
        key_pem: &PyByteArray,
    ) -> PyRefMut<'a, Self> {
        self_.set(|b| {
            b.termination(
                Bytes::from(cert_pem.to_vec()),
                Bytes::from(key_pem.to_vec()),
            );
        });
        self_
    }
}
