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
    tunnel::TcpTunnel,
    Session,
};
use async_trait::async_trait;
use futures::prelude::*;
use lazy_static::lazy_static;
use ngrok::{
    forwarder::Forwarder,
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
};
use regex::Regex;
use tokio::{
    sync::Mutex,
    task::JoinHandle,
};
use tracing::{
    debug,
    info,
};
use url::Url;

#[cfg(target_os = "windows")]
use crate::wrapper::wrap_object;
use crate::{
    py_err,
    py_ngrok_err,
    wrapper::{
        self,
        bound_default_tcp_socket,
        bound_default_unix_socket,
    },
};

/// Python dictionary of id's to sockets.
static SOCK_CELL: GILOnceCell<Py<PyDict>> = GILOnceCell::new();

// no forward host section to allow for relative unix paths
pub(crate) const UNIX_PREFIX: &str = "unix:";
pub(crate) const TCP_PREFIX: &str = "tcp://";

lazy_static! {
    // Listener references to be kept until explicit close to prevent python gc from dropping them.
    // the Listener wrapper object, and the underlying Listener, both have references to the Session
    // so the Session is safe from premature dropping.
    static ref GLOBAL_LISTENERS: Mutex<HashMap<String,Arc<Storage>>> = Mutex::new(HashMap::new());
}

/// Stores the listener and session references to be kept until explicit close.
struct Storage {
    listener: Option<Arc<Mutex<dyn ExtendedListener>>>,
    forwarder: Option<Arc<Mutex<dyn ExtendedForwarder>>>,
    session: Session,
    tun_meta: Arc<ListenerInfo>,
}

struct ListenerInfo {
    id: String,
    forwards_to: String,
    metadata: String,
    url: Option<String>,
    proto: Option<String>,
    labels: HashMap<String, String>,
}

/// The upstream object cannot be turned into an object since it contains generics, so implementing
/// a proxy trait without generics which can be the dyn type stored in the global map.
#[async_trait]
pub trait ExtendedListener: Send {
    async fn fwd(&mut self, url: Url) -> CoreResult<(), io::Error>;
}

pub trait ExtendedForwarder: Send {
    fn get_join(&mut self) -> &mut JoinHandle<Result<(), io::Error>>;
}

/// An ngrok listener.
#[pyclass]
#[derive(Clone)]
pub(crate) struct Listener {
    session: Session,
    tun_meta: Arc<ListenerInfo>,
}

