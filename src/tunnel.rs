use core::result::Result as CoreResult;
use std::{
    collections::HashMap,
    error::Error,
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
use ngrok::{
    session::ConnectError,
    tunnel::{
        HttpTunnel,
        LabeledTunnel,
        TlsTunnel,
    },
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

#[cfg(target_os = "windows")]
use crate::wrapper::wrap_object;
use crate::{
    py_err,
    py_ngrok_err,
    wrapper::{
        self,
        bound_default_pipe_socket,
        bound_default_tcp_socket,
    },
};

/// Python dictionary of id's to sockets.
static SOCK_CELL: GILOnceCell<Py<PyDict>> = GILOnceCell::new();

lazy_static! {
    // tunnel references to be kept until explicit close to prevent python gc from dropping them.
    // the tunnel wrapper object, and the underlying tunnel, both have references to the Session
    // so the Session is safe from premature dropping.
    static ref GLOBAL_TUNNELS: Mutex<HashMap<String,Arc<Storage>>> = Mutex::new(HashMap::new());
}

/// Stores the tunnel and session references to be kept until explicit close.
struct Storage {
    tunnel: Arc<Mutex<dyn ExtendedTunnel>>,
    session: Session,
    tun_meta: Arc<TunnelMetadata>,
}

struct TunnelMetadata {
    id: String,
    forwards_to: String,
    metadata: String,
    url: Option<String>,
    proto: Option<String>,
    labels: HashMap<String, String>,
}

/// The TunnelExt cannot be turned into an object since it contains generics, so implementing
/// a proxy trait without generics which can be the dyn type stored in the global map.
#[async_trait]
pub trait ExtendedTunnel: Tunnel {
    async fn fwd_tcp(&mut self, addr: String) -> CoreResult<(), io::Error>;
    async fn fwd_pipe(&mut self, addr: String) -> CoreResult<(), io::Error>;
}

/// An ngrok tunnel.
#[pyclass]
#[derive(Clone)]
pub(crate) struct NgrokTunnel {
    session: Session,
    tun_meta: Arc<TunnelMetadata>,
}

macro_rules! make_tunnel_type {
    // the common (non-labeled) branch
    ($(#[$outer:meta])* $wrapper:ident, $tunnel:tt, common) => {

        $(#[$outer])*
        #[allow(dead_code)]
        pub(crate) struct $wrapper {
        }

        #[allow(dead_code)]
        impl $wrapper {
            pub(crate) async fn new_tunnel(session: Session, raw_tunnel: $tunnel) -> NgrokTunnel {
                let id = raw_tunnel.id().to_string();
                let tun_meta = Arc::new(TunnelMetadata {
                    id: id.clone(),
                    forwards_to: raw_tunnel.forwards_to().to_string(),
                    metadata: raw_tunnel.metadata().to_string(),
                    url: Some(raw_tunnel.url().to_string()),
                    proto: Some(raw_tunnel.proto().to_string()),
                    labels: HashMap::new(),
                });
                info!("Created tunnel {id:?} with url {:?}", raw_tunnel.url());
                // keep a tunnel reference until an explicit call to close to prevent python gc dropping it
                let storage = Arc::new(Storage {
                    tunnel: Arc::new(Mutex::new(raw_tunnel)),
                    session,
                    tun_meta,
                });
                GLOBAL_TUNNELS.lock().await.insert(id, storage.clone());
                // create the user-facing object
                NgrokTunnel::from_storage(&storage)
            }
        }

        make_tunnel_type!($wrapper, $tunnel);
    };

    // the labeled branch
    ($(#[$outer:meta])* $wrapper:ident, $tunnel:tt, label) => {
        $(#[$outer])*
        #[allow(dead_code)]
        pub(crate) struct $wrapper {
        }

        #[allow(dead_code)]
        impl $wrapper {
            pub(crate) async fn new_tunnel(session: Session, raw_tunnel: $tunnel) -> NgrokTunnel {
                let id = raw_tunnel.id().to_string();
                let tun_meta = Arc::new(TunnelMetadata {
                    id: id.clone(),
                    forwards_to: raw_tunnel.forwards_to().to_string(),
                    metadata: raw_tunnel.metadata().to_string(),
                    url: None,
                    proto: None,
                    labels: raw_tunnel.labels().clone(),
                });
                info!("Created tunnel {id:?} with labels {:?}", tun_meta.labels);
                // keep a tunnel reference until an explicit call to close to prevent python gc dropping it
                let storage = Arc::new(Storage {
                    tunnel: Arc::new(Mutex::new(raw_tunnel)),
                    session,
                    tun_meta,
                });
                GLOBAL_TUNNELS.lock().await.insert(id, storage.clone());
                // create the user-facing object
                NgrokTunnel::from_storage(&storage)
            }
        }

        make_tunnel_type!($wrapper, $tunnel);
    };

    // all tunnels get these
    ($wrapper:ident, $tunnel:tt) => {
        #[async_trait]
        impl ExtendedTunnel for $tunnel {
            async fn fwd_tcp(&mut self, addr: String) -> CoreResult<(), io::Error> {
                self.forward_tcp(addr).await
            }
            async fn fwd_pipe(&mut self, addr: String) -> CoreResult<(), io::Error> {
                self.forward_pipe(addr).await
            }
        }
    };
}

impl NgrokTunnel {
    /// Create NgrokTunnel from Storage
    fn from_storage(storage: &Arc<Storage>) -> NgrokTunnel {
        // create the user-facing object
        NgrokTunnel {
            session: storage.session.clone(),
            tun_meta: storage.tun_meta.clone(),
        }
    }
}

#[pymethods]
#[allow(dead_code)]
impl NgrokTunnel {
    /// Returns a tunnel's unique ID.
    pub fn id(&self) -> String {
        self.tun_meta.id.clone()
    }

    /// The URL that this tunnel backs.
    pub fn url(&self) -> Option<String> {
        self.tun_meta.url.clone()
    }

    /// The protocol of the endpoint that this tunnel backs.
    pub fn proto(&self) -> Option<String> {
        self.tun_meta.proto.clone()
    }

    /// The labels this tunnel was started with.
    pub fn labels(&self) -> HashMap<String, String> {
        self.tun_meta.labels.clone()
    }

    /// Returns a human-readable string presented in the ngrok dashboard
    /// and the Tunnels API. Use the `HttpTunnelBuilder::forwards_to <https://docs.rs/ngrok/0.11.0/ngrok/config/struct.HttpTunnelBuilder.html#method.forwards_to>`_,
    /// `TcpTunnelBuilder::forwards_to <https://docs.rs/ngrok/0.11.0/ngrok/config/struct.TcpTunnelBuilder.html#method.forwards_to>`_, etc. to set this value
    /// explicitly.
    pub fn forwards_to(&self) -> String {
        self.tun_meta.forwards_to.clone()
    }

    /// Returns the arbitrary metadata string for this tunnel.
    pub fn metadata(&self) -> String {
        self.tun_meta.metadata.clone()
    }

    /// Forward incoming tunnel connections to the provided TCP address.
    pub fn forward_tcp<'a>(&self, py: Python<'a>, addr: String) -> PyResult<&'a PyAny> {
        let id = self.tun_meta.id.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move { forward_tcp(&id, addr).await })
    }

    /// Forward incoming tunnel connections to the provided file socket path.
    /// On Linux/Darwin addr can be a unix domain socket path, e.g. "/tmp/ngrok.sock"
    /// On Windows addr can be a named pipe, e.e. "\\\\.\\pipe\\an_ngrok_pipe"
    pub fn forward_pipe<'a>(&self, py: Python<'a>, addr: String) -> PyResult<&'a PyAny> {
        let id = self.tun_meta.id.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move { forward_pipe(&id, addr).await })
    }

    /// Close the tunnel.
    ///
    /// This is an RPC call that must be `.await`ed.
    /// It is equivalent to calling `Session::close_tunnel` with this
    /// tunnel's ID.
    pub fn close<'a>(&self, py: Python<'a>) -> PyResult<&'a PyAny> {
        let session = self.session.clone();
        let id = self.tun_meta.id.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            debug!("{} closing, id: {id:?}", stringify!($wrapper));

            // we may not be able to lock our reference to the tunnel due to the forward_* calls which
            // continuously accept-loop while the tunnel is active, so calling close on the Session.
            let res = session
                .close_tunnel(id.clone())
                .await
                .map_err(|e| py_ngrok_err("error closing tunnel", &e));

            // drop our internal reference to the tunnel after awaiting close
            remove_global_tunnel(&id).await?;

            res
        })
    }
}

