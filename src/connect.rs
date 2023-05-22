use lazy_static::lazy_static;
use log::info;
use pyo3::{
    pyfunction,
    types::{
        PyBool,
        PyByteArray,
        PyDict,
        PyFloat,
        PyInt,
        PyList,
        PyString,
    },
    IntoPy,
    Py,
    PyAny,
    PyCell,
    PyDowncastError,
    PyErr,
    PyObject,
    PyResult,
    Python,
};
use tokio::sync::Mutex;

use crate::{
    py_err,
    session::{
        NgrokSession,
        NgrokSessionBuilder,
    },
    tunnel::{
        self,
        NgrokTunnel,
    },
    tunnel_builder::{
        NgrokHttpTunnelBuilder,
        NgrokLabeledTunnelBuilder,
        NgrokTcpTunnelBuilder,
        NgrokTlsTunnelBuilder,
    },
    wrapper,
};

lazy_static! {
    // Save a user-facing NgrokSession to use for connect use cases
    pub(crate) static ref SESSION: Mutex<Option<NgrokSession>> = Mutex::new(None);
}

const PIPE_PREFIX: &str = "pipe:";

/// Single string configuration
macro_rules! plumb {
    ($builder:tt, $self:tt, $config:tt, $name:tt) => {
        plumb!($builder, $self, $config, $name, $name)
    };
    ($builder:tt, $self:tt, $config:tt, $name:tt, $config_name:tt) => {
        if let Some(v) = $config.get_item(stringify!($config_name)) {
            $builder::$name($self.borrow_mut(), get_string(v)?);
        }
    };
}

/// Boolean configuration
macro_rules! plumb_bool {
    ($builder:tt, $self:tt, $config:tt, $name:tt) => {
        plumb_bool!($builder, $self, $config, $name, $name)
    };
    ($builder:tt, $self:tt, $config:tt, $name:tt, $config_name:tt) => {
        if let Some(v) = $config.get_item(stringify!($config_name)) {
            if get_bool(v)? {
                $builder::$name($self.borrow_mut());
            }
        }
    };
}

/// Vector configuration
macro_rules! plumb_vec {
    ($builder:tt, $self:tt, $config:tt, $name:tt) => {
        plumb_vec!($builder, $self, $config, $name, $name)
    };
    ($builder:tt, $self:tt, $config:tt, $name:tt, $config_name:tt) => {
        if let Some(v) = $config.get_item(stringify!($config_name)) {
            for val in get_list(v)? {
                $builder::$name($self.borrow_mut(), get_string(val)?);
            }
        }
    };
    ($builder:tt, $self:tt, $config:tt, $name:tt, $config_name:tt, vecu8) => {
        if let Some(v) = $config.get_item(stringify!($config_name)) {
            for val in get_list(v)? {
                $builder::$name($self.borrow_mut(), get_byte_array(val)?);
            }
        }
    };
    ($builder:tt, $self:tt, $config:tt, $name:tt, $config_name:tt, $split:tt) => {
        if let Some(v) = $config.get_item(stringify!($config_name)) {
            for val in get_list(v)? {
                let s = get_string(val)?;
                let (a, b) = s.split_once($split).expect("split of value failed: ${val}");
                $builder::$name($self.borrow_mut(), a.to_string(), b.to_string());
            }
        }
    };
}

/// All non-labeled tunnels have these common configuration options
macro_rules! config_common {
    ($builder:tt, $self:tt, $config:tt) => {
        plumb!($builder, $self, $config, metadata);
        plumb_vec!($builder, $self, $config, allow_cidr);
        plumb_vec!($builder, $self, $config, deny_cidr);
        plumb!($builder, $self, $config, proxy_proto);
        plumb!($builder, $self, $config, forwards_to);
    };
}

fn get_string(v: &PyAny) -> Result<String, PyErr> {
    v.downcast::<PyString>()?.extract::<String>()
}

fn get_bool(v: &PyAny) -> Result<bool, PyErr> {
    v.downcast::<PyBool>()?.extract::<bool>()
}

fn get_list(v: &PyAny) -> Result<Vec<&PyAny>, PyErr> {
    if v.is_instance(v.py().get_type::<PyList>())? {
        return v.downcast::<PyList>()?.extract::<Vec<&PyAny>>();
    }
    // turn scalars into lists
    Ok(vec![v])
}