macro_rules! make_listener_type {
    // the common (non-labeled) branch
    ($(#[$outer:meta])* $wrapper:ident, $listener:tt, common) => {

        $(#[$outer])*
        #[allow(dead_code)]
        pub(crate) struct $wrapper {
        }

        #[allow(dead_code)]
        impl $wrapper {
            pub(crate) async fn new_listener(session: Session, raw_listener: $listener) -> Listener {
                let id = raw_listener.id().to_string();
                let tun_meta = Arc::new(ListenerInfo {
                    id: id.clone(),
                    forwards_to: raw_listener.forwards_to().to_string(),
                    metadata: raw_listener.metadata().to_string(),
                    url: Some(raw_listener.url().to_string()),
                    proto: Some(raw_listener.proto().to_string()),
                    labels: HashMap::new(),
                });
                info!("Created listener {id:?} with url {:?}", raw_listener.url());
                // keep a listener reference until an explicit call to close to prevent python gc dropping it
                let storage = Arc::new(Storage {
                    listener: Some(Arc::new(Mutex::new(raw_listener))),
                    forwarder: None,
                    session,
                    tun_meta,
                });
                GLOBAL_LISTENERS.lock().await.insert(id, storage.clone());
                // create the user-facing object
                Listener::from_storage(&storage)
            }

            pub(crate) async fn new_forwarder(session: Session, forwarder: Forwarder<$listener>) -> Listener {
                let id = forwarder.id().to_string();
                let tun_meta = Arc::new(ListenerInfo {
                    id: id.clone(),
                    forwards_to: forwarder.forwards_to().to_string(),
                    metadata: forwarder.metadata().to_string(),
                    url: Some(forwarder.url().to_string()),
                    proto: Some(forwarder.proto().to_string()),
                    labels: HashMap::new(),
                });
                info!("Created listener {id:?} with url {:?}", forwarder.url());
                // keep a listener reference until an explicit call to close to prevent python gc dropping it
                let storage = Arc::new(Storage {
                    listener: None,
                    forwarder: Some(Arc::new(Mutex::new(forwarder))),
                    session,
                    tun_meta,
                });
                GLOBAL_LISTENERS.lock().await.insert(id, storage.clone());
                // create the user-facing object
                Listener::from_storage(&storage)
            }
        }

        make_listener_type!($wrapper, $listener);
    };

    // the labeled branch
    ($(#[$outer:meta])* $wrapper:ident, $listener:tt, label) => {
        $(#[$outer])*
        #[allow(dead_code)]
        pub(crate) struct $wrapper {
        }

        #[allow(dead_code)]
        impl $wrapper {
            pub(crate) async fn new_listener(session: Session, raw_listener: $listener) -> Listener {
                let id = raw_listener.id().to_string();
                let tun_meta = Arc::new(ListenerInfo {
                    id: id.clone(),
                    forwards_to: raw_listener.forwards_to().to_string(),
                    metadata: raw_listener.metadata().to_string(),
                    url: None,
                    proto: None,
                    labels: raw_listener.labels().clone(),
                });
                info!("Created listener {id:?} with labels {:?}", tun_meta.labels);
                // keep a listener reference until an explicit call to close to prevent python gc dropping it
                let storage = Arc::new(Storage {
                    listener: Some(Arc::new(Mutex::new(raw_listener))),
                    forwarder: None,
                    session,
                    tun_meta,
                });
                GLOBAL_LISTENERS.lock().await.insert(id, storage.clone());
                // create the user-facing object
                Listener::from_storage(&storage)
            }

            pub(crate) async fn new_forwarder(session: Session, forwarder: Forwarder<$listener>) -> Listener {
                let id = forwarder.id().to_string();
                let tun_meta = Arc::new(ListenerInfo {
                    id: id.clone(),
                    forwards_to: forwarder.forwards_to().to_string(),
                    metadata: forwarder.metadata().to_string(),
                    url: None,
                    proto: None,
                    labels: forwarder.labels().clone(),
                });
                info!("Created listener {id:?} with labels {:?}", tun_meta.labels);
                // keep a listener reference until an explicit call to close to prevent python gc dropping it
                let storage = Arc::new(Storage {
                    listener: None,
                    forwarder: Some(Arc::new(Mutex::new(forwarder))),
                    session,
                    tun_meta,
                });
                GLOBAL_LISTENERS.lock().await.insert(id, storage.clone());
                // create the user-facing object
                Listener::from_storage(&storage)
            }
        }

        make_listener_type!($wrapper, $listener);
    };

    // all listeners get these
    ($wrapper:ident, $listener:tt) => {
        #[async_trait]
        impl ExtendedListener for $listener {
            #[allow(deprecated)]
            async fn fwd(&mut self, url: Url) -> CoreResult<(), io::Error> {
                ngrok::prelude::TunnelExt::forward(self, url).await
            }
        }

        impl ExtendedForwarder for Forwarder<$listener> {
            fn get_join(&mut self) -> &mut JoinHandle<Result<(), io::Error>> {
                self.join()
            }
        }
    };
}

impl Listener {
    /// Create Listener from Storage
    fn from_storage(storage: &Arc<Storage>) -> Listener {
        // create the user-facing object
        Listener {
            session: storage.session.clone(),
            tun_meta: storage.tun_meta.clone(),
        }
    }
}

#[pymethods]
#[allow(dead_code)]
impl Listener {
    /// Returns a listener's unique ID.
    pub fn id(&self) -> String {
        self.tun_meta.id.clone()
    }

    /// The URL that this listener backs.
    pub fn url(&self) -> Option<String> {
        self.tun_meta.url.clone()
    }

    /// The protocol of the endpoint that this listener backs.
    pub fn proto(&self) -> Option<String> {
        self.tun_meta.proto.clone()
    }

    /// The labels this listener was started with.
    pub fn labels(&self) -> HashMap<String, String> {
        self.tun_meta.labels.clone()
    }

    /// Returns a human-readable string presented in the ngrok dashboard
    /// and the API. Use the :meth:`HttpListenerBuilder.forwards_to`, :meth:`TcpListenerBuilder.forwards_to`, etc.
    /// to set this value explicitly.
    ///
    /// To automatically forward connections, you can use :any:`listen_and_forward`,
    /// or :any:`listen_and_serve` on the Listener Builder. These methods will also set this `forwards_to` value.
    pub fn forwards_to(&self) -> String {
        self.tun_meta.forwards_to.clone()
    }

    /// Returns the arbitrary metadata string for this listener.
    pub fn metadata(&self) -> String {
        self.tun_meta.metadata.clone()
    }

    /// .. deprecated:: 0.10.0
    /// Use :meth:`listen_and_forward` on Listener builders instead,
    /// for example :meth:`HttpListenerBuilder.listen_and_forward`.
    ///
    /// Forward incoming listener connections. This can be either a TCP address or a file socket path.
    /// For file socket paths on Linux/Darwin, addr can be a unix domain socket path, e.g. "/tmp/ngrok.sock".
    pub fn forward<'a>(&self, py: Python<'a>, addr: String) -> PyResult<&'a PyAny> {
        let id = self.tun_meta.id.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move { forward(&id, addr).await })
    }

    /// Wait for the forwarding task to exit.
    pub fn join<'a>(&mut self, py: Python<'a>) -> PyResult<&'a PyAny> {
        let id = self.tun_meta.id.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let forwarder_option = &get_storage_by_id(&id).await?.forwarder;
            if let Some(forwarder_mutex) = forwarder_option {
                forwarder_mutex
                    .lock()
                    .await
                    .get_join()
                    .fuse()
                    .await
                    .map_err(|e| py_err(format!("error on join: {e:?}")))?
                    .map_err(|e| py_err(format!("error on join: {e:?}")))
            } else {
                Err(py_err("Listener is not joinable"))
            }
        })
    }

    /// Close the listener.
    ///
    /// This is an RPC call that must be `.await`ed.
    /// It is equivalent to calling `Session::close_listener` with this
    /// listener's ID.
    pub fn close<'a>(&self, py: Python<'a>) -> PyResult<&'a PyAny> {
        let session = self.session.clone();
        let id = self.tun_meta.id.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            debug!("{} closing, id: {id:?}", stringify!($wrapper));

            // we may not be able to lock our reference to the listener due to the forward_* calls which
            // continuously accept-loop while the listener is active, so calling close on the Session.
            let res = session
                .close_tunnel(id.clone())
                .await
                .map_err(|e| py_ngrok_err("error closing listener", &e));

            // drop our internal reference to the listener after awaiting close
            remove_global_listener(&id).await?;

            res
        })
    }
}