// Methods designed to act like a native socket
#[pymethods]
#[allow(dead_code)]
impl NgrokTunnel {
    // for aiohttp case, proxy calls to socket
    #[getter]
    pub fn get_family(&self, py: Python) -> PyResult<Py<PyAny>> {
        self.get_sock_attr(py, intern!(py, "family"))
    }

    pub fn getsockname(&self, py: Python) -> PyResult<Py<PyAny>> {
        self.get_sock_attr(py, intern!(py, "getsockname"))?
            .call0(py)
    }

    #[getter]
    pub fn get_type(&self, py: Python) -> PyResult<Py<PyAny>> {
        self.get_sock_attr(py, intern!(py, "type"))
    }

    pub fn setblocking(&self, py: Python, blocking: bool) -> PyResult<Py<PyAny>> {
        let args = PyTuple::new(py, [blocking]);
        self.get_sock_attr(py, intern!(py, "setblocking"))?
            .call1(py, args)
    }

    pub fn fileno(&self, py: Python) -> PyResult<Py<PyAny>> {
        self.get_sock_attr(py, intern!(py, "fileno"))?.call0(py)
    }

    pub fn accept(&self, py: Python) -> PyResult<Py<PyAny>> {
        self.get_sock_attr(py, intern!(py, "accept"))?.call0(py)
    }

