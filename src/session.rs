use std::{
    io,
    sync::Arc,
    time::Duration,
};

// the lib.name and the pymodule below need to be 'ngrok' for that to be the python library
// name, so this has to explicitly set this as a crate with the '::' prefix
use ::ngrok::session::Session;
use async_rustls::rustls::{
    self,
    ClientConfig,
};
use ngrok::{
    session::{
        default_connect,
        ConnectError,
        SessionBuilder,
        Update,
    },
    tunnel::AcceptError,
};
use parking_lot::Mutex as SyncMutex;
use pyo3::{
    pyclass,
    pymethods,
    types::PyByteArray,
    PyAny,
    PyObject,
    PyRefMut,
    PyResult,
    Python,
};
use rustls_pemfile::Item;
use tracing::{
    debug,
    info,
};

use crate::{
    py_err,
    tunnel::remove_global_tunnel,
    tunnel_builder::{
        NgrokHttpTunnelBuilder,
        NgrokLabeledTunnelBuilder,
        NgrokTcpTunnelBuilder,
        NgrokTlsTunnelBuilder,
    },
};

const CLIENT_TYPE: &str = "library/official/python";
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The builder for an ngrok session.
#[pyclass]
#[allow(dead_code)]
pub(crate) struct NgrokSessionBuilder {
    raw_builder: Arc<SyncMutex<Option<SessionBuilder>>>,
    connect_handler: Option<PyObject>,
    disconnect_handler: Option<PyObject>,
}

impl NgrokSessionBuilder {
    /// Handle the locking and Option management
    fn set<F>(&self, f: F)
    where
        F: FnOnce(SessionBuilder) -> SessionBuilder,
    {
        let mut builder = self.raw_builder.lock();
        *builder = builder.take().map(f);
    }

    /// Update the connector callback in the upstream rust sdk.
    fn update_connector(&self) {
        // register connect handler. this needs the return value, so cannot use call_tsfn().
        // let mut builder = self.raw_builder.lock();
        // clone for move to connector function
        let disconnect_handler = self.disconnect_handler.clone();
        let connect_handler = self.connect_handler.clone();

        self.set(|b| {
            b.connector(
                move |addr: String, tls_config: Arc<ClientConfig>, err: Option<AcceptError>| {
                    // clone for async move out of environment
                    let disconn_tsfn = disconnect_handler.clone();
                    let conn_tsfn = connect_handler.clone();
                    async move {
                        // call disconnect javascript handler
                        if let Some(handler) = disconn_tsfn {
                            if let Some(err) = err.clone() {
                                Python::with_gil(|py| -> PyResult<()> {
                                    handler
                                        .call(py, (addr.clone(), err.to_string()), None)
                                        .map(|_o| ())
                                })
                                .map_err(|_e| ConnectError::Canceled)?;
                            }
                        };
                        // call connect javascript handler
                        if let Some(handler) = conn_tsfn {
                            Python::with_gil(|py| -> PyResult<()> {
                                handler.call(py, (addr.clone(),), None).map(|_o| ())
                            })
                            .map_err(|_e| ConnectError::Canceled)?;
                        };
                        // call the upstream connector
                        default_connect(addr, tls_config, err).await
                    }
                },
            )
        });
    }
}

#[pymethods]
impl NgrokSessionBuilder {
    fn __str__(&self) -> String {
        "ngrok_session_builder".to_string()
    }

    /// Create a new session builder
    #[new]
    pub fn new() -> Self {
        NgrokSessionBuilder {
            raw_builder: Arc::new(SyncMutex::new(Some(
                Session::builder().child_client(CLIENT_TYPE, VERSION),
            ))),
            connect_handler: None,
            disconnect_handler: None,
        }
    }

    /// Configures the session to authenticate with the provided authtoken. You
    /// can [find your existing authtoken] or [create a new one] in the ngrok
    /// dashboard.
    ///
    /// See the [authtoken parameter in the ngrok docs] for additional details.
    ///
    /// [find your existing authtoken]: https://dashboard.ngrok.com/get-started/your-authtoken
    /// [create a new one]: https://dashboard.ngrok.com/tunnels/authtokens
    /// [authtoken parameter in the ngrok docs]: https://ngrok.com/docs/ngrok-agent/config#authtoken
    pub fn authtoken(self_: PyRefMut<Self>, authtoken: String) -> PyRefMut<Self> {
        self_.set(|b| b.authtoken(authtoken));
        self_
    }

