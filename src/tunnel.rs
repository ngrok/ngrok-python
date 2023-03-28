use std::{
    collections::HashMap,
    io,
    sync::Arc,
};

// the lib.name and the pymodule below need to be 'ngrok' for that to be the python library
// name, so this has to explicitly set this as a crate with the '::' prefix
use ::ngrok::{
    prelude::*,
    tunnel::{
        TcpTunnel,
        UrlTunnel,
    },
    Session,
};
use async_trait::async_trait;
use lazy_static::lazy_static;
use ngrok::tunnel::{
    HttpTunnel,
    LabeledTunnel,
    TlsTunnel,
};
use pyo3::{
    intern,
    once_cell::GILOnceCell,
    prelude::*,
    pyclass,
    pymethods,
    types::{
        PyDict,
        PyString,
        PyTuple,
    },
    PyAny,
    PyResult,
    Python,
};
use tokio::sync::Mutex;
use tracing::{
    debug,
    info,
};

use crate::{
    py_err,
    wrapper::{
        bound_default_pipe_socket,
        bound_default_tcp_socket,
    },
};

static SOCK_CELL: GILOnceCell<Py<PyDict>> = GILOnceCell::new();

lazy_static! {
    // tunnel references to be kept until explicit close to prevent python gc from dropping them.
    // the tunnel wrapper object, and the underlying tunnel, both have references to the Session
    // so the Session is safe from premature dropping.
    static ref GLOBAL_TUNNELS: Mutex<HashMap<String,Arc<Mutex<dyn ExtendedTunnel>>>> = Mutex::new(HashMap::new());
}

/// The TunnelExt cannot be turned into an object since it contains generics, so implementing
/// a proxy trait without generics which can be the dyn type stored in the global map.
#[async_trait]
pub trait ExtendedTunnel: Tunnel {
    async fn fwd_tcp(&mut self, addr: String) -> core::result::Result<(), io::Error>;
    async fn fwd_pipe(&mut self, addr: String) -> core::result::Result<(), io::Error>;
}

