use std::{
    str::FromStr,
    sync::Arc,
};

use ngrok::{
    prelude::*,
    Session,
};
use parking_lot::Mutex;
use pyo3::{
    pyclass,
    pymethods,
    Py,
    PyAny,
    PyRefMut,
    PyResult,
    Python,
};
use tracing::debug;
use url::Url;

use crate::{
    listener::{
        HttpListener,
        LabeledListener,
        Listener,
        TcpListener,
        TlsListener,
    },
    py_err,
    py_ngrok_err,
    wrapper::address_from_server,
};

macro_rules! make_listener_builder {
    ($(#[$outer:meta])* $wrapper:ident, $builder:tt, $listener:tt, $mode:tt) => {
        $(#[$outer])*
        #[pyclass]
        #[allow(dead_code)]
        pub(crate) struct $wrapper {
            session: Arc<Mutex<Session>>,
            pub(crate) listener_builder: Arc<Mutex<$builder>>,
        }

        #[pymethods]
        #[allow(dead_code)]
        impl $wrapper {
            /// Listener-specific opaque metadata. Viewable via the API.
            pub fn metadata(self_: PyRefMut<Self>, metadata: String) -> PyRefMut<Self> {
                self_.set(|b| {b.metadata(metadata);});
                self_
            }

            /// Whether to disable certificate verification for this listener.
            pub fn verify_upstream_tls(self_: PyRefMut<Self>, verify_upstream_tls: bool) -> PyRefMut<Self> {
                self_.set(|b| {b.verify_upstream_tls(verify_upstream_tls);});
                self_
            }

            /// Begin listening for new connections on this listener.
            pub fn listen<'a>(&self, py: Python<'a>) -> PyResult<&'a PyAny> {
                let session = self.session.lock().clone();
                let tun = self.listener_builder.lock().clone();
                pyo3_asyncio::tokio::future_into_py(
                    py,
                    async move {
                        $wrapper::do_listen(session, tun).await
                    },
                )
            }

            /// Begin listening for new connections on this listener and forwarding them to the given url.
            /// This url can be either a TCP/HTTP address or a file socket path, for example:
            /// "http://localhost:8080", "https://192.168.1.100:8443", or for file socket paths on
            /// Linux/Darwin "unix:///path/to/unix.sock".
            ///
            /// :param to_url: The URL to forward traffic on to
            /// :return: A task to await for the :class:`Listener` linked with the server.
            /// :rtype: Task
            pub fn listen_and_forward<'a>(&self, to_url: String, py: Python<'a>) -> PyResult<&'a PyAny> {
                let url = Url::parse(&to_url).map_err(|e| py_err(format!("Url forward argument parse failure, {e}")))?;
                let session = self.session.lock().clone();
                let builder = self.listener_builder.lock().clone();

                pyo3_asyncio::tokio::future_into_py(
                    py,
                    async move {
                        let result = builder
                        .listen_and_forward(url)
                        .await
                        .map_err(|e| py_ngrok_err("failed to start listener", &e));

                        // create the wrapping listener object via its async new()
                        match result {
                            Ok(raw_fwd) => Ok($listener::new_forwarder(session, raw_fwd).await),
                            Err(val) => Err(val),
                        }
                    },
                )
            }

            /// Begin listening for new connections on this listener and forwarding them to the given http server.
            ///
            /// :param server: The server to link with a :class:`Listener`.
            /// :type server: http.server.HTTPServer or None
            /// :return: A task to await for the :class:`Listener` linked with the server.
            /// :rtype: Task
            pub fn listen_and_serve<'a>(
                &self,
                py: Python<'a>,
                server: Py<PyAny>,
            ) -> PyResult<&'a PyAny> {
                let address = address_from_server(py, server)?;
                return self.listen_and_forward(address, py)
            }
        }

        #[allow(dead_code)]
        impl $wrapper {
            pub(crate) fn new(session: Session, raw_listener_builder: $builder) -> Self {
                $wrapper {
                    session: Arc::new(Mutex::new(session)),
                    listener_builder: Arc::new(Mutex::new(raw_listener_builder)),
                }
            }

            /// Handle the locking and Option management
            pub(crate) fn set<F>(&self, f: F)
            where
                F: FnOnce(&mut parking_lot::lock_api::MutexGuard<'_, parking_lot::RawMutex, $builder>),
            {
                let mut builder = self.listener_builder.lock();
                f(&mut builder);
            }

            pub(crate) async fn async_listen(&self) -> PyResult<Listener> {
                let session = self.session.lock().clone();
                let tun = self.listener_builder.lock().clone();
                $wrapper::do_listen(session, tun).await
            }

            async fn do_listen(session: Session, builder: $builder) -> PyResult<Listener> {
                let result = builder
                            .listen()
                            .await
                            .map_err(|e| py_ngrok_err("failed to start listener", &e));

                // create the wrapping listener object via its async new()
                match result {
                    Ok(raw_tun) => Ok($listener::new_listener(session, raw_tun).await),
                    Err(val) => Err(val),
                }
            }
        }

        impl Drop for $wrapper {
            fn drop(&mut self) {
                debug!("{} drop", stringify!($wrapper));
            }
        }


        // mode specific methods
        make_listener_builder!($mode, $wrapper);
    };

    (common, $wrapper:ty) => {
        #[pymethods]
        #[allow(dead_code)]
        impl $wrapper {
            /// Restriction placed on the origin of incoming connections to the edge to only allow these CIDR ranges.
            /// Call multiple times to add additional CIDR ranges.
            /// See `IP restrictions`_ in the ngrok docs for additional details.
            ///
            /// .. _IP restrictions: https://ngrok.com/docs/cloud-edge/modules/ip-restrictions/
            pub fn allow_cidr(self_: PyRefMut<Self>, cidr: String) -> PyRefMut<Self> {
                self_.set(|b| {b.allow_cidr(cidr);});
                self_
            }
            /// Restriction placed on the origin of incoming connections to the edge to deny these CIDR ranges.
            /// Call multiple times to add additional CIDR ranges.
            /// See `IP restrictions`_ in the ngrok docs for additional details.
            ///
            /// .. _IP restrictions: https://ngrok.com/docs/cloud-edge/modules/ip-restrictions/
            pub fn deny_cidr(self_: PyRefMut<Self>, cidr: String) -> PyRefMut<Self> {
                self_.set(|b| {b.deny_cidr(cidr);});
                self_
            }
            /// The version of PROXY protocol to use with this listener "1", "2", or "" if not using.
            pub fn proxy_proto(self_: PyRefMut<Self>, proxy_proto: String) -> PyRefMut<Self> {
                self_.set(|b| {b.proxy_proto(
                    ProxyProto::from_str(proxy_proto.as_str())
                        .unwrap_or_else(|_| panic!("Unknown proxy protocol: {:?}", proxy_proto)),
                );});
                self_
            }
            /// Listener backend metadata. Viewable via the dashboard and API, but has no
            /// bearing on listener behavior.
            ///
            /// To automatically forward connections, you can use :any:`listen_and_forward`,
            /// or :any:`listen_and_serve` on the Listener Builder.
            pub fn forwards_to(self_: PyRefMut<Self>, forwards_to: String) -> PyRefMut<Self> {
                self_.set(|b| {b.forwards_to(forwards_to);});
                self_
            }

            /// DEPRECATED: use traffic_policy instead.
            /// :param str policy_config: Traffic policy configuration to be attached to the listener.
            pub fn policy(self_: PyRefMut<Self>, policy_config: String) -> PyRefMut<Self> {
                self_.set(|b| {b.traffic_policy(policy_config);});
                self_
            }

            /// Traffic Policy configuration.
            /// :param str policy_config: Traffic policy configuration to be attached to the listener.
            pub fn traffic_policy(self_: PyRefMut<Self>, policy_config: String) -> PyRefMut<Self> {
                self_.set(|b| {b.traffic_policy(policy_config);});
                self_
            }
        }
    };

    (label, $wrapper:ty) => {
        #[pymethods]
        #[allow(dead_code)]
        impl $wrapper {
            /// Add a label, value pair for this listener.
            /// See `Using Labels`_ in the ngrok docs for additional details.
            ///
            /// .. _Using Labels: https://ngrok.com/docs/guides/using-labels-within-ngrok/
            pub fn label(self_: PyRefMut<Self>, label: String, value: String) -> PyRefMut<Self> {
                self_.set(|b| {b.label(label, value);});
                self_
            }

            /// Set the L7 application protocol used for this listener, i.e. "http1" or "http2" (default "http1")
            pub fn app_protocol(self_: PyRefMut<Self>, app_protocol: String) -> PyRefMut<Self> {
                self_.set(|b| {b.app_protocol(app_protocol);});
                self_
            }
        }
    };
}

make_listener_builder! {
    /// An ngrok listener backing an HTTP endpoint.
    HttpListenerBuilder, HttpTunnelBuilder, HttpListener, common
}
make_listener_builder! {
    /// An ngrok listener backing a TCP endpoint.
    TcpListenerBuilder, TcpTunnelBuilder, TcpListener, common
}
make_listener_builder! {
    /// An ngrok listener backing a TLS endpoint.
    TlsListenerBuilder, TlsTunnelBuilder, TlsListener, common
}
make_listener_builder! {
    /// A labeled ngrok listener.
    LabeledListenerBuilder, LabeledTunnelBuilder, LabeledListener, label
}