    /// Shortcut for calling [SessionBuilder::authtoken] with the value of the
    /// NGROK_AUTHTOKEN environment variable.
    pub fn authtoken_from_env(self_: PyRefMut<Self>) -> PyRefMut<Self> {
        self_.set(|b| b.authtoken_from_env());
        self_
    }

    /// Configures how often the session will send heartbeat messages to the ngrok
    /// service to check session liveness.
    ///
    /// See the [heartbeat_interval parameter in the ngrok docs] for additional
    /// details.
    ///
    /// [heartbeat_interval parameter in the ngrok docs]: https://ngrok.com/docs/ngrok-agent/config#heartbeat_interval
    pub fn heartbeat_interval(self_: PyRefMut<Self>, heartbeat_interval: u32) -> PyRefMut<Self> {
        self_.set(|b| b.heartbeat_interval(Duration::new(heartbeat_interval.into(), 0)));
        self_
    }

    /// Configures the duration to wait for a response to a heartbeat before
    /// assuming the session connection is dead and attempting to reconnect.
    ///
    /// See the [heartbeat_tolerance parameter in the ngrok docs] for additional
    /// details.
    ///
    /// [heartbeat_tolerance parameter in the ngrok docs]: https://ngrok.com/docs/ngrok-agent/config#heartbeat_tolerance
    pub fn heartbeat_tolerance(self_: PyRefMut<Self>, heartbeat_tolerance: u32) -> PyRefMut<Self> {
        self_.set(|b| b.heartbeat_tolerance(Duration::new(heartbeat_tolerance.into(), 0)));
        self_
    }

    /// Configures the opaque, machine-readable metadata string for this session.
    /// Metadata is made available to you in the ngrok dashboard and the Agents API
    /// resource. It is a useful way to allow you to uniquely identify sessions. We
    /// suggest encoding the value in a structured format like JSON.
    ///
    /// See the [metdata parameter in the ngrok docs] for additional details.
    ///
    /// [metdata parameter in the ngrok docs]: https://ngrok.com/docs/ngrok-agent/config#metadata
    pub fn metadata(self_: PyRefMut<Self>, metadata: String) -> PyRefMut<Self> {
        self_.set(|b| b.metadata(metadata));
        self_
    }

    /// Configures the network address to dial to connect to the ngrok service.
    /// Use this option only if you are connecting to a custom agent ingress.
    ///
    /// See the [server_addr parameter in the ngrok docs] for additional details.
    ///
    /// [server_addr parameter in the ngrok docs]: https://ngrok.com/docs/ngrok-agent/config#server_addr
    pub fn server_addr(self_: PyRefMut<Self>, addr: String) -> PyRefMut<Self> {
        self_.set(|b| b.server_addr(addr));
        self_
    }