fn get_str_list(v: Option<&PyAny>) -> Result<Option<Vec<String>>, PyErr> {
    // vectorize PyAny's, then convert them to Strings
    v.map(get_list)
        .transpose()?
        .map(|v| v.iter().map(|v| get_string(v)).collect())
        .transpose()
}

fn get_byte_array(v: &PyAny) -> Result<&PyByteArray, PyDowncastError> {
    v.downcast::<PyByteArray>()
}

/// Establish ngrok ingress, returning an tunnel object.
///
/// :param int, str or None addr: The address to forward traffic to, this can be an integer port, or a host:port string, e.g. "localhost:8080"
/// :param str or None proto: The protocol type of the tunnel, one of "http", "tcp", "tls", "labeled"
/// :param options: A dict of options to pass to the tunnel.
/// :return: A tunnel object.
#[pyfunction]
#[pyo3(signature = (addr=None, proto=None, **options), text_signature = "(addr=None, proto=None, **options)")]
pub fn connect(
    py: Python,
    addr: Option<&PyAny>,
    proto: Option<String>,
    options: Option<&PyDict>,
) -> PyResult<Py<PyAny>> {
    let kwargs: &PyDict = options.unwrap_or(PyDict::new(py));
    // decode address string
    let mut addr_str = "localhost:80".to_string();
    if let Some(a) = addr {
        if a.is_instance(py.get_type::<PyInt>())? {
            addr_str = format!("localhost:{}", a.downcast::<PyInt>()?.extract::<i32>()?);
        } else if a.is_instance(py.get_type::<PyString>())? {
            addr_str = a.downcast::<PyString>()?.extract::<String>()?;

            // Fix up an addr that mistakenly has a protocol or is missing a port
            let mut assume_port = 80;
            let original = addr_str.clone();
            if addr_str.starts_with("http://") {
                addr_str = addr_str.split_once("://").unwrap().1.to_string();
            }
            if addr_str.starts_with("https://") {
                addr_str = addr_str.split_once("://").unwrap().1.to_string();
                assume_port = 443;
            }
            if !addr_str.starts_with(PIPE_PREFIX) && !addr_str.contains(':') {
                addr_str = format!("{addr_str}:{assume_port}");
            }
            if original != addr_str {
                info!("Converted addr '{original}' to '{addr_str}'");
            }
        }
    }

    // package up args to kwargs
    if addr.is_some() || kwargs.get_item("addr").is_none() {
        kwargs.set_item("addr", addr_str)?;
    }
    if proto.is_some() {
        kwargs.set_item("proto", proto)?;
    }

    // Remove all None's from kwargs to avoid casting problems on keys we will ignore
    for k in kwargs.keys() {
        if let Some(v) = kwargs.get_item(k) {
            if v.is_none() {
                kwargs.del_item(k)?;
            } else if get_string(k)?.contains('.') {
                // handle cases like "oauth.provider" -> "oauth_provider"
                kwargs.del_item(k)?;
                kwargs.set_item(get_string(k)?.replace('.', "_"), v)?;
            }
        }
    }

    // move to async, handling if there is an async loop running or not
    wrapper::loop_wrap(
        py,
        Some(kwargs.into()),
        "    return await ngrok.async_connect(input)",
    )
}

#[pyfunction]
pub fn async_connect(py: Python, config: Py<PyDict>) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async move { do_connect(config).await })
}

fn configure_session(options: &Py<PyDict>) -> Result<NgrokSessionBuilder, PyErr> {
    Python::with_gil(|py: Python| {
        let s_builder = PyCell::new(py, NgrokSessionBuilder::new())?;
        let cfg = options.as_ref(py);
        type B = NgrokSessionBuilder;
        plumb!(B, s_builder, cfg, authtoken);
        plumb_bool!(B, s_builder, cfg, authtoken_from_env);
        plumb!(B, s_builder, cfg, metadata, session_metadata);
        Ok(s_builder.replace(NgrokSessionBuilder::new()))
    })
}

