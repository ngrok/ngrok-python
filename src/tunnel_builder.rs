use std::{
    str::FromStr,
    sync::Arc,
};

use ngrok::{
    config::{
        HttpTunnelBuilder,
        LabeledTunnelBuilder,
        ProxyProto,
        TcpTunnelBuilder,
        TlsTunnelBuilder,
    },
    prelude::*,
    Session,
};
use parking_lot::Mutex;
use pyo3::{
    pyclass,
    pymethods,
    PyAny,
    PyRefMut,
    PyResult,
    Python,
};
use tracing::debug;

use crate::{
    py_ngrok_err,
    tunnel::{
        NgrokHttpTunnel,
        NgrokLabeledTunnel,
        NgrokTcpTunnel,
        NgrokTlsTunnel,
        NgrokTunnel,
    },
};

macro_rules! make_tunnel_builder {
    ($(#[$outer:meta])* $wrapper:ident, $builder:tt, $tunnel:tt, $mode:tt) => {
        $(#[$outer])*
        #[pyclass]
        #[allow(dead_code)]
        pub(crate) struct $wrapper {
            session: Arc<Mutex<Session>>,
            pub(crate) tunnel_builder: Arc<Mutex<Option<$builder>>>,
        }

        #[pymethods]
        #[allow(dead_code)]
        impl $wrapper {
            /// Tunnel-specific opaque metadata. Viewable via the API.
            pub fn metadata(self_: PyRefMut<Self>, metadata: String) -> PyRefMut<Self> {
                self_.set(|b| b.metadata(metadata));
                self_
            }

            /// Begin listening for new connections on this tunnel.
            pub fn listen<'a>(&self, py: Python<'a>) -> PyResult<&'a PyAny> {
                let session = self.session.lock().clone();
                let tun = self.tunnel_builder.lock().clone();
                pyo3_asyncio::tokio::future_into_py(
                    py,
                    async move {
                        $wrapper::do_listen(session, tun).await
                    },
                )
            }
        }

        #[allow(dead_code)]
        impl $wrapper {
            pub(crate) fn new(session: Session, raw_tunnel_builder: $builder) -> Self {
                $wrapper {
                    session: Arc::new(Mutex::new(session)),
                    tunnel_builder: Arc::new(Mutex::new(Some(raw_tunnel_builder))),
                }
            }

            /// Handle the locking and Option management
            pub(crate) fn set<F>(&self, f: F)
            where
                F: FnOnce($builder) -> $builder,
            {
                let mut builder = self.tunnel_builder.lock();
                *builder = builder.take().map(f);
            }

            pub(crate) async fn async_listen(&self) -> PyResult<NgrokTunnel> {
                let session = self.session.lock().clone();
                let tun = self.tunnel_builder.lock().clone();
                $wrapper::do_listen(session, tun).await
            }

            async fn do_listen(session: Session, builder: Option<$builder>) -> PyResult<NgrokTunnel> {
                let result = builder.expect("tunnel builder is always set")
                            .listen()
                            .await
                            .map_err(|e| py_ngrok_err("failed to start tunnel", &e));

                // create the wrapping tunnel object via its async new()
                match result {
                    Ok(raw_tun) => Ok($tunnel::new_tunnel(session, raw_tun).await),
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
        make_tunnel_builder!($mode, $wrapper);
    };

    (common, $wrapper:ty) => {
        #[pymethods]
        #[allow(dead_code)]
        impl $wrapper {
            /// Restriction placed on the origin of incoming connections to the edge to only allow these CIDR ranges.
            /// Call multiple times to add additional CIDR ranges.
            pub fn allow_cidr(self_: PyRefMut<Self>, cidr: String) -> PyRefMut<Self> {
                self_.set(|b| b.allow_cidr(cidr));
                self_
            }
            /// Restriction placed on the origin of incoming connections to the edge to deny these CIDR ranges.
            /// Call multiple times to add additional CIDR ranges.
            pub fn deny_cidr(self_: PyRefMut<Self>, cidr: String) -> PyRefMut<Self> {
                self_.set(|b| b.deny_cidr(cidr));
                self_
            }
            /// The version of PROXY protocol to use with this tunnel "1", "2", or "" if not using.
            pub fn proxy_proto(self_: PyRefMut<Self>, proxy_proto: String) -> PyRefMut<Self> {
                self_.set(|b| b.proxy_proto(
                    ProxyProto::from_str(proxy_proto.as_str())
                        .unwrap_or_else(|_| panic!("Unknown proxy protocol: {:?}", proxy_proto)),
                ));
                self_
            }
            /// Tunnel backend metadata. Viewable via the dashboard and API, but has no
            /// bearing on tunnel behavior.
            pub fn forwards_to(self_: PyRefMut<Self>, forwards_to: String) -> PyRefMut<Self> {
                self_.set(|b| b.forwards_to(forwards_to));
                self_
            }
        }
    };

    (label, $wrapper:ty) => {
        #[pymethods]
        #[allow(dead_code)]
        impl $wrapper {
            /// Add a label, value pair for this tunnel.
            pub fn label(self_: PyRefMut<Self>, label: String, value: String) -> PyRefMut<Self> {
                self_.set(|b| b.label(label, value));
                self_
            }
        }
    };
}

make_tunnel_builder! {
    /// An ngrok tunnel backing an HTTP endpoint.
    NgrokHttpTunnelBuilder, HttpTunnelBuilder, NgrokHttpTunnel, common
}
make_tunnel_builder! {
    /// An ngrok tunnel backing a TCP endpoint.
    NgrokTcpTunnelBuilder, TcpTunnelBuilder, NgrokTcpTunnel, common
}
make_tunnel_builder! {
    /// An ngrok tunnel backing a TLS endpoint.
    NgrokTlsTunnelBuilder, TlsTunnelBuilder, NgrokTlsTunnel, common
}
make_tunnel_builder! {
    /// A labeled ngrok tunnel.
    NgrokLabeledTunnelBuilder, LabeledTunnelBuilder, NgrokLabeledTunnel, label
}
