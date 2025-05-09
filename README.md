# Python SDK for ngrok

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

`ngrok-python` is the official Python SDK for ngrok that requires no binaries. Quickly enable secure production-ready connectivity to your applications and services directly from your code.

[ngrok](https://ngrok.com) is a globally distributed gateway that provides secure connectivity for applications and services running in any environment.

# Installation

The `ngrok-python` SDK can be installed from [PyPI](https://pypi.org/project/ngrok) via `pip`:

```shell
pip install ngrok
```

# Quickstart

1. [Install `ngrok-python`](#installation)
2. Export your [authtoken from the ngrok dashboard](https://dashboard.ngrok.com/get-started/your-authtoken) as `NGROK_AUTHTOKEN` in your terminal
3. Add the following code to your application to establish connectivity via the [forward method](https://github.com/ngrok/ngrok-python/blob/main/examples/ngrok-forward-minimal.py) through port `9000` on `localhost`:

    ```python
    # import ngrok python sdk
    import ngrok
    import time
    
    # Establish connectivity
    listener = ngrok.forward(9000, authtoken_from_env=True)
    
    # Output ngrok url to console
    print(f"Ingress established at {listener.url()}")

    # Keep the listener alive
    try:
    while True:
        time.sleep(1)
    except KeyboardInterrupt:
        print("Closing listener")
    ```

That's it! Your application should now be available through the url output in your terminal. 

> **Note**
> You can find more examples in [the examples directory](https://github.com/ngrok/ngrok-python/tree/main/examples).

# Documentation

A full quickstart guide and API reference can be found in the [ngrok-python documentation](https://ngrok.github.io/ngrok-python/).

### Authentication

To use most of ngrok's features, you'll need an authtoken. To obtain one, sign up for free at [ngrok.com](https://dashboard.ngrok.com/signup) and retrieve it from the [authtoken page in your ngrok dashboard](https://dashboard.ngrok.com/get-started/your-authtoken). Once you have copied your authtoken, you can reference it in several ways.

You can set it in the `NGROK_AUTHTOKEN` environment variable and pass `authtoken_from_env=True` to the [forward](https://ngrok.github.io/ngrok-python/module.html) method:

```python
ngrok.forward(authtoken_from_env=True, ...)
```

Or pass the authtoken directly to the [forward](https://ngrok.github.io/ngrok-python/module.html) method:

```python
ngrok.forward(authtoken=token, ...)
```

Or set it for all connections with the [set_auth_token](https://ngrok.github.io/ngrok-python/module.html) method:

```python
ngrok.set_auth_token(token)
```

### Connection

The [forward](https://ngrok.github.io/ngrok-python/module.html) method is the easiest way to start an ngrok session and establish a listener to a specified address. If an asynchronous runtime is running, the [forward](https://ngrok.github.io/ngrok-python/module.html) method returns a promise that resolves to the public listener object.

With no arguments, the [forward](https://ngrok.github.io/ngrok-python/module.html) method will start an HTTP listener to `localhost` port `80`:

```python
listener = ngrok.forward()
```

You can pass the port number to forward on `localhost`:

```python
listener = ngrok.forward(4242)
```

Or you can specify the host and port via a string:

```python
listener = ngrok.forward("localhost:4242")
```

More options can be passed to the `forward` method to customize the connection:

```python
listener = ngrok.forward(8080, basic_auth="ngrok:online1line"})
listener = ngrok.forward(8080, oauth_provider="google", oauth_allow_domains="example.com")
```

The second (optional) argument is the listener type, which defaults to `http`. To create a TCP listener:

```python
listener = ngrok.forward(25565, "tcp")
```

Since the options are kwargs, you can also use the `**` operator to pass a dictionary for configuration:

```python
options = {"authtoken_from_env":True, "response_header_add":"X-Awesome:yes"}
listener = ngrok.forward(8080, **options)
```

See [Full Configuration](#full-configuration) for the list of possible configuration options.

### Disconnection

To close a listener use the [disconnect](https://ngrok.github.io/ngrok-python/module.html) method with the `url` of the listener to close. If there is an asynchronous runtime running the [disconnect](https://ngrok.github.io/ngrok-python/module.html) method returns a promise that resolves when the call is complete.

```python
ngrok.disconnect(url)
```

Or omit the `url` to close all listeners:

```python
ngrok.disconnect()
```

The [close](https://ngrok.github.io/ngrok-python/ngrok_listener.html) method on a listener will shut it down, and also stop the ngrok session if it is no longer needed. This method returns a promise that resolves when the listener is closed.

```python
await listener.close()
```

### List all Listeners

To list all current non-closed listeners use the [get_listeners](https://ngrok.github.io/ngrok-python/module.html) method. If there is an asynchronous runtime running the [get_listeners](https://ngrok.github.io/ngrok-python/module.html) method returns a promise that resolves to the list of listener objects.

```python
listeners = ngrok.get_listeners()
```

### TLS Backends

As of version `0.10.0` there is backend TLS connection support, validated by a filepath specified in the `SSL_CERT_FILE` environment variable, or falling back to the host OS installed trusted certificate authorities. So it is now possible to do this to connect:

```python
ngrok.forward("https://127.0.0.1:3000", authtoken_from_env=True)
```

If the service is using certs not trusted by the OS, such as self-signed certificates, add an environment variable like this before running: `SSL_CERT_FILE=/path/to/ca.crt`. There is also a `verify_upstream_tls=False` option to disable certification verification.

### Unix Sockets

You may also choose to use Unix Sockets instead of TCP. You can view an example of this [here](https://github.com/ngrok/ngrok-python/blob/main/examples/ngrok-http-full.py).

A socket address may be passed directly into the listener `forward()` call as well by prefixing the address with `unix:`, for example `unix:/tmp/socket-123`.

### Builders

For more control over Sessions and Listeners, the builder classes can be used.

A minimal example using the builder class looks like [the following](https://github.com/ngrok/ngrok-python/blob/main/examples/ngrok-http-minimal.py):

```python
async def create_listener():
    session = await ngrok.NgrokSessionBuilder().authtoken_from_env().connect()
    listener = await session.http_endpoint().listen()
    print (f"Ingress established at {listener.url()}")
    listener.forward("localhost:9000")
```

See here for a [Full Configuration Example](https://github.com/ngrok/ngrok-python/blob/main/examples/ngrok-http-full.py)

### Full Configuration

This example shows [all the possible configuration items of ngrok.forward](https://github.com/ngrok/ngrok-python/blob/main/examples/ngrok-forward-full.py):

```python
listener = ngrok.forward(
    # session configuration
    addr="localhost:8080",
    authtoken="<authtoken>",
    authtoken_from_env=True,
    app_protocol="http2",
    session_metadata="Online in One Line",
    # advanced session connection configuration
    server_addr="example.com:443",
    root_cas="trusted",
    session_ca_cert=load_file("ca.pem"),
    # listener configuration
    metadata="example listener metadata from python",
    domain="<domain>",
    schemes=["HTTPS"],
    proto="http",
    proxy_proto="",  # One of: "", "1", "2"
    labels="edge:edghts_2G...",  # Along with proto="labeled"
    # module configuration
    basic_auth=["ngrok:online1line"],
    circuit_breaker=0.1,
    compression=True,
    allow_user_agent="^mozilla.*",
    deny_user_agent="^curl.*",
    allow_cidr="0.0.0.0/0",
    deny_cidr="10.1.1.1/32",
    crt=load_file("crt.pem"),
    key=load_file("key.pem"),
    mutual_tls_cas=load_file("ca.crt"),
    oauth_provider="google",
    oauth_allow_domains=["<domain>"],
    oauth_allow_emails=["<email>"],
    oauth_scopes=["<scope>"],
    oauth_client_id="<id>",
    oauth_client_secret="<id>",
    oidc_issuer_url="<url>",
    oidc_client_id="<id>",
    oidc_client_secret="<secret>",
    oidc_allow_domains=["<domain>"],
    oidc_allow_emails=["<email>"],
    oidc_scopes=["<scope>"],
    policy="<policy_json>",
    request_header_remove="X-Req-Nope",
    response_header_remove="X-Res-Nope",
    request_header_add="X-Req-Yup:true",
    response_header_add="X-Res-Yup:true",
    verify_upstream_tls=False,
    verify_webhook_provider="twilio",
    verify_webhook_secret="asdf",
    websocket_tcp_converter=True,
)
```

# ASGI Runner

`ngrok-python` comes bundled with an ASGI (Asynchronous Server Gateway Interface) runner `ngrok-asgi` that can be used for Uvicorn, Gunicorn, Django and more, with no code. 

To use prefix your start up command for a Uvicorn or Gunicorn web server with either `ngrok-asgi` or `python -m ngrok`. 

Any TCP or Unix Domain Socket arguments will be used to establish connectivity automatically. The ngrok listener can be configured using command flags, for instance adding `--basic-auth ngrok online1line` will introduce basic authentication to the ingress listener.

### Uvicorn

```shell
# Basic Usage
ngrok-asgi uvicorn mysite.asgi:application

# With custom host and port
ngrok-asgi uvicorn mysite.asgi:application \
    --host localhost \
    --port 1234

# Using basic auth
ngrok-asgi uvicorn mysite.asgi:application \
    --host localhost \
    --port 1234 \
    --basic-auth ngrok online1line

# Using custom sock file
ngrok-asgi uvicorn mysite.asgi:application \
    --uds /tmp/uvicorn.sock

# Using module name
python -m ngrok uvicorn mysite.asgi:application \
    --oauth-provider google \
    --allow-emails bob@example.com
```

### Gunicorn

```shell
# Basic Usage
ngrok-asgi gunicorn mysite.asgi:application -k uvicorn.workers.UvicornWorker

# With custom host and port
ngrok-asgi gunicorn mysite.asgi:application -k uvicorn.workers.UvicornWorker \
    --bind localhost:1234

# Using webhook verifications
ngrok-asgi gunicorn mysite.asgi:application -k uvicorn.workers.UvicornWorker \
    --webhook-verification twilio s3cr3t

# Using custom sock file
ngrok-asgi gunicorn mysite.asgi:application -k uvicorn.workers.UvicornWorker \
    --bind unix:/tmp/gunicorn.sock

# Using module name
python -m ngrok gunicorn mysite.asgi:application -k uvicorn.workers.UvicornWorker --response-header X-Awesome True
```

# Examples

#### Listeners
  - [HTTP](https://github.com/ngrok/ngrok-python/tree/main/examples/ngrok-http-minimal.py)
    - [Full Configuration Example](https://github.com/ngrok/ngrok-python/tree/main/examples/ngrok-http-full.py)
  - [Labeled](https://github.com/ngrok/ngrok-python/tree/main/examples/ngrok-labeled.py)
  - [TCP](https://github.com/ngrok/ngrok-python/tree/main/examples/ngrok-tcp.py)
  - [TLS](https://github.com/ngrok/ngrok-python/tree/main/examples/ngrok-tls.py)

#### Frameworks
  - [AIOHTTP](https://github.com/ngrok/ngrok-python/tree/main/examples/aiohttp-ngrok.py)
  - [AWS APP Runner](https://github.com/ngrok/ngrok-sdk-serverless-example)
    - with [changes for Python](https://docs.aws.amazon.com/apprunner/latest/dg/service-source-code-python.html)
  - Django
    - [Single File Example](https://github.com/ngrok/ngrok-python/tree/main/examples/django-single-file.py)
    - [Modify manage.py Example](https://github.com/ngrok/ngrok-python/tree/main/examples/djangosite/manage.py)
    - [Modify asgi.py Example](https://github.com/ngrok/ngrok-python/tree/main/examples/djangosite/djangosite/ngrok-asgi.py)
    - or [via `ngrok-asgi`](#asgi-runner)
  - [Flask](https://github.com/ngrok/ngrok-python/tree/main/examples/flask-ngrok.py)
  - [Gunicorn](#gunicorn)
  - [Hypercorn](https://github.com/ngrok/ngrok-python/tree/main/examples/hypercorn-http2-ngrok.py)
  - [Streamlit](https://github.com/ngrok/ngrok-python/tree/main/examples/streamlit/streamlit-ngrok.py)
  - [Tornado](https://github.com/ngrok/ngrok-python/tree/main/examples/tornado-ngrok.py)
  - [Uvicorn](https://github.com/ngrok/ngrok-python/tree/main/examples/uvicorn-ngrok.py)

#### Machine Learning
  - Gradio
    - [ngrok-asgi Example](https://github.com/ngrok/ngrok-python/tree/main/examples/gradio/gradio-asgi.py)
    - [gradio CLI Example](https://github.com/ngrok/ngrok-python/tree/main/examples/gradio/gradio-ngrok.py)
  - [OpenPlayground](https://github.com/ngrok/ngrok-python/tree/main/examples/openplayground/run.py)
  - [GPT4ALL](https://github.com/ngrok/ngrok-python/tree/main/examples/gpt4all/run.py)
  - [Stable Diffusion WebUI](https://github.com/AUTOMATIC1111/stable-diffusion-webui/) by AUTOMATIC1111
    - `ngrok-python` is now built-in, see the `--ngrok` and `--ngrok-options` arguments.
  - [Text Generation WebUI](https://github.com/oobabooga/text-generation-webui) by oobabooga
    - `ngrok-python` is now built-in, see the `--extension ngrok` argument.

# Platform Support

Pre-built binaries are provided on PyPI for the following platforms:

| OS         | i686 | x64 | aarch64 | arm |
| ---------- | -----|-----|---------|-----|
| Windows    |   ✓  |  ✓  |    ✓    |     |
| MacOS      |      |  ✓  |    ✓    |     |
| Linux      |      |  ✓  |    ✓    |  ✓  |
| Linux musl |      |  ✓  |    ✓    |     |
| FreeBSD    |      |  *  |         |     |

> **Note**
> `ngrok-python`, and [ngrok-rust](https://github.com/ngrok/ngrok-rust/) which it depends on, are open source, so it may be possible to build them for other platforms.
> - FreeBSD-x64 is built by the release process, but PyPI won't accept BSD flavors.

# Dependencies

- This project relies on [PyO3](https://pyo3.rs/), an excellent system to ease development and building of Rust plugins for Python.
- Thank you to [OpenIoTHub](https://github.com/OpenIoTHub/ngrok) for handing over the ngrok name on PyPI.

# Changelog

Changes to `ngrok-python` are tracked under [CHANGELOG.md](https://github.com/ngrok/ngrok-python/blob/main/CHANGELOG.md).

# Join the ngrok Community

- Check out [our official docs](https://docs.ngrok.com)
- Read about updates on [our blog](https://blog.ngrok.com)
- Open an [issue](https://github.com/ngrok/ngrok-python/issues) or [pull request](https://github.com/ngrok/ngrok-python/pulls)
- Join our [Slack community](https://ngrok.com/slack)
- Follow us on [X/Twitter (@ngrokHQ)](https://twitter.com/ngrokhq)
- Subscribe to our [YouTube channel (@ngrokHQ)](https://www.youtube.com/@ngrokhq)

# License

This project is dual-licensed under [Apache, Version 2.0](LICENSE-APACHE) and [MIT](LICENSE-MIT).
You can choose between one of them if you use this work.

### Contributions

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in `ngrok-python` by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

# Development: Getting Started

**Prerequisites:**
- a valid Ngrok `authtoken`
-  `make` available in your PATH

1. Update [Cargo.toml](./Cargo.toml) with the _latest supported_ ```ngrok = { version = "=VERSION_HERE" }``` from [ngrok-rust](https://github.com/ngrok/ngrok-rust/blob/main/ngrok/Cargo.toml#L3). `ngrok-rust` is used for the bindings in [src/rust_files_here.rs](./src)

2. Run `make build` (builds the `rust` bindings / `python` dependencies)

3. Happy developing!

<br/>

**Example Commands**:

_building the project_
```shell
make develop
```

_running the entire test suite_
```shell
# running the entire test suite
export NGROK_AUTHTOKEN="YOUR_AUTHTOKEN_HERE"; make test
```

_running an individual test_
```shell
# running an individual test
export NGROK_AUTHTOKEN="YOUR_AUTHTOKEN_HERE"; make test="-k TEST_CLASS.NAME_OF_TEST" test
```

[See the MakeFile for more examples](./Makefile)

### HTTP2 

The examples include a minimal `hypercorn` HTTP/2 example if you run `make http2`. You can curl the endpoint logged with `INFO:ngrok.listener:Created` and verify the HTTP/2 response from `hypercorn`.

```bash
curl --http2 -v https://<YOUR_LISTENER_URL>
*   Trying <YOUR_IP>:443...
* Connected to a6278d6c07ce.ngrok.app (<YOUR_IP>) port 443 (#0)
* ALPN, offering h2
* ALPN, offering http/1.1
...
> GET / HTTP/2
> Host: a6278d6c07ce.ngrok.app
> user-agent: curl/7.81.0
> accept: */*
>
...
< HTTP/2 200
< content-type: text/plain
< date: Fri, 01 Mar 2024 18:50:23 GMT
< ngrok-agent-ips: <YOUR_AGENT_IP>
< ngrok-trace-id: ed038ace04876818149cf0769bd43e38
< server: hypercorn-h2
<
* TLSv1.2 (IN), TLS header, Supplemental data (23):
* TLSv1.2 (IN), TLS header, Supplemental data (23):
* Connection #0 to host <YOUR_LISTENER_URL> left intact
hello
```