async fn do_connect(options: Py<PyDict>) -> PyResult<PyObject> {
    // Using a singleton session for connect use cases
    let mut opt = SESSION.lock().await;
    if opt.is_none() {
        opt.replace(configure_session(&options)?.async_connect().await?);
    }
    let session = opt.as_ref().unwrap();

    // decode address
    let addr = Python::with_gil(|py| -> PyResult<String> {
        // decode address string
        get_string(options.as_ref(py).get_item("addr").expect("addr set"))
    })?;

    // decode proto
    let proto = Python::with_gil(|py| -> PyResult<String> {
        Ok(match options.as_ref(py).get_item("proto") {
            Some(p) => get_string(p)?,
            None => "http".to_string(),
        })
    })?;

    // create tunnel
    let tunnel = match proto.as_str() {
        "http" => http_endpoint(session, addr, options).await,
        "tcp" => tcp_endpoint(session, addr, options).await,
        "tls" => tls_endpoint(session, addr, options).await,
        "labeled" => labeled_tunnel(session, addr, options).await,
        _ => Err(py_err(format!("unhandled protocol {proto:?}"))),
    }?;
    Ok(Python::with_gil(|py| tunnel.into_py(py)))
}

/// HTTP tunnel creation and forwarding
async fn http_endpoint(
    session: &NgrokSession,
    addr: String,
    options: Py<PyDict>,
) -> PyResult<NgrokTunnel> {
    let bld = Python::with_gil(|py: Python| {
        let bld = PyCell::new(py, session.http_endpoint())?;
        let cfg = options.as_ref(py);
        type B = NgrokHttpTunnelBuilder;
        config_common!(B, bld, cfg);
        plumb_vec!(B, bld, cfg, scheme, schemes);
        plumb!(B, bld, cfg, domain, hostname); // synonym for domain
        plumb!(B, bld, cfg, domain);
        plumb_vec!(B, bld, cfg, mutual_tlsca, mutual_tls_cas, vecu8);
        plumb_bool!(B, bld, cfg, compression);
        plumb_bool!(
            B,
            bld,
            cfg,
            websocket_tcp_conversion,
            websocket_tcp_converter
        );
        plumb_vec!(B, bld, cfg, request_header, request_header_add, ":");
        plumb_vec!(B, bld, cfg, response_header, response_header_add, ":");
        plumb_vec!(B, bld, cfg, remove_request_header, request_header_remove);
        plumb_vec!(B, bld, cfg, remove_response_header, response_header_remove);
        plumb_vec!(B, bld, cfg, basic_auth, basic_auth, ":");
        // circuit breaker
        if let Some(cb) = cfg.get_item("circuit_breaker") {
            let cb64 = cb.downcast::<PyFloat>()?.extract::<f64>()?;
            NgrokHttpTunnelBuilder::circuit_breaker(bld.borrow_mut(), cb64);
        }
        // oauth
        if let Some(provider) = cfg.get_item("oauth_provider") {
            NgrokHttpTunnelBuilder::oauth(
                bld.borrow_mut(),
                get_string(provider)?,
                get_str_list(cfg.get_item("oauth_allow_emails"))?,
                get_str_list(cfg.get_item("oauth_allow_domains"))?,
                get_str_list(cfg.get_item("oauth_scopes"))?,
            );
        }
        // oidc
        if let Some(issuer_url) = cfg.get_item("oidc_issuer_url") {
            let client_id = cfg.get_item("oidc_client_id").ok_or_else(|| {
                py_err("Missing client id for oidc. oidc_client_id must be set if oidc_issuer_url is set")
            })?;
            let client_secret = cfg.get_item("oidc_client_secret").ok_or_else(|| {
                py_err("Missing client secret for oidc. oidc_client_secret must be set if oidc_issuer_url is set")
            })?;
            NgrokHttpTunnelBuilder::oidc(
                bld.borrow_mut(),
                get_string(issuer_url)?,
                get_string(client_id)?,
                get_string(client_secret)?,
                get_str_list(cfg.get_item("oidc_allow_emails"))?,
                get_str_list(cfg.get_item("oidc_allow_domains"))?,
                get_str_list(cfg.get_item("oidc_scopes"))?,
            );
        }
        // webhook verification
        if let Some(provider) = cfg.get_item("verify_webhook_provider") {
            if let Some(secret) = cfg.get_item("verify_webhook_secret") {
                NgrokHttpTunnelBuilder::webhook_verification(
                    bld.borrow_mut(),
                    get_string(provider)?,
                    get_string(secret)?,
                );
            } else {
                return Err(py_err("Missing key for tls termination"));
            }
        }
        Ok::<_, PyErr>(bld.replace(session.http_endpoint()))
    })?;
    forward(bld.async_listen().await?, addr).await
}

