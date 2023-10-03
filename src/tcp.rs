use pyo3::{
    pymethods,
    PyRefMut,
};

use crate::listener_builder::TcpListenerBuilder;

#[pymethods]
#[allow(dead_code)]
impl TcpListenerBuilder {
    /// The TCP address to request for this edge.
    pub fn remote_addr(self_: PyRefMut<Self>, remote_addr: String) -> PyRefMut<Self> {
        self_.set(|b| {
            b.remote_addr(remote_addr);
        });
        self_
    }
}
