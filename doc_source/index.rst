.. ngrok documentation master file, created by
   sphinx-quickstart on Fri Mar 10 17:11:45 2023.
   You can adapt this file completely to your liking, but it should at least
   contain the root `toctree` directive.

The ngrok Agent SDK for Python
=================================

**Note: This is alpha-quality software. Interfaces are subject to change without warning.**

ngrok is a globally distributed reverse proxy commonly used for quickly getting a public URL to a
service running inside a private network, such as on your local laptop. The ngrok agent is usually
deployed inside a private network and is used to communicate with the ngrok cloud service.

This is the ngrok agent in library form, suitable for integrating directly into Python
applications. This allows you to quickly build ngrok into your application with no separate process
to manage.

Getting Started
===============

Installation
------------

The published library is available on `PyPI <https://pypi.org/project/ngrok>`_.

::

    pip install ngrok

ngrok-py officially supports Python 3.7+.

Quickstart
------------

After you've installed the package, you'll need an Auth Token. Retrieve one on the
`Auth Token page of your ngrok dashboard <https://dashboard.ngrok.com/get-started/your-authtoken>`_

There are multiple examples in `the /examples directory <https://github.com/ngrok/ngrok-py/tree/main/examples>`_.
A minimal use-case looks like the following:

::

  async def create_tunnel():
    builder = NgrokSessionBuilder()
    session = await builder.authtoken_from_env().connect()
    tunnel = await session.http_endpoint().metadata("python tun meta").listen()
    print("tunnel: {}".format(tunnel.url()))

    res = await tunnel.forward_tcp("localhost:9000")

Platform Support
-----------------

Pre-built binaries are provided on PyPI for the following platforms:

::

| OS         | i686 | x64 | aarch64 | arm |
| ---------- | -----|-----|---------|-----|
| Windows    |   ✓  |  ✓  |    *    |     |
| MacOS      |      |  ✓  |    ✓    |     |
| Linux      |      |  ✓  |    ✓    |  ✓  |
| Linux musl |      |  ✓  |    ✓    |     |
| FreeBSD    |      |  *  |         |     |

ngrok-py, and `ngrok-rs <https://github.com/ngrok/ngrok-rs/>`_ which it depends on, are open source, so it may be possible to build them for other platforms.

* Windows-aarch64 will be supported after the next release of `Ring <https://github.com/briansmith/ring/issues/1167>`_.
* FreeBSD-x64 is built by the release process, but PyPI won't accept BSD flavors.

Dependencies
------------

This project relies on `PyO3 <https://pyo3.rs/>`_, an excellent system to ease development and building of Rust plugins for Python.

Thank you to `OpenIoTHub <https://github.com/OpenIoTHub/ngrok>`_ for handing over the ngrok name on PyPI.

License
------------

This project is licensed under either of

 * Apache License, Version 2.0, (`LICENSE-APACHE <http://www.apache.org/licenses/LICENSE-2.0>`_)
 * MIT license (`LICENSE-MIT <http://opensource.org/licenses/MIT>`_)

at your option.

Contribution
------------

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in tokio-core by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.


API Reference
=============

.. toctree::
   :caption: Sessions:
   :maxdepth: 1

   ngrok_session_builder
   ngrok_session

.. toctree::
   :caption: Tunnel Builders:
   :maxdepth: 1

   ngrok_http_tunnel_builder
   ngrok_tcp_tunnel_builder
   ngrok_tls_tunnel_builder
   ngrok_labeled_tunnel_builder

.. toctree::
   :caption: Tunnels:
   :maxdepth: 1

   ngrok_http_tunnel
   ngrok_tcp_tunnel
   ngrok_tls_tunnel
   ngrok_labeled_tunnel

.. toctree::
   :caption: Module:
   :maxdepth: 1

   module

Indices and tables
==================

* :ref:`genindex`
* :ref:`modindex`
* :ref:`search`