    pub fn gettimeout(&self, py: Python) -> PyResult<Py<PyAny>> {
        self.get_sock_attr(py, intern!(py, "gettimeout"))?.call0(py)
    }

    pub fn listen(
        &self,
        py: Python,
        backlog: i32,
        listen_attr: Option<&str>,
    ) -> PyResult<Py<PyAny>> {
        // call listen on socket
        let args = PyTuple::new(py, [backlog]);
        let listen_string = PyString::new(py, listen_attr.unwrap_or("listen"));
        let result = self.get_sock_attr(py, listen_string)?.call1(py, args);

        // set up forwarding depending on socket type
        let sockname = self.getsockname(py)?;
        let socket = PyModule::import(py, "socket")?;
        // windows does not have AF_UNIX enum at all
        let af_unix = socket.getattr(intern!(py, "AF_UNIX"));
        if let Ok(af_unix) = af_unix {
            if self.get_family(py)?.as_ref(py).eq(af_unix)? {
                // pipe
                let sockname_str: &PyString = sockname.downcast(py)?;
                self.forward_pipe(py, sockname_str.to_string())?;
                return result;
            }
        }
        // fallback to tcp
        let sockname_tuple: &PyTuple = sockname.downcast(py)?;
        self.forward_tcp(py, format!("localhost:{}", sockname_tuple.get_item(1)?))?;
        result
    }

    // For uvicorn case, generate a file descriptor for a listening socket
    #[getter]
    pub fn get_fd(&self, py: Python) -> PyResult<Py<PyAny>> {
        self.listen(py, 0, None)?;
        self.fileno(py)
    }

    // Get or create the python socket to use for this tunnel, and return an attribute of it.
    fn get_sock_attr(&self, py: Python, attr: &PyString) -> PyResult<Py<PyAny>> {
        self.get_sock(py)?.getattr(py, attr)
    }

    fn get_sock(&self, py: Python) -> PyResult<Py<PyAny>> {
        let map: &PyDict = SOCK_CELL
            .get_or_init(py, || PyDict::new(py).into())
            .extract(py)?;
        let maybe_socket = map.get_item(&self.tun_meta.id);
        Ok(match maybe_socket {
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
                map.set_item(self.tun_meta.id.clone(), res.clone())?;
                res
            }
        })
    }

    // Fulfill the iterator protocol so aiohttp will grab the wrap object.
    // This prevents the windows proactor framework from trying to use a weakref
    // directly to the rust tunnel, which is not supported until ABI 3.9.
    #[cfg(target_os = "windows")]
    fn __iter__(self_: PyRef<'_, Self>, py: Python) -> PyResult<Py<Iter>> {
        // Wrap self in a native python object to support weakref
        let burrito = wrap_object(py, self_.into_py(py))?;
        let iter = Iter {
            inner: vec![burrito].into_iter(),
        };
        Py::new(py, iter)
    }
}

