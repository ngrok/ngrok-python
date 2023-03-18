use log::LevelFilter;
use pyo3::{
    pyfunction,
    PyResult,
    Python,
};
use pyo3_log::{
    Caching,
    Logger,
};

use crate::py_err;

/// Set the log level for the bridge to Python logging.
/// The log level defaults to INFO, it can be set to one of ERROR, WARN, INFO, DEBUG, or TRACE.
#[pyfunction]
pub fn log_level(py: Python, level: Option<String>) -> PyResult<()> {
    let tracing_level = if let Some(level) = level {
        match level.to_uppercase().as_str() {
            "ERROR" => LevelFilter::Error,
            "WARN" => LevelFilter::Warn,
            "INFO" => LevelFilter::Info,
            "DEBUG" => LevelFilter::Debug,
            "TRACE" => LevelFilter::Trace,
            _ => return Err(py_err("Unknown log level: {level:?}")),
        }
    } else {
        LevelFilter::Info
    };

    if let Err(err) = Logger::new(py, Caching::LoggersAndLevels)?
        .filter(LevelFilter::Trace)
        .install()
    {
        if !err.to_string().contains("already initialized") {
            return Err(py_err(format!("Failed to subscribe logger, {err}")));
        }
    }
    log::set_max_level(tracing_level);
    Ok(())
}