// Methods designed to act like a native socket
#[pymethods]
#[allow(dead_code)]
impl Listener {
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
                // unix socket
                let sockname_str: &PyString = sockname.downcast(py)?;
                self.forward(py, format!("{UNIX_PREFIX}{sockname_str}"))?;
                return result;
            }
        }
        // fallback to tcp
        let sockname_tuple: &PyTuple = sockname.downcast(py)?;
        self.forward(py, format!("localhost:{}", sockname_tuple.get_item(1)?))?;
        result
    }

    // For uvicorn case, generate a file descriptor for a listening socket
    #[getter]
    pub fn get_fd(&self, py: Python) -> PyResult<Py<PyAny>> {
        self.listen(py, 0, None)?;
        self.fileno(py)
    }

    // Get or create the python socket to use for this listener, and return an attribute of it.
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
                // try unix first, fall back to tcp
                let res = match bound_default_unix_socket(py) {
                    Ok(res) => res,
                    Err(error) => {
                        debug!("error binding to unix: {}", error);
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
    // directly to the rust listener, which is not supported until ABI 3.9.
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
impl Drop for Listener {
    fn drop(&mut self) {
        debug!("Listener finalize, id: {}", self.tun_meta.id);
    }
}

make_listener_type! {
    /// An ngrok listener backing an HTTP endpoint.
    HttpListener, HttpTunnel, common
}
make_listener_type! {
    /// An ngrok listener backing a TCP endpoint.
    TcpListener, TcpTunnel, common
}
make_listener_type! {
    /// An ngrok listener bcking a TLS endpoint.
    TlsListener, TlsTunnel, common
}
make_listener_type! {
    /// A labeled ngrok listener.
    LabeledListener, LabeledTunnel, label
}

pub async fn forward(id: &String, mut addr: String) -> PyResult<()> {
    let tun_option = &get_storage_by_id(id).await?.listener;
    if let Some(tun) = tun_option {
        // if addr is not a full url, choose a default protocol
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^[a-z0-9\-\.]+:\d+$").unwrap();
        }
        if !addr.contains(':') || RE.find(&addr).is_some() {
            if addr.contains('/') {
                addr = format!("{UNIX_PREFIX}{addr}")
            } else {
                addr = format!("{TCP_PREFIX}{addr}")
            }
        }
        // parse to a url
        let url = Url::parse(addr.as_str())
            .map_err(|e| py_err(format!("Cannot parse address: {addr}, error: {e}")))?;

        info!("Listener {id:?} forwarding to {:?}", url.to_string());
        let res = tun.lock().await.fwd(url).await;

        debug!("forward returning");
        canceled_is_ok(res)
    } else {
        Err(py_err("listener is not forwardable"))
    }
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
    // to the mutex to unlock if the listener wrapper struct is dropped.
    Ok(GLOBAL_LISTENERS
        .lock()
        .await
        .get(id)
        .ok_or(py_err("Listener is no longer running"))?
        .clone()) // required clone
}

/// Delete any reference to the listener id
pub(crate) async fn remove_global_listener(id: &String) -> PyResult<()> {
    GLOBAL_LISTENERS.lock().await.remove(id);

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

/// Close a listener with the given url, or all listeners if no url is defined.
#[allow(dead_code)]
pub(crate) async fn close_url(url: Option<String>) -> PyResult<()> {
    let mut close_ids: Vec<String> = vec![];
    let listeners = GLOBAL_LISTENERS.lock().await;
    for (id, storage) in listeners.iter() {
        debug!("listener: {}", id);
        if url.as_ref().is_none() || url == storage.tun_meta.url {
            debug!("closing listener: {}", id);
            storage
                .session
                .close_tunnel(id)
                .await
                .map_err(|e| py_ngrok_err("error closing listener", &e))?;
            close_ids.push(id.clone());
        }
    }
    drop(listeners); // unlock GLOBAL_LISTENERS

    // remove references entirely
    for id in close_ids {
        remove_global_listener(&id).await?;
    }
    Ok(())
}

/// Make a list of all Listeners by iterating over the global Listener map and creating an Listener from each.
pub(crate) async fn list_listeners(session_id: Option<String>) -> PyResult<Vec<Listener>> {
    let mut listeners: Vec<Listener> = vec![];
    for (_, storage) in GLOBAL_LISTENERS.lock().await.iter() {
        // filter by session_id, if provided
        if let Some(session_id) = session_id.as_ref() {
            if session_id.ne(&storage.session.id()) {
                continue;
            }
        }
        // create a new Listener from the storage
        listeners.push(Listener::from_storage(storage));
    }
    Ok(listeners)
}

/// Retrieve a list of non-closed listeners, in no particular order.
#[pyfunction]
pub fn get_listeners(py: Python) -> PyResult<Py<PyAny>> {
    // move to async, handling if there is an async loop running or not
    wrapper::loop_wrap(py, None, "    return await ngrok.async_listeners()")
}

#[pyfunction]
pub fn async_listeners(py: Python) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async move { list_listeners(None).await })
}

// Helper class to implement the iterator protocol for listener sockets.
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
