use pyo3::{
    intern,
    pyfunction,
    types::{
        PyModule,
        PyString,
        PyTuple,
    },
    IntoPy,
    Py,
    PyAny,
    PyErr,
    PyResult,
    Python,
};

use crate::{
    py_err,
    tunnel::{
        TCP_PREFIX,
        UNIX_PREFIX,
    },
};

/// Create a path name to use for pipe forwarding.
/// This will be a file path in the temp directory on unix-like systems,
/// or a named pipe on Windows. Files will be removed at program exit.
#[pyfunction]
pub fn pipe_name(py: Python) -> PyResult<Py<PyAny>> {
    call_code(
        py,
        None,
        r###"
import atexit
import logging
import os
import random
import tempfile

path = '\\\\.\\pipe\\ngrok_pipe' if os.name == 'nt' else \
    "{}/tun-{}.sock".format(tempfile.gettempdir(), random.randrange(0,1000000))

def delete_socket():
    if os.path.exists(path):
        logging.info('deleting {}'.format(path))
        os.remove(path)

def run(input=None):
    atexit.register(delete_socket)
    return path
    "###,
    )
}

/// Create a default HTTP tunnel. Optionally pass in a connected NgrokSession to use.
///
/// Returns the tunnel if no async loop is running, otherwise returns a Task to await with a tunnel result.
///
/// :param session: The NgrokSession to use to create the tunnel.
/// :type session: NgrokSession or None
/// :return: The created tunnel.
/// :rtype: NgrokTunnel
#[pyfunction]
#[pyo3(text_signature = "(session=None)")]
pub fn default(py: Python, session: Option<Py<PyAny>>) -> PyResult<Py<PyAny>> {
    default_tunnel_with_return(py, session, "tunnel")
}

/// Create a default HTTP tunnel and get its file descriptor. Optionally pass in a connected NgrokSession to use.
///
/// Returns the file descriptor if no async loop is running, otherwise returns a Task to await with a file descriptor result.
///
/// :param session: The NgrokSession to use to create the tunnel.
/// :type session: NgrokSession or None
/// :return: The file descriptor of the created tunnel's forwarding socket.
/// :rtype: int
#[pyfunction]
#[pyo3(text_signature = "(session=None)")]
pub fn fd(py: Python, session: Option<Py<PyAny>>) -> PyResult<Py<PyAny>> {
    default_tunnel_with_return(py, session, "tunnel.fd")
}

/// Create a default HTTP tunnel and get its socket name. Optionally pass in a connected NgrokSession to use.
///
/// Returns the socket name if no async loop is running, otherwise returns a Task to await with a socket name result.
///
/// :param session: The NgrokSession to use to create the tunnel.
/// :type session: NgrokSession or None
/// :return: The name of the created tunnel's forwarding socket.
/// :rtype: str
#[pyfunction]
#[pyo3(text_signature = "(session=None)")]
pub fn getsockname(py: Python, session: Option<Py<PyAny>>) -> PyResult<Py<PyAny>> {
    default_tunnel_with_return(py, session, "tunnel.getsockname()")
}

fn default_tunnel_with_return(
    py: Python,
    session: Option<Py<PyAny>>,
    return_str: &str,
) -> PyResult<Py<PyAny>> {
    loop_wrap(
        py,
        session,
        &format!(
            r###"
    if input is None:
        input = await NgrokSessionBuilder().authtoken_from_env().connect()
    tunnel = await input.http_endpoint().listen()
    return {return_str}
    "###
        ),
    )
}

/// Create and return a listening default HTTP tunnel.
/// Optionally pass in an object with at "server_address" attribute,
/// such as a http.server.HTTPServer, and the tunnel will
/// forward TCP to that server_address. Optionally also pass in a previously created tunnel.
///
/// Returns the created tunnel if no async loop is running, otherwise returns a Task to await with a tunnel result.
///
/// :param server: The server to link with a tunnel.
/// :type server: http.server.HTTPServer or None
/// :param tunnel: The NgrokTunnel to use to link with the server.
/// :type tunnel: NgrokTunnel or None
/// :return: The tunnel linked with the server, or a Task to await for said tunnel.
/// :rtype: NgrokTunnel or Task
#[pyfunction]
#[pyo3(text_signature = "(server=None, tunnel=None)")]
pub fn listen(
    py: Python,
    server: Option<Py<PyAny>>,
    tunnel: Option<Py<PyAny>>,
) -> PyResult<Py<PyAny>> {
    let mut forward = "".to_string();
    if let Some(server) = server {
        let address = address_from_server(py, server)?;
        forward = format!("input.forward('{address}')");
    }

    loop_wrap(
        py,
        tunnel,
        &format!(
            r###"
    if input is None:
        session = await NgrokSessionBuilder().authtoken_from_env().connect()
        input = await session.http_endpoint().listen()
    {forward}
    return input
    "###
        ),
    )
}

pub(crate) fn address_from_server(py: Python, server: Py<PyAny>) -> Result<String, PyErr> {
    let server_address_attr = server.getattr(py, "server_address")?;
    let address_type = server_address_attr.as_ref(py).get_type();
    if server_address_attr
        .as_ref(py)
        .is_instance(py.get_type::<PyTuple>())?
    {
        let address: &PyTuple = server_address_attr.downcast(py)?;
        Ok(format!(
            "{TCP_PREFIX}{}:{}",
            address.get_item(0)?,
            address.get_item(1)?
        ))
    } else if server_address_attr
        .as_ref(py)
        .is_instance(py.get_type::<PyString>())?
    {
        let address: &PyString = server_address_attr.downcast(py)?;
        Ok(format!("{UNIX_PREFIX}{address}"))
    } else {
        return Err(py_err(format!(
            "Unhandled server_address type: {address_type}"
        )));
    }
}

/// Set the WERKZEUG_SERVER_FD environment variable with a file descriptor from a default HTTP tunnel.
/// Also sets WERKZEUG_RUN_MAIN to "true" to engage the use of WERKZEUG_SERVER_FD.
///
/// Returns the created tunnel if no async loop is running, otherwise returns a Task to await with a tunnel result.
///
/// :param tunnel: The NgrokTunnel to use to link with the werkzeug server.
/// :type tunnel: NgrokTunnel or None
/// :return: The tunnel linked with the server, or a Task to await for said tunnel.
/// :rtype: NgrokTunnel or Task
#[pyfunction]
#[pyo3(text_signature = "(tunnel=None)")]
pub fn werkzeug_develop(py: Python, tunnel: Option<Py<PyAny>>) -> PyResult<Py<PyAny>> {
    loop_wrap(
        py,
        tunnel,
        r###"
    if input is None:
        session = await NgrokSessionBuilder().authtoken_from_env().connect()
        input = await session.http_endpoint().listen()

    import os
    os.environ["WERKZEUG_SERVER_FD"] = str(input.fd)
    os.environ["WERKZEUG_RUN_MAIN"] = "true"
    return input
    "###,
    )
}

/// Python wrapper to call the passed in work in an async context whether or not an async loop is running.
pub(crate) fn loop_wrap(py: Python, input: Option<Py<PyAny>>, work: &str) -> PyResult<Py<PyAny>> {
    let code = format!(
        r###"
import asyncio
import ngrok
from ngrok import NgrokSessionBuilder

async def wrap(input=None):
{work}

def run(input=None):
    try:
        running_loop = asyncio.get_running_loop()
        return running_loop.create_task(wrap(input))
    except RuntimeError:
        pass

    return asyncio.run(wrap(input))
    "###
    );

    call_code(py, input, code.as_str())
}

/// Call the given code, returning the required 'retval' attribute from it.
fn call_code(py: Python, input: Option<Py<PyAny>>, code: &str) -> PyResult<Py<PyAny>> {
    // give fake filename and module name to not interfere with other projects' empty-string module
    let run = PyModule::from_code(py, code, "ngrok_wrapper", "ngrok_wrapper")?.getattr("run")?;

    let res = match input {
        Some(input) => {
            let args = PyTuple::new(py, &[input]);
            run.call1(args)?
        }
        None => run.call0()?,
    };

    Ok(res.into())
}

/// Create and bind a python localhost TCP socket.
pub(crate) fn bound_default_tcp_socket(py: Python) -> PyResult<Py<PyAny>> {
    let socket = PyModule::import(py, intern!(py, "socket"))?;
    let sock_func = socket.getattr(intern!(py, "socket"))?;
    let obj = sock_func.call0()?;
    let bind = obj.getattr(intern!(py, "bind"))?;
    let host: &PyAny = PyString::new(py, "localhost");
    let port: &PyAny = 0u8.into_py(py).into_ref(py);
    let address = PyTuple::new(py, [host, port]);
    let args = PyTuple::new(py, [address]);
    bind.call1(args)?;
    let res = obj.into_py(py);
    Ok(res)
}

/// Create and bind a python unix socket.
pub(crate) fn bound_default_unix_socket(py: Python) -> PyResult<Py<PyAny>> {
    let socket = PyModule::import(py, intern!(py, "socket"))?;
    let sock_func = socket.getattr(intern!(py, "socket"))?;
    let af_unix = socket.getattr(intern!(py, "AF_UNIX"))?;
    let sock_args = PyTuple::new(py, [af_unix]);
    let obj = sock_func.call1(sock_args)?;
    let bind = obj.getattr(intern!(py, "bind"))?;
    let address = pipe_name(py)?;
    let args = PyTuple::new(py, &[address]);
    bind.call1(args)?;
    let res = obj.into_py(py);
    Ok(res)
}

pub fn wrap_object(py: Python, input: Py<PyAny>) -> PyResult<Py<PyAny>> {
    call_code(
        py,
        Some(input),
        r###"
class Proxy:
    def __init__(self, proxied):
        object.__setattr__(self, '_proxied', proxied)

    def __getattribute__(self, name):
        p = object.__getattribute__(self, '_proxied')
        return getattr(p, name)

    def __setattr__(self, name, value):
        p = object.__getattribute__(self, '_proxied')
        setattr(p, name, value)

    def __getitem__(self, key):
        p = object.__getattribute__(self, '_proxied')
        return p[key]

    def __setitem__(self, key, value):
        p = object.__getattribute__(self, '_proxied')
        p[key] = value

    def __delitem__(self, key):
        p = object.__getattribute__(self, '_proxied')
        del p[key]

def run(input):
    return Proxy(input)
"###,
    )
}
