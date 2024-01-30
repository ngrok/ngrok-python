use std::{
    borrow::BorrowMut,
    env,
    sync::Arc,
    time::Duration,
};

use ::ngrok::policy::Policy as NgrokPolicy; // TODO: (Kristopher Paulsen) Where to import policy???
use async_rustls::rustls::ClientConfig;
use bytes::Bytes;
use lazy_static::lazy_static;
use ngrok::{
    Policy::{
        default_connect,
        ConnectError,
        PolicyBuilder as NgrokPolicyBuilder,
        Update,
    },
    tunnel::AcceptError,
};

// the lib.name and the pymodule below need to be 'ngrok' for that to be the python library
// name, so this has to explicitly set this as a crate with the '::' prefix
use ::ngrok::policy::Policy as NgrokPolicy;
use async_rustls::rustls::ClientConfig;
use bytes::Bytes;
use lazy_static::lazy_static;
use pyo3::{
    pyclass,
    pyfunction,
    pymethods,
    types::PyByteArray,
    PyAny,
    PyErr,
    PyObject,
    PyRefMut,
    PyResult,
    Python,
};
use tracing::{
    debug,
    info,
};

const CLIENT_TYPE: &str = "ngrok-python";
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The builder for an ngrok session.
#[pyclass]
#[allow(dead_code)]
pub(crate) struct PolicyBuilder {
    raw_builder: Arc<SyncMutex<NgrokPolicyBuilder>>,
    // disconnect_handler: Option<PyObject>,
    // auth_token_set: bool, // TODO: (Kristopher Paulsen) ????? what do here?
}

#[pymethods]
impl PolicyBuilder {
    fn __str__(&self) -> String {
        "ngrok_policy_builder".to_string()
    }

    fn add_inbound() -> Void {

    }

    fn add_outbound() -> Void {
    }
}
#[pymethods]
impl PolicyRuleBuilder {
    fn __str__(&self) -> String {
        "ngrok_policy_rule_builder".to_string()
    }

    fn add_expression(&self) -> Void {
    }

    fn add_action() -> Void {
    }
}