use pyo3::{
    pymethods,
    PyRefMut,
};

use crate::listener_builder::TcpListenerBuilder;

#[pymethods]
#[allow(dead_code)]
impl TcpListenerBuilder {
    /// The TCP address to request for this edge.
    /// These addresses can be reserved in the `ngrok dashboard`_ to use across sessions. For example: remote_addr("2.tcp.ngrok.io:21746")
    ///
    /// .. _ngrok dashboard: https://dashboard.ngrok.com/cloud-edge/tcp-addresses
    pub fn remote_addr(self_: PyRefMut<Self>, remote_addr: String) -> PyRefMut<Self> {
        self_.set(|b| {
            b.remote_addr(remote_addr);
        });
        self_
    }
}