/// TCP tunnel creation and forwarding
async fn tcp_endpoint(
    session: &NgrokSession,
    addr: String,
    options: Py<PyDict>,
) -> PyResult<NgrokTunnel> {
    let bld = Python::with_gil(|py: Python| {
        let bld = PyCell::new(py, session.tcp_endpoint())?;
        let cfg = options.as_ref(py);
        type B = NgrokTcpTunnelBuilder;
        config_common!(B, bld, cfg);
        plumb!(B, bld, cfg, remote_addr);
        Ok::<_, PyErr>(bld.replace(session.tcp_endpoint()))
    })?;
    forward(bld.async_listen().await?, addr).await
}

/// TLS tunnel creation and forwarding
async fn tls_endpoint(
    session: &NgrokSession,
    addr: String,
    options: Py<PyDict>,
) -> PyResult<NgrokTunnel> {
    let bld = Python::with_gil(|py: Python| {
        let bld = PyCell::new(py, session.tls_endpoint())?;
        let cfg = options.as_ref(py);
        type B = NgrokTlsTunnelBuilder;
        config_common!(B, bld, cfg);
        plumb!(B, bld, cfg, domain, hostname); // synonym for domain
        plumb!(B, bld, cfg, domain);
        plumb_vec!(B, bld, cfg, mutual_tlsca, mutual_tls_cas, vecu8);
        // tls termination
        if let Some(crt) = cfg.get_item("crt") {
            if let Some(key) = cfg.get_item("key") {
                NgrokTlsTunnelBuilder::termination(
                    bld.borrow_mut(),
                    get_byte_array(crt)?,
                    get_byte_array(key)?,
                );
            } else {
                return Err(py_err("Missing key for tls termination"));
            }
        }
        Ok::<_, PyErr>(bld.replace(session.tls_endpoint()))
    })?;
    forward(bld.async_listen().await?, addr).await
}

/// Labeled tunnel creation and forwarding
async fn labeled_tunnel(
    session: &NgrokSession,
    addr: String,
    options: Py<PyDict>,
) -> PyResult<NgrokTunnel> {
    let bld = Python::with_gil(|py: Python| {
        let bld = PyCell::new(py, session.labeled_tunnel())?;
        let cfg = options.as_ref(py);
        type B = NgrokLabeledTunnelBuilder;
        plumb!(B, bld, cfg, metadata);
        plumb_vec!(B, bld, cfg, label, labels, ":");
        Ok::<_, PyErr>(bld.replace(session.labeled_tunnel()))
    })?;
    forward(bld.async_listen().await?, addr).await
}

/// Background the tunnel forwarding
async fn forward(tunnel: NgrokTunnel, addr: String) -> PyResult<NgrokTunnel> {
    let id = tunnel.id();
    // move forwarding to another task
    tokio::spawn(async move {
        if addr.starts_with(PIPE_PREFIX) {
            tunnel::forward_pipe(&id, addr.clone().split_off(5)).await
        } else {
            tunnel::forward_tcp(&id, addr).await
        }
        .map(|_| ())
    });
    Ok(tunnel)
}

/// Shut down all tunnels and sessions.
#[pyfunction]
pub fn kill(py: Python) -> PyResult<Py<PyAny>> {
    disconnect(py, None)
}

/// Shut down tunnel with the given url, or if no url is given, shut down all tunnels.
///
/// :param str or None url: The url of the NgrokTunnel to disconnect, or None to disconnect all tunnels.
#[pyfunction]
#[pyo3(text_signature = "(url=None)")]
pub fn disconnect(py: Python, url: Option<Py<PyString>>) -> PyResult<Py<PyAny>> {
    // move to async, handling if there is an async loop running or not
    wrapper::loop_wrap(
        py,
        url.map(|u| u.into()),
        "    return await ngrok.async_disconnect(input)",
    )
}

#[pyfunction]
pub fn async_disconnect(py: Python, url: Option<String>) -> PyResult<&PyAny> {
    info!("async Disconnecting");
    pyo3_asyncio::tokio::future_into_py(py, async move {
        tunnel::close_url(url.clone()).await?;

        // if closing every tunnel, remove any stored session
        if url.is_none() {
            SESSION.lock().await.take();
        }

        Ok(())
    })
}
