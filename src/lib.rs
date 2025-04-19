use ::ngrok::prelude::Error;
use listener::Listener;
use listener_builder::{
    HttpListenerBuilder,
    LabeledListenerBuilder,
    TcpListenerBuilder,
    TlsListenerBuilder,
};
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
    Session,
    SessionBuilder,
};
use tracing::debug;

use crate::{
    connect::{
        async_connect,
        async_disconnect,
        connect as connect_fn,
        disconnect,
        forward,
        kill,
    },
    listener::{
        async_listeners,
        get_listeners,
    },
    logging::log_level,
    session::set_auth_token,
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
pub mod listener;
pub mod listener_builder;
pub mod logging;
pub mod session;
pub mod tcp;
pub mod tls;
pub mod wrapper;

// A Python module implemented in Rust. The name of this function must match
// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
// import the module.

/// The ngrok Agent SDK for Python
#[pymodule]
fn ngrok(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(async_connect, m)?)?;
    m.add_function(wrap_pyfunction!(async_disconnect, m)?)?;
    m.add_function(wrap_pyfunction!(async_listeners, m)?)?;
    m.add_function(wrap_pyfunction!(connect_fn, m)?)?;
    m.add_function(wrap_pyfunction!(default, m)?)?;
    m.add_function(wrap_pyfunction!(disconnect, m)?)?;
    m.add_function(wrap_pyfunction!(fd, m)?)?;
    m.add_function(wrap_pyfunction!(forward, m)?)?;
    m.add_function(wrap_pyfunction!(getsockname, m)?)?;
    m.add_function(wrap_pyfunction!(kill, m)?)?;
    m.add_function(wrap_pyfunction!(listen, m)?)?;
    m.add_function(wrap_pyfunction!(log_level, m)?)?;
    m.add_function(wrap_pyfunction!(pipe_name, m)?)?;
    m.add_function(wrap_pyfunction!(set_auth_token, m)?)?;
    m.add_function(wrap_pyfunction!(get_listeners, m)?)?;
    m.add_function(wrap_pyfunction!(werkzeug_develop, m)?)?;

    m.add_class::<SessionBuilder>()?;
    m.add_class::<Session>()?;

    m.add_class::<Listener>()?;
    m.add_class::<HttpListenerBuilder>()?;
    m.add_class::<LabeledListenerBuilder>()?;
    m.add_class::<TcpListenerBuilder>()?;
    m.add_class::<TlsListenerBuilder>()?;

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

// Shorthand for Error creation from NgrokError
pub(crate) fn py_ngrok_err(message: impl Into<String>, err: &impl Error) -> PyErr {
    let py_err = if let Some(error_code) = err.error_code() {
        PyValueError::new_err((message.into(), err.msg(), error_code.to_string()))
    } else {
        PyValueError::new_err((message.into(), err.msg()))
    };
    py_err
}