    /// Configures the TLS certificate used to connect to the ngrok service while
    /// establishing the session. Use this option only if you are connecting through
    /// a man-in-the-middle or deep packet inspection proxy. Pass in the bytes of the certificate
    /// to be used to validate the connection, then override the address to connect to via
    /// the server_addr call.
    ///
    /// Roughly corresponds to the [root_cas parameter in the ngrok docs].
    ///
    /// [root_cas parameter in the ngrok docs]: https://ngrok.com/docs/ngrok-agent/config#root_cas
    pub fn ca_cert<'a>(self_: PyRefMut<'a, Self>, cert_bytes: &PyByteArray) -> PyRefMut<'a, Self> {
        let mut root_store = rustls::RootCertStore::empty();
        let mut cert_pem = io::Cursor::new(cert_bytes.to_vec());
        root_store.add_parsable_certificates(
            rustls_pemfile::read_all(&mut cert_pem)
                .expect("a valid root certificate")
                .into_iter()
                .filter_map(|it| match it {
                    Item::X509Certificate(bs) => Some(bs),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .as_slice(),
        );

        let tls_config = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        self_.set(|b| b.tls_config(tls_config));
        self_
    }

    /// Configures a function which is called to prior the connection to the
    /// ngrok service. In the event of network disruptions, it will be called each time
    /// the session reconnects. The handler is given the address that will be used to
    /// connect the session to, e.g. "example.com:443".
    pub fn handle_connection(mut self_: PyRefMut<Self>, handler: PyObject) -> PyRefMut<Self> {
        self_.connect_handler = Some(handler);
        self_.update_connector();
        self_
    }

    /// Configures a function which is called to after a disconnection to the
    /// ngrok service. In the event of network disruptions, it will be called each time
    /// the session reconnects. The handler is given the address that will be used to
    /// connect the session to, e.g. "example.com:443", and the message from the error
    /// that occurred.
    pub fn handle_disconnection(mut self_: PyRefMut<Self>, handler: PyObject) -> PyRefMut<Self> {
        self_.disconnect_handler = Some(handler);
        self_.update_connector();
        self_
    }

    /// Configures a function which is called when the ngrok service requests that
    /// this [Session] stops. Your application may choose to interpret this callback
    /// as a request to terminate the [Session] or the entire process.
    ///
    /// Errors returned by this function will be visible to the ngrok dashboard or
    /// API as the response to the Stop operation.
    ///
    /// Do not block inside this callback. It will cause the Dashboard or API
    /// stop operation to time out. Do not call [std::process::exit] inside this
    /// callback, it will also cause the operation to time out.
    pub fn handle_stop_command(self_: PyRefMut<'_, Self>, handler: PyObject) -> PyRefMut<'_, Self> {
        self_.set(|b| {
            b.handle_stop_command(move |_req| {
                let handler = handler.clone();
                async move {
                    Python::with_gil(|py| -> PyResult<()> {
                        handler.call(py, (), None).map(|_o| ())
                    })
                    .map_err(|e| format!("Callback error {e:?}"))
                }
            })
        });
        self_
    }

    /// Configures a function which is called when the ngrok service requests
    /// that this [Session] updates. Your application may choose to interpret
    /// this callback as a request to restart the [Session] or the entire
    /// process.
    ///
    /// Errors returned by this function will be visible to the ngrok dashboard or
    /// API as the response to the Restart operation.
    ///
    /// Do not block inside this callback. It will cause the Dashboard or API
    /// stop operation to time out. Do not call [std::process::exit] inside this
    /// callback, it will also cause the operation to time out.
    pub fn handle_restart_command(
        self_: PyRefMut<'_, Self>,
        handler: PyObject,
    ) -> PyRefMut<'_, Self> {
        self_.set(|b| {
            b.handle_restart_command(move |_req| {
                let handler = handler.clone();
                async move {
                    Python::with_gil(|py| -> PyResult<()> {
                        handler.call(py, (), None).map(|_o| ())
                    })
                    .map_err(|e| format!("Callback error {e:?}"))
                }
            })
        });
        self_
    }

    /// Configures a function which is called when the ngrok service requests
    /// that this [Session] updates. Your application may choose to interpret
    /// this callback as a request to update its configuration, itself, or to
    /// invoke some other application-specific behavior.
    ///
    /// Errors returned by this function will be visible to the ngrok dashboard or
    /// API as the response to the Restart operation.
    ///
    /// Do not block inside this callback. It will cause the Dashboard or API
    /// stop operation to time out. Do not call [std::process::exit] inside this
    /// callback, it will also cause the operation to time out.
    pub fn handle_update_command(
        self_: PyRefMut<'_, Self>,
        handler: PyObject,
    ) -> PyRefMut<'_, Self> {
        self_.set(|b| {
            b.handle_update_command(move |req: Update| {
                let handler = handler.clone();
                async move {
                    Python::with_gil(|py| -> PyResult<()> {
                        handler
                            .call(py, (req.version, req.permit_major_version), None)
                            .map(|_o| ())
                    })
                    .map_err(|e| format!("Callback error {e:?}"))
                }
            })
        });
        self_
    }

    /// Call the provided handler whenever a heartbeat response is received,
    /// with the latency in milliseconds.
    ///
    /// If the handler returns an error, the heartbeat task will exit, resulting
    /// in the session eventually dying as well.
    pub fn handle_heartbeat(self_: PyRefMut<'_, Self>, handler: PyObject) -> PyRefMut<'_, Self> {
        self_.set(|b| {
            b.handle_heartbeat(move |latency: Option<Duration>| {
                let handler = handler.clone();
                let millis = latency.map(|d| u32::try_from(d.as_millis()).ok()).flatten();
                async move {
                    Python::with_gil(|py| -> PyResult<()> {
                        if let Some(m) = millis {
                            handler.call(py, (m,), None)
                        } else {
                            handler.call(py, (), None)
                        }
                        .map(|_o| ())
                    })
                    .map_err(|e| format!("Callback error {e:?}").into())
                }
            })
        });
        self_
    }

    /// Attempt to establish an ngrok session using the current configuration.
    pub fn connect<'a>(&self, py: Python<'a>) -> PyResult<&'a PyAny> {
        let builder = self.raw_builder.lock().clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            builder
                .expect("session builder is always set")
                .connect()
                .await
                .map(|s| {
                    info!("Session created");
                    NgrokSession {
                        raw_session: Arc::new(SyncMutex::new(s)),
                    }
                })
                .map_err(|e| py_err(format!("failed to connect session, {e:?}")))
        })
    }
}