#[allow(unused_mut)]
impl Drop for NgrokTunnel {
    fn drop(&mut self) {
        debug!("NgrokTunnel finalize, id: {}", self.tun_meta.id);
    }
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

pub async fn forward_tcp(id: &String, addr: String) -> PyResult<()> {
    info!("Tunnel {id:?} TCP forwarding to {addr:?}");
    let res = get_storage_by_id(id)
        .await?
        .tunnel
        .lock()
        .await
        .fwd_tcp(addr)
        .await;

    debug!("forward_tcp returning");
    canceled_is_ok(res)
}

pub async fn forward_pipe(id: &String, addr: String) -> PyResult<()> {
    info!("Tunnel {id:?} Pipe forwarding to {addr:?}");
    let res = get_storage_by_id(id)
        .await?
        .tunnel
        .lock()
        .await
        .fwd_pipe(addr)
        .await;

    debug!("forward_pipe returning");
    canceled_is_ok(res)
}

fn canceled_is_ok(input: CoreResult<(), io::Error>) -> PyResult<()> {
    match input {
        Ok(_) => Ok(()),
        Err(e) => {
            if let Some(source) = e
                .source()
                .and_then(|s| s.downcast_ref::<Arc<ConnectError>>())
            {
                if let ConnectError::Canceled = **source {
                    debug!("Reconnect was canceled, session is closing, returning Ok");
                    return Ok(());
                }
            }

            Err(py_err(format!("error forwarding: {e:?}")))
        }
    }
}

async fn get_storage_by_id(id: &String) -> PyResult<Arc<Storage>> {
    // we must clone the Arc before any locking so there is a local reference
    // to the mutex to unlock if the tunnel wrapper struct is dropped.
    Ok(GLOBAL_TUNNELS
        .lock()
        .await
        .get(id)
        .ok_or(py_err("Tunnel is no longer running"))?
        .clone()) // required clone
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
                debug!("closing socket: {}", id);
                existing.call_method0("close")?;

                // delete reference
                dict.del_item(id)?;
            }
        }
        Ok(())
    })
}

/// Close a tunnel with the given url, or all tunnels if no url is defined.
#[allow(dead_code)]
pub(crate) async fn close_url(url: Option<String>) -> PyResult<()> {
    let mut close_ids: Vec<String> = vec![];
    let tunnels = GLOBAL_TUNNELS.lock().await;
    for (id, storage) in tunnels.iter() {
        debug!("tunnel: {}", id);
        if url.as_ref().is_none() || url == storage.tun_meta.url {
            debug!("closing tunnel: {}", id);
            storage
                .session
                .close_tunnel(id)
                .await
                .map_err(|e| py_ngrok_err("error closing tunnel", &e))?;
            close_ids.push(id.clone());
        }
    }
    drop(tunnels); // unlock GLOBAL_TUNNELS

    // remove references entirely
    for id in close_ids {
        remove_global_tunnel(&id).await?;
    }
    Ok(())
}

/// Make a list of all tunnels by iterating over the global tunnel map and creating an NgrokTunnel from each.
pub(crate) async fn list_tunnels(session_id: Option<String>) -> PyResult<Vec<NgrokTunnel>> {
    let mut tunnels: Vec<NgrokTunnel> = vec![];
    for (_, storage) in GLOBAL_TUNNELS.lock().await.iter() {
        // filter by session_id, if provided
        if let Some(session_id) = session_id.as_ref() {
            if session_id.ne(&storage.session.id()) {
                continue;
            }
        }
        // create a new NgrokTunnel from the storage
        tunnels.push(NgrokTunnel::from_storage(storage));
    }
    Ok(tunnels)
}

/// Retrieve a list of non-closed tunnels, in no particular order.
#[pyfunction]
pub fn get_tunnels(py: Python) -> PyResult<Py<PyAny>> {
    // move to async, handling if there is an async loop running or not
    wrapper::loop_wrap(py, None, "    return await ngrok.async_tunnels()")
}

#[pyfunction]
pub fn async_tunnels(py: Python) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async move { list_tunnels(None).await })
}

// Helper class to implement the iterator protocol for tunnel sockets.
#[pyclass]
struct Iter {
    inner: std::vec::IntoIter<Py<PyAny>>,
}

#[pymethods]
impl Iter {
    #[allow(clippy::self_named_constructors)]
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<Py<PyAny>> {
        slf.inner.next()
    }
}
