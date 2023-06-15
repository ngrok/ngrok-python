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
[mit-url]: https://github.com/ngrok/ngrok-rust/blob/main/LICENSE-MIT
[apache-badge]: https://img.shields.io/badge/license-Apache_2.0-blue.svg
[apache-url]: https://github.com/ngrok/ngrok-rust/blob/main/LICENSE-APACHE
[ci-badge]: https://github.com/ngrok/ngrok-python/actions/workflows/ci.yml/badge.svg
[ci-url]: https://github.com/ngrok/ngrok-python/actions/workflows/ci.yml

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

To verify that the library is correctly installed use the following code, which forwards to `localhost` port `80`:
```python
import ngrok
tunnel = ngrok.connect()
```

`ngrok-python` officially supports Python 3.7+.

# Documentation

A quickstart guide and a full API reference are included in the [ngrok-python API documentation](https://ngrok.github.io/ngrok-python/).

# Quickstart

After you've installed the package, you'll need an Auth Token. Retrieve one on the
[Auth Token page of your ngrok dashboard](https://dashboard.ngrok.com/get-started/your-authtoken)

Here is a minimal code block [using the 'connect' convenience function](https://github.com/ngrok/ngrok-python/blob/main/examples/ngrok-connect-minimal.py), with `authtoken_from_env=True` to use an Auth Token from the `NGROK_AUTHTOKEN` environment variable, and making a connection to `localhost` port `9000`:

```python
tunnel = ngrok.connect(9000, authtoken_from_env=True)
print (f"Ingress established at {tunnel.url()}")
```

There are multiple examples in [the /examples directory](https://github.com/ngrok/ngrok-python/tree/main/examples).

## Authorization

To use most of ngrok's features, you'll need an authtoken. To obtain one, sign up for free at [ngrok.com](https://dashboard.ngrok.com/signup) and retrieve it from the [authtoken page in your ngrok dashboard](https://dashboard.ngrok.com/get-started/your-authtoken). Once you have copied your authtoken, you can reference it in several ways.

You can set the authtoken in the environment variable `NGROK_AUTHTOKEN` and then pass `authtoken_from_env=True` to the [connect](https://ngrok.github.io/ngrok-python/module.html) function:

```python
ngrok.connect(authtoken_from_env=True, ...)
```

You can also set the default Auth Token to use for all connections by calling the [set_auth_token](https://ngrok.github.io/ngrok-python/module.html) function:

```python
ngrok.set_auth_token(token)
```

Or the Auth Token can be passed directly to the [connect](https://ngrok.github.io/ngrok-python/module.html) function:

```python
ngrok.connect(authtoken=token, ...)
```

## Connection

The [connect](https://ngrok.github.io/ngrok-python/module.html) function is the most common way to use this library. It will start an ngrok session if it is not already running, and then establish a tunnel to the specified address. If there is an asynchronous runtime running the [connect](https://ngrok.github.io/ngrok-python/module.html) function returns a promise that resolves to the public tunnel object.

With just an integer, the `connect` function will forward to `localhost` on the specified port, or the host can be specified as a string:

```python
tunnel = ngrok.connect(4242);
tunnel = ngrok.connect("localhost:4242");
```

More options can be passed to the `connect` function to customize the connection:

```python
tunnel = ngrok.connect(8080, basic_auth="ngrok:online1line"})
tunnel = ngrok.connect(8080, oauth_provider="google", oauth_allow_domains="example.com")
```

The second (optional) argument is the tunnel type, with defaults to `http`. To create a TCP tunnel:

```python
tunnel = ngrok.connect(25565, "tcp")
```

Since the options are kwargs, you can also use the `**` operator to pass a dictionary of configuration:

```python
options = {"authtoken_from_env":True, "response_header_add":"X-Awesome:yes"}
tunnel = ngrok.connect(8080, **options)
```

## Disconnection

To close a tunnel use the [disconnect](https://ngrok.github.io/ngrok-python/module.html) function with the `url` of the tunnel to close. If there is an asynchronous runtime running the [disconnect](https://ngrok.github.io/ngrok-python/module.html) function returns a promise that resolves when the call is complete.

```python
ngrok.disconnect(url)
```

Or omit the URL to close all tunnels:

```python
ngrok.disconnect()
```

The [close](https://ngrok.github.io/ngrok-python/ngrok_tunnel.html) method on a tunnel will shut it down, and also stop the ngrok session if it is no longer needed. This method returns a promise that resolves when the tunnel is closed.

```python
await tunnel.close()
```

## Listing Tunnels

To list all current non-closed tunnels use the [get_tunnels](https://ngrok.github.io/ngrok-python/module.html) function. If there is an asynchronous runtime running the [get_tunnels](https://ngrok.github.io/ngrok-python/module.html) function returns a promise that resolves to the list of tunnel objects.

```python
tunnels = ngrok.get_tunnels()
```

# Full Configuration

This example shows [all the possible configuration items ngrok.connect](https://github.com/ngrok/ngrok-python/blob/main/examples/ngrok-connect-full.py):

```python
tunnel = ngrok.connect(
    # session configuration
    addr="localhost:8080",
    authtoken="<authtoken>",
    authtoken_from_env=True,
    session_metadata="Online in One Line",
    # tunnel configuration
    basic_auth=["ngrok:online1line"],
    circuit_breaker=0.1,
    compression=True,
    domain="<domain>",
    ip_restriction_allow_cidrs="0.0.0.0/0",
    ip_restriction_deny_cidrs="10.1.1.1/32",
    metadata="example tunnel metadata from python",
    mutual_tls_cas=load_file("ca.crt"),
    oauth_provider="google",
    oauth_allow_domains=["<domain>"],
    oauth_allow_emails=["<email>"],
    oauth_scopes=["<scope>"],
    oidc_issuer_url="<url>",
    oidc_client_id="<id>",
    oidc_client_secret="<secret>",
    oidc_allow_domains=["<domain>"],
    oidc_allow_emails=["<email>"],
    oidc_scopes=["<scope>"],
    proxy_proto="",  # One of: "", "1", "2"
    request_header_remove="X-Req-Nope",
    response_header_remove="X-Res-Nope",
    request_header_add="X-Req-Yup:true",
    response_header_add="X-Res-Yup:true",
    schemes=["HTTPS"],
    verify_webhook_provider="twilio",
    verify_webhook_secret="asdf",
    websocket_tcp_converter=True,
)
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
* [Aiohttp](https://docs.aiohttp.org) - [Example](https://github.com/ngrok/ngrok-python/tree/main/examples/aiohttp-ngrok.py)
* [AWS App Runner](https://aws.amazon.com/apprunner/) - See the [ngrok SDK Serverless Example](https://github.com/ngrok/ngrok-sdk-serverless-example) repository, making the necessary [changes to use Python](https://docs.aws.amazon.com/apprunner/latest/dg/service-source-code-python.html) instead of NodeJS
* [Django](https://www.djangoproject.com/) - [Single File Example](https://github.com/ngrok/ngrok-python/tree/main/examples/django-single-file.py), [Modify manage.py Example](https://github.com/ngrok/ngrok-python/tree/main/examples/djangosite/manage.py), [Modify asgi.py Example](https://github.com/ngrok/ngrok-python/tree/main/examples/djangosite/djangosite/ngrok-asgi.py), or use the `ngrok-asgi` ASGI Runner discussed above
* [Flask](https://flask.palletsprojects.com) - [Example](https://github.com/ngrok/ngrok-python/tree/main/examples/flask-ngrok.py)
* [Gunicorn](https://gunicorn.org/) - Use the `ngrok-asgi` ASGI Runner discussed above
* [Streamlit](https://streamlit.io) - [Example](https://github.com/ngrok/ngrok-python/tree/main/examples/streamlit/streamlit-ngrok.py)
* [Tornado](https://www.tornadoweb.org) - [Example](https://github.com/ngrok/ngrok-python/tree/main/examples/tornado-ngrok.py)
* [Uvicorn](https://www.uvicorn.org/) - [Example](https://github.com/ngrok/ngrok-python/tree/main/examples/uvicorn-ngrok.py), or use the `ngrok-asgi` ASGI Runner discussed above

## Machine Learning
* [Gradio](https://gradio.app/) - [ngrok-asgi Example](https://github.com/ngrok/ngrok-python/tree/main/examples/gradio/gradio-asgi.py), [gradio CLI Example](https://github.com/ngrok/ngrok-python/tree/main/examples/gradio/gradio-ngrok.py) sharing machine learning apps
* [OpenPlayground](https://github.com/nat/openplayground) - [Example](https://github.com/ngrok/ngrok-python/tree/main/examples/openplayground/run.py) of an LLM playground you can run on your laptop
* [GPT4ALL](https://github.com/nomic-ai/gpt4all) - [Example](https://github.com/ngrok/ngrok-python/tree/main/examples/gpt4all/run.py) of running the [GPT4All-L Snoozy 13B](https://gpt4all.io/index.html) model with a Gradio frontend

## Tunnel Types
* HTTP - [Minimal Example](https://github.com/ngrok/ngrok-python/tree/main/examples/ngrok-http-minimal.py), [Full Configuration Example](https://github.com/ngrok/ngrok-python/tree/main/examples/ngrok-http-full.py)
* Labeled - [Example](https://github.com/ngrok/ngrok-python/tree/main/examples/ngrok-labeled.py)
* TCP - [Example](https://github.com/ngrok/ngrok-python/tree/main/examples/ngrok-tcp.py)
* TLS - [Example](https://github.com/ngrok/ngrok-python/tree/main/examples/ngrok-tls.py)

# Builders

For more control over Sessions and Tunnels, the builder classes can be used.

A minimal builder use-case looks like [the following](https://github.com/ngrok/ngrok-python/blob/main/examples/ngrok-http-minimal.py):

```python
async def create_tunnel():
    session = await ngrok.NgrokSessionBuilder().authtoken_from_env().connect()
    tunnel = await session.http_endpoint().listen()
    print (f"Ingress established at {tunnel.url()}")
    tunnel.forward_tcp("localhost:9000")
```

See here for a [Full Configuration Example](https://github.com/ngrok/ngrok-python/blob/main/examples/ngrok-http-full.py)

# Platform Support

Pre-built binaries are provided on PyPI for the following platforms:

| OS         | i686 | x64 | aarch64 | arm |
| ---------- | -----|-----|---------|-----|
| Windows    |   ✓  |  ✓  |    *    |     |
| MacOS      |      |  ✓  |    ✓    |     |
| Linux      |      |  ✓  |    ✓    |  ✓  |
| Linux musl |      |  ✓  |    ✓    |     |
| FreeBSD    |      |  *  |         |     |

ngrok-python, and [ngrok-rust](https://github.com/ngrok/ngrok-rust/) which it depends on, are open source, so it may be possible to build them for other platforms.

* Windows-aarch64 will be supported after the next release of [Ring](https://github.com/briansmith/ring/issues/1167).
* FreeBSD-x64 is built by the release process, but PyPI won't accept BSD flavors.

# Dependencies

This project relies on [PyO3](https://pyo3.rs/), an excellent system to ease development and building of Rust plugins for Python.

Thank you to [OpenIoTHub](https://github.com/OpenIoTHub/ngrok) for handing over the ngrok name on PyPI.

# Change Log

Changes are tracked in [CHANGELOG.md](https://github.com/ngrok/ngrok-python/blob/main/CHANGELOG.md).

# License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in ngrok-python by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
