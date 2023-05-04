use pyo3::{
    exceptions::PyValueError,
    pymodule,
    types::PyModule,
    wrap_pyfunction,
    PyErr,
    PyResult,
    Python,
};
use session::{
    NgrokSession,
    NgrokSessionBuilder,
};
use tracing::debug;
use tunnel::NgrokTunnel;
use tunnel_builder::{
    NgrokHttpTunnelBuilder,
    NgrokLabeledTunnelBuilder,
    NgrokTcpTunnelBuilder,
    NgrokTlsTunnelBuilder,
};

use crate::{
    connect::{
        async_connect,
        connect as connect_fn,
    },
    logging::log_level,
    wrapper::{
        default,
        fd,
        getsockname,
        listen,
        pipe_name,
        werkzeug_develop,
    },
};

pub mod connect;
pub mod http;
pub mod logging;
pub mod session;
pub mod tcp;
pub mod tls;
pub mod tunnel;
pub mod tunnel_builder;
pub mod wrapper;

// A Python module implemented in Rust. The name of this function must match
// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
// import the module.

/// The ngrok Agent SDK for Python
#[pymodule]
fn ngrok(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(async_connect, m)?)?;
    m.add_function(wrap_pyfunction!(connect_fn, m)?)?;
    m.add_function(wrap_pyfunction!(default, m)?)?;
    m.add_function(wrap_pyfunction!(fd, m)?)?;
    m.add_function(wrap_pyfunction!(getsockname, m)?)?;
    m.add_function(wrap_pyfunction!(listen, m)?)?;
    m.add_function(wrap_pyfunction!(log_level, m)?)?;
    m.add_function(wrap_pyfunction!(pipe_name, m)?)?;
    m.add_function(wrap_pyfunction!(werkzeug_develop, m)?)?;

    m.add_class::<NgrokSessionBuilder>()?;
    m.add_class::<NgrokSession>()?;

    m.add_class::<NgrokTunnel>()?;
    m.add_class::<NgrokHttpTunnelBuilder>()?;
    m.add_class::<NgrokLabeledTunnelBuilder>()?;
    m.add_class::<NgrokTcpTunnelBuilder>()?;
    m.add_class::<NgrokTlsTunnelBuilder>()?;

    // turn on logging bridge by default, since user won't see unless they activate Python logging
    if let Err(e) = log_level(py, None) {
        debug!("Error enabling logging: {e:?}")
    }
    Ok(())
}

// Shorthand for PyValueError creation
pub(crate) fn py_err(message: impl Into<String>) -> PyErr {
    PyValueError::new_err(message.into())
}