macro_rules! make_tunnel_type {
    // the common (non-labeled) branch
    ($(#[$outer:meta])* $wrapper:ident, $tunnel:tt, common) => {

        $(#[$outer])*
        #[pyclass]
        #[allow(dead_code)]
        pub(crate) struct $wrapper {
            id: String,
            forwards_to: String,
            metadata: String,
            url: String,
            proto: String,
            session: Session,
        }

        #[allow(dead_code)]
        impl $wrapper {
            pub(crate) async fn new(session: Session, raw_tunnel: $tunnel) -> Self {
                let id = raw_tunnel.id().to_string();
                let forwards_to = raw_tunnel.forwards_to().to_string();
                let metadata = raw_tunnel.metadata().to_string();
                let url = raw_tunnel.url().to_string();
                let proto = raw_tunnel.proto().to_string();
                info!("Created tunnel {id:?} with url {url:?}");
                // keep a tunnel reference until an explicit call to close to prevent python gc dropping it
                GLOBAL_TUNNELS.lock().await.insert(id.clone(), Arc::new(Mutex::new(raw_tunnel)));
                $wrapper {
                    id,
                    forwards_to,
                    metadata,
                    url,
                    proto,
                    session,
                }
            }
        }

        #[pymethods]
        #[allow(dead_code)]
        impl $wrapper {
            /// The URL that this tunnel backs.
            pub fn url(&self) -> String {
                self.url.clone()
            }

            /// The protocol of the endpoint that this tunnel backs.
            pub fn proto(&self) -> String {
                self.proto.clone()
            }
        }

        make_tunnel_type!($wrapper, $tunnel);
    };

    // the labeled branch
    ($(#[$outer:meta])* $wrapper:ident, $tunnel:tt, label) => {
        $(#[$outer])*
        #[pyclass]
        #[allow(dead_code)]
        pub(crate) struct $wrapper {
            id: String,
            forwards_to: String,
            metadata: String,
            labels: HashMap<String,String>,
            session: Session,
        }

        #[allow(dead_code)]
        impl $wrapper {
            pub(crate) async fn new(session: Session, raw_tunnel: $tunnel) -> Self {
                let id = raw_tunnel.id().to_string();
                let forwards_to = raw_tunnel.forwards_to().to_string();
                let metadata = raw_tunnel.metadata().to_string();
                let labels = raw_tunnel.labels().clone();
                info!("Created tunnel {id:?} with labels {labels:?}");
                // keep a tunnel reference until an explicit call to close to prevent python gc dropping it
                GLOBAL_TUNNELS.lock().await.insert(id.clone(), Arc::new(Mutex::new(raw_tunnel)));
                $wrapper {
                    id,
                    forwards_to,
                    metadata,
                    labels,
                    session,
                }
            }
        }

        #[pymethods]
        #[allow(dead_code)]
        impl $wrapper {
            /// The labels this tunnel was started with.
            pub fn labels(&self) -> HashMap<String, String> {
                self.labels.clone()
            }
        }

        make_tunnel_type!($wrapper, $tunnel);
    };

    // all tunnels get these
    ($wrapper:ident, $tunnel:tt) => {
        #[async_trait]
        impl ExtendedTunnel for $tunnel {
            async fn fwd_tcp(&mut self, addr: String) -> core::result::Result<(), io::Error> {
                self.forward_tcp(addr).await
            }
            async fn fwd_pipe(&mut self, addr: String) -> core::result::Result<(), io::Error> {
                self.forward_pipe(addr).await
            }
        }

        #[pymethods]
        #[allow(dead_code)]
        impl $wrapper {
            /// Returns a tunnel's unique ID.
            pub fn id(&self) -> String {
                self.id.clone()
            }

            /// Returns a human-readable string presented in the ngrok dashboard
            /// and the Tunnels API. Use the [HttpTunnelBuilder::forwards_to],
            /// [TcpTunnelBuilder::forwards_to], etc. to set this value
            /// explicitly.
            pub fn forwards_to(&self) -> String {
                self.forwards_to.clone()
            }

            /// Returns the arbitrary metadata string for this tunnel.
            pub fn metadata(&self) -> String {
                self.metadata.clone()
            }

            /// Forward incoming tunnel connections to the provided TCP address.
            pub fn forward_tcp<'a>(&self, py: Python<'a>, addr: String) -> PyResult<&'a PyAny> {
                let id = self.id.clone();
                pyo3_asyncio::tokio::future_into_py(
                    py,
                    async move {
                        info!("Tunnel {id:?} TCP forwarding to {addr:?}");

                        // we must clone the Arc before locking so we have a local reference
                        // to the mutex to unlock if this struct is dropped.
                        let arc = GLOBAL_TUNNELS.lock().await
                            .get(&id)
                            .ok_or(py_err("Tunnel is no longer running"))?
                            .clone(); // required clone

                        // doing this as a seperate statement so the GLOBAL_TUNNELS lock is dropped
                        let res = arc
                            .lock()
                            .await
                            .fwd_tcp(addr)
                            .await
                            .map_err(|e| py_err(format!("cannot forward tcp: {e:?}")));
                        debug!("forward_tcp returning");
                        res
                    }
                )
            }

            /// Forward incoming tunnel connections to the provided file socket path.
            /// On Linux/Darwin addr can be a unix domain socket path, e.g. "/tmp/ngrok.sock"
            /// On Windows addr can be a named pipe, e.e. "\\\\.\\pipe\\an_ngrok_pipe"
            pub fn forward_pipe<'a>(&self, py: Python<'a>, addr: String) -> PyResult<&'a PyAny> {
                let id = self.id.clone();
                pyo3_asyncio::tokio::future_into_py(
                    py,
                    async move {
                        info!("Tunnel {id:?} Pipe forwarding to {addr:?}");

                        // we must clone the Arc before locking so we have a local reference
                        // to the mutex to unlock if this struct is dropped.
                        let arc = GLOBAL_TUNNELS.lock().await
                            .get(&id)
                            .ok_or(py_err("Tunnel is no longer running"))?
                            .clone(); // required clone

                        // doing this as a seperate statement so the GLOBAL_TUNNELS lock is dropped
                        let res = arc
                            .lock()
                            .await
                            .fwd_pipe(addr)
                            .await
                            .map_err(|e| py_err(format!("cannot forward pipe: {e:?}")));
                        debug!("forward_pipe returning");
                        res
                    }
                )
            }

            /// Close the tunnel.
            ///
            /// This is an RPC call that must be `.await`ed.
            /// It is equivalent to calling `Session::close_tunnel` with this
            /// tunnel's ID.
            pub fn close<'a>(&self, py: Python<'a>) -> PyResult<&'a PyAny> {
                let session = self.session.clone();
                let id = self.id.clone();
                pyo3_asyncio::tokio::future_into_py(
                    py,
                    async move {
                        debug!("{} closing, id: {id:?}", stringify!($wrapper));

                        // we may not be able to lock our reference to the tunnel due to the forward_* calls which
                        // continuously accept-loop while the tunnel is active, so calling close on the Session.
                        let res = session.close_tunnel(id.clone())
                            .await
                            .map_err(|e| py_err(format!("error closing tunnel: {e:?}")));

                        // drop our internal reference to the tunnel after awaiting close
                        remove_global_tunnel(&id).await?;

                        res
                    }
                )
            }
        }

        // Methods designed to act like a native socket
        #[pymethods]
        #[allow(dead_code)]
        impl $wrapper {
            // for aiohttp case, proxy calls to socket
            #[getter]
            pub fn get_family(&self, py: Python) -> PyResult<Py<PyAny>> {
                self.get_sock_attr(py, intern!(py, "family"))
            }

            pub fn getsockname(&self, py: Python) -> PyResult<Py<PyAny>> {
                self.get_sock_attr(py, intern!(py, "getsockname"))?.call0(py)
            }

            #[getter]
            pub fn get_type(&self, py: Python) -> PyResult<Py<PyAny>> {
                self.get_sock_attr(py, intern!(py, "type"))
            }

            pub fn setblocking(&self, py: Python, blocking: bool) -> PyResult<Py<PyAny>> {
                let args = PyTuple::new(py, &[blocking]);
                self.get_sock_attr(py, intern!(py, "setblocking"))?.call1(py, args)
            }

            pub fn fileno(&self, py: Python) -> PyResult<Py<PyAny>> {
                self.get_sock_attr(py, intern!(py, "fileno"))?.call0(py)
            }

            pub fn accept(&self, py: Python) -> PyResult<Py<PyAny>> {
                self.get_sock_attr(py, intern!(py, "accept"))?.call0(py)
            }

            pub fn listen(&self, py: Python, backlog: i32) -> PyResult<Py<PyAny>> {
                // call listen on socket
                let args = PyTuple::new(py, &[backlog]);
                let result = self.get_sock_attr(py, intern!(py, "listen"))?.call1(py, args);

                // set up forwarding depending on socket type
                let sockname = self.getsockname(py)?;
                let socket = PyModule::import(py, "socket")?;
                let af_unix = socket.getattr(intern!(py, "AF_UNIX"))?;
                if self.get_family(py)?.as_ref(py).eq(af_unix)? {
                    // pipe
                    let sockname_str: &PyString = sockname.downcast(py)?;
                    self.forward_pipe(py, sockname_str.to_string())?;
                }
                else {
                    // tcp
                    let sockname_tuple: &PyTuple = sockname.downcast(py)?;
                    self.forward_tcp(py, format!("localhost:{}", sockname_tuple.get_item(1)?))?;
                }
                result
            }

            // For uvicorn case, generate a file descriptor for a listening socket
            #[getter]
            pub fn get_fd(&self, py: Python) -> PyResult<Py<PyAny>> {
                self.listen(py, 0)?;
                self.fileno(py)
            }

            // Get or create the python socket to use for this tunnel, and return an attribute of it.
            fn get_sock_attr(&self, py: Python, attr: &PyString) -> PyResult<Py<PyAny>> {
                let map: &PyDict = SOCK_CELL.get_or_init(py, || PyDict::new(py).into()).extract(py)?;
                let maybe_socket = map.get_item(&self.id);
                let socket = match maybe_socket {
                    Some(s) => s.into_py(py),
                    None => {
                        // try pipe first, fall back to tcp
                        let res = match bound_default_pipe_socket(py) {
                            Ok(res) => res,
                            Err(error) => {
                                debug!("error binding to pipe: {}", error);
                                bound_default_tcp_socket(py)?
                            }
                        };
                        map.set_item(self.id.clone(), res.clone())?;
                        res
                    }
                };
                socket.getattr(py, attr)
            }
        }

        #[allow(unused_mut)]
        impl Drop for $wrapper {
            fn drop(&mut self) {
                debug!("{} finalize, id: {}", stringify!($wrapper), self.id);
            }
        }
    };
}

make_tunnel_type! {
    /// An ngrok tunnel backing an HTTP endpoint.
    NgrokHttpTunnel, HttpTunnel, common
}
make_tunnel_type! {
    /// An ngrok tunnel backing a TCP endpoint.
    NgrokTcpTunnel, TcpTunnel, common
}
make_tunnel_type! {
    /// An ngrok tunnel bcking a TLS endpoint.
    NgrokTlsTunnel, TlsTunnel, common
}
make_tunnel_type! {
    /// A labeled ngrok tunnel.
    NgrokLabeledTunnel, LabeledTunnel, label
}

/// Delete any reference to the tunnel id
pub(crate) async fn remove_global_tunnel(id: &String) -> PyResult<()> {
    GLOBAL_TUNNELS.lock().await.remove(id);

    // remove any references to sockets
    Python::with_gil(|py| -> PyResult<()> {
        if let Some(map) = SOCK_CELL.get(py) {
            let dict: &PyDict = map.extract(py)?;
            // close socket if it exists
            let existing = dict.get_item(id);
            if let Some(existing) = existing {
                existing.call_method0("close")?;

                // delete reference
                dict.del_item(id)?;
            }
        }
        Ok(())
    })
}
