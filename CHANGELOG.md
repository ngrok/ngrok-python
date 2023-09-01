## 0.9.0

- Flattened `tunnel.forward_pipe()` and `tunnel.forward_tcp()` into `tunnel.forward()`. Determination will be made based on `addr` input.
- Added [Mypy](https://mypy.readthedocs.io/en/stable/) static typing information.
- Added `session.get_tunnels()` to get a list of current non-closed tunnels for the session.
- Added `ngrok.set_auth_token(<token>)` to set a default auth token to use for sessions.
- Added [ngrok error codes](https://ngrok.com/docs/errors/reference/) as 3rd argument to thrown Errors, when available.

## 0.8.1:

- Cleanly return from a tunnel forward call after a `session.close()`

## 0.8.0:

- Add `NgrokSession.client_info()`
- Rename to `ngrok-python`

## 0.7.0:

- Add `ngrok.disconnect(url)` and `ngrok.kill()` convenience functions.
- Add examples for [Tornado](https://www.tornadoweb.org), [OpenPlayground](https://github.com/nat/openplayground), [Streamlit](https://streamlit.io/), [GPT4All](https://github.com/nomic-ai/gpt4all).
- Handle protocols in `addr`, and dots in `connect` option keys, for ease-of-use.
- [Docs](https://ngrok.github.io/ngrok-python/) improvements

## 0.6.0:

- Add `ngrok.connect(options)` convenience function.
- Add [Gradio](https://gradio.app/) example.
- Consolidate multiple tunnel classes into one.

## 0.5.0:

- Add ngrok-asgi ASGI runner.
- Migrate `ca_cert` to the upstream call in `ngrok-rust`.

## 0.4.0:

- Django, auto-format, and windows support in examples.

## 0.3.0:

- Add wrapper convenience functions.
- Add `close`, `ca_cert`, `handle_disconnection`, `handle_heartbeat` to NgrokSession.

## 0.2.3:

- Initial public release.
