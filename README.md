# The ngrok Agent SDK for Python

[![PyPI][pypi-badge]][pypi-url]
[![Supported Versions][ver-badge]][ver-url]
[![MIT licensed][mit-badge]][mit-url]
[![Apache-2.0 licensed][apache-badge]][apache-url]
[![Continuous integration][ci-badge]][ci-url]

[pypi-badge]: https://img.shields.io/pypi/v/ngrok
[pypi-url]: https://pypi.org/project/ngrok
[ver-badge]: https://img.shields.io/pypi/pyversions/ngrok.svg
[ver-url]: https://pypi.org/project/ngrok
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/ngrok/ngrok-rs/blob/main/LICENSE-MIT
[apache-badge]: https://img.shields.io/badge/license-Apache_2.0-blue.svg
[apache-url]: https://github.com/ngrok/ngrok-rs/blob/main/LICENSE-APACHE
[ci-badge]: https://github.com/ngrok/ngrok-py/actions/workflows/ci.yml/badge.svg
[ci-url]: https://github.com/ngrok/ngrok-py/actions/workflows/ci.yml

**Note: This is beta-quality software. Interfaces may change without warning.**

[ngrok](https://ngrok.com) is a globally distributed reverse proxy commonly used for quickly getting a public URL to a
service running inside a private network, such as on your local laptop. The ngrok agent is usually
deployed inside a private network and is used to communicate with the ngrok cloud service.

This is the ngrok agent in library form, suitable for integrating directly into Python
applications. This allows you to quickly build ngrok into your application with no separate process
to manage.

If you're looking for the previous agent downloader project, it's over [here](https://github.com/OpenIoTHub/ngrok).

# Installation

The published library is available on
[PyPI](https://pypi.org/project/ngrok).

```shell
python -m pip install ngrok
```

ngrok-py officially supports Python 3.7+.

# Documentation

A quickstart guide and a full API reference are included in the [ngrok-py Python API documentation](https://ngrok.github.io/ngrok-py/).

# Quickstart

After you've installed the package, you'll need an Auth Token. Retrieve one on the
[Auth Token page of your ngrok dashboard](https://dashboard.ngrok.com/get-started/your-authtoken)

There are multiple examples in [the /examples directory](https://github.com/ngrok/ngrok-py/tree/main/examples).
A minimal use-case looks like the following:

```python
async def create_tunnel():
    session = await ngrok.NgrokSessionBuilder().authtoken_from_env().connect()
    tunnel = await session.http_endpoint().listen()
    print (f"Ingress established at {tunnel.url()}")
    tunnel.forward_tcp("localhost:9000")
```

# ASGI Runner - Tunnels to Uvicorn, Gunicorn, Django and More, With No Code

Prefix the command line which starts up a Uvicorn or Gunicorn web server with either `ngrok-asgi` or `python -m ngrok`. Any TCP or Unix Domain Socket arguments will be used to establish connectivity automatically. There are many command line arguments to configure the Tunnel used, for instance adding `--basic-auth ngrok online1line` will introduce basic authentication to the ingress tunnel.

## Uvicorn
Examples:
```shell
ngrok-asgi uvicorn mysite.asgi:application
ngrok-asgi uvicorn mysite.asgi:application --host localhost --port 1234
ngrok-asgi uvicorn mysite.asgi:application --host localhost --port 1234 --basic-auth ngrok online1line
ngrok-asgi uvicorn mysite.asgi:application --uds /tmp/uvicorn.sock

# Can use the module name as well, such as:
python -m ngrok uvicorn mysite.asgi:application --oauth-provider google --allow-emails bob@example.com
```

## Gunicorn
Examples:
```shell
ngrok-asgi gunicorn mysite.asgi:application -k uvicorn.workers.UvicornWorker
ngrok-asgi gunicorn mysite.asgi:application -k uvicorn.workers.UvicornWorker --webhook-verification twilio s3cr3t
ngrok-asgi gunicorn mysite.asgi:application -k uvicorn.workers.UvicornWorker --bind localhost:1234
ngrok-asgi gunicorn mysite.asgi:application -k uvicorn.workers.UvicornWorker --bind unix:/tmp/gunicorn.sock

# Can use the module name as well, such as:
python -m ngrok gunicorn mysite.asgi:application -k uvicorn.workers.UvicornWorker --response-header X-Awesome True
```

# Examples

## Frameworks
* [Aiohttp](https://docs.aiohttp.org) - [Example](https://github.com/ngrok/ngrok-py/tree/main/examples/aiohttp-ngrok.py)
* [Django](https://www.djangoproject.com/) - [Single File Example](https://github.com/ngrok/ngrok-py/tree/main/examples/django-single-file.py), [Modify manage.py Example](https://github.com/ngrok/ngrok-py/tree/main/examples/djangosite/manage.py), [Modify asgi.py Example](https://github.com/ngrok/ngrok-py/tree/main/examples/djangosite/djangosite/ngrok-asgi.py), or use the `ngrok-asgi` ASGI Runner discussed above
* [Flask](https://flask.palletsprojects.com) - [Example](https://github.com/ngrok/ngrok-py/tree/main/examples/flask-ngrok.py)
* [Gunicorn](https://gunicorn.org/) - Use the `ngrok-asgi` ASGI Runner discussed above
* [Uvicorn](https://www.uvicorn.org/) - [Example](https://github.com/ngrok/ngrok-py/tree/main/examples/uvicorn-ngrok.py), or use the `ngrok-asgi` ASGI Runner discussed above

## Tunnel Types
* HTTP - [Minimal Example](https://github.com/ngrok/ngrok-py/tree/main/examples/ngrok-http-minimal.py), [Full Configuration Example](https://github.com/ngrok/ngrok-py/tree/main/examples/ngrok-http-full.py)
* Labeled - [Example](https://github.com/ngrok/ngrok-py/tree/main/examples/ngrok-labeled.py)
* TCP - [Example](https://github.com/ngrok/ngrok-py/tree/main/examples/ngrok-tcp.py)
* TLS - [Example](https://github.com/ngrok/ngrok-py/tree/main/examples/ngrok-tls.py)

# Platform Support

Pre-built binaries are provided on PyPI for the following platforms:

| OS         | i686 | x64 | aarch64 | arm |
| ---------- | -----|-----|---------|-----|
| Windows    |   ✓  |  ✓  |    *    |     |
| MacOS      |      |  ✓  |    ✓    |     |
| Linux      |      |  ✓  |    ✓    |  ✓  |
| Linux musl |      |  ✓  |    ✓    |     |
| FreeBSD    |      |  *  |         |     |

ngrok-py, and [ngrok-rs](https://github.com/ngrok/ngrok-rs/) which it depends on, are open source, so it may be possible to build them for other platforms.

* Windows-aarch64 will be supported after the next release of [Ring](https://github.com/briansmith/ring/issues/1167).
* FreeBSD-x64 is built by the release process, but PyPI won't accept BSD flavors.

# Dependencies

This project relies on [PyO3](https://pyo3.rs/), an excellent system to ease development and building of Rust plugins for Python.

Thank you to [OpenIoTHub](https://github.com/OpenIoTHub/ngrok) for handing over the ngrok name on PyPI.

# License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in tokio-core by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