impl Drop for NgrokSessionBuilder {
    fn drop(&mut self) {
        debug!("NgrokSessionBuilder drop");
    }
}

/// An ngrok session.
#[pyclass]
#[derive(Clone)]
pub(crate) struct NgrokSession {
    raw_session: Arc<SyncMutex<Session>>,
}

#[pymethods]
impl NgrokSession {
    fn __str__(&self) -> String {
        "ngrok_session".to_string()
    }

    /// Start building a tunnel backing an HTTP endpoint.
    pub fn http_endpoint(&self) -> NgrokHttpTunnelBuilder {
        let session = self.raw_session.lock().clone();
        NgrokHttpTunnelBuilder::new(session.clone(), session.http_endpoint())
    }

    /// Start building a tunnel backing a TCP endpoint.
    pub fn tcp_endpoint(&self) -> NgrokTcpTunnelBuilder {
        let session = self.raw_session.lock().clone();
        NgrokTcpTunnelBuilder::new(session.clone(), session.tcp_endpoint())
    }

    /// Start building a tunnel backing a TLS endpoint.
    pub fn tls_endpoint(&self) -> NgrokTlsTunnelBuilder {
        let session = self.raw_session.lock().clone();
        NgrokTlsTunnelBuilder::new(session.clone(), session.tls_endpoint())
    }

    /// Start building a labeled tunnel.
    pub fn labeled_tunnel(&self) -> NgrokLabeledTunnelBuilder {
        let session = self.raw_session.lock().clone();
        NgrokLabeledTunnelBuilder::new(session.clone(), session.labeled_tunnel())
    }

    /// Close a tunnel with the given ID.
    pub fn close_tunnel<'a>(&self, py: Python<'a>, id: String) -> PyResult<&'a PyAny> {
        let session = self.raw_session.lock().clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let res = session
                .close_tunnel(id.clone())
                .await
                .map_err(|e| py_err(format!("failed to close tunnel, {e:?}")));

            if res.is_ok() {
                // remove our reference to allow it to drop
                remove_global_tunnel(&id).await?;
            }
            res
        })
    }

    /// Close the ngrok session.
    pub fn close<'a>(&self, py: Python<'a>) -> PyResult<&'a PyAny> {
        let mut session = self.raw_session.lock().clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            session
                .close()
                .await
                .map_err(|e| py_err(format!("failed to close tunnel, {e:?}")))
        })
    }
}

impl Drop for NgrokSession {
    fn drop(&mut self) {
        debug!("NgrokSession drop");
    }
}
