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

[Website](https://ngrok.com)

**Note: This is alpha-quality software. Interfaces are subject to change without warning.**

ngrok is a globally distributed reverse proxy commonly used for quickly getting a public URL to a
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

# Quickstart

After you've installed the package, you'll need an Auth Token. Retrieve one on the
[Auth Token page of your ngrok dashboard](https://dashboard.ngrok.com/get-started/your-authtoken)

There are multiple examples in [the /examples directory](https://github.com/ngrok/ngrok-py/tree/main/examples).
A minimal use-case looks like the following:

```python
async def create_tunnel():
  builder = NgrokSessionBuilder()
  session = await builder.authtoken_from_env().connect()
  tunnel = await session.http_endpoint().metadata("python tun meta").listen()
  print("tunnel: {}".format(tunnel.url()))

  res = await tunnel.forward_tcp("localhost:9000")
```

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
