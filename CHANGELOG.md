## Unreleased

- Change how a session waits between reconnect attempts after it loses its connection, for every session: the wait starts at 50 ms and doubles up to 30 s, each wait scaled by a random factor in [0, 1). Before, the wait grew 50-fold per attempt with no cap and no random factor: about 2 minutes before the fourth attempt and 1 h 44 min before the fifth.
- Add `SessionBuilder.reconnect_backoff(initial_ms, max_ms)`, which sets the first wait between reconnect attempts and the cap the doubling waits stop at, and `SessionBuilder.reconnect_jitter(enabled)`, which turns off or on the random scaling of each wait. The defaults are 50 ms, 30 s and jitter on. `reconnect_backoff` refuses a start below 1 ms or above the cap, and a cap above one day, with a `ValueError`.
- Add `SessionBuilder.reconnect_max_attempts(attempts)`, which stops a session reconnecting after that many failed attempts in one outage; every listener then stops forwarding, and awaiting its `forward()` or `join()` raises the last attempt's error. The forwarding that `ngrok.forward` and `ngrok.listen(server)` start then stops without raising or logging. It refuses anything but an int from 1 to 4294967295. By default a session never stops reconnecting on its own.

## 1.7.0 
- Add support for CEL filtering when listing resources.
- Add support for service users
- Add support for `vault_name` on Secrets
-Make `pooling_enabled` on Endpoints optional

## 1.6.0

- Add support for internal endpoints

## 1.5.0:
- Adds `pooling_enabled` to listener builders, allowing the endpoint to pool with other endpoints with the same host/port/binding
- Adds support for Windows-aarch64

## 1.4.0:

- Rename `policy` to `traffic_policy` in Listener builders and `ngrok.forward`
- Fix quickstart example in README

## 1.3.0:

- Add `root_cas` to session builder and `ngrok.forward`. Setting this to `host` will use the host's trusted certificates to connect for the ngrok session.
- Add `session_ca_cert` and `server_addr` to `ngrok.forward`, which correspond to the same functions in the session builder.

## 1.2.0:

- Add `verify_upstream_tls` to listener builders and `ngrok.forward`. Setting this to false will skip verification of the upstream application's TLS certificate.

## 1.1.0:

- Add `app_protocol(string)` to http and labeled listener builders, and `app_protocol: string` as a `ngrok.forward()` argument. Setting this to "http2" will enable HTTP/2 support to the backend application.

## 1.0.0

- Add policy support to Listener builders and `ngrok.forward`

## 0.12.1

- Move to `ngrok.forward` from `ngrok.connect`, keeping an alias

## 0.12.0

- Add `allow_user_agent` and `deny_user_agent` options to HTTP listeners
- Add `client_id` and `client_secret` options to OAuth

## 0.11.0

- Rename all assets to listener
- Add links to ngrok documentation
- Log if an auth token was used for session creation

## 0.10.1

- Fix publishing for aarch64

## 0.10.0

- Add `listen_and_forward` and `listen_and_serve` to listener builders.
- Update to latest version of underlying rust library, allowing TLS backends.
- Fix aarch64 build for docker-ubuntu-on-macos and raspberry pi.

## 0.9.0

- Flattened `listener.forward_pipe()` and `listener.forward_tcp()` into `listener.forward()`. Determination will be made based on `addr` input.
- Added [Mypy](https://mypy.readthedocs.io/en/stable/) static typing information.
- Added `session.get_listeners()` to get a list of current non-closed listeners for the session.
- Added `ngrok.set_auth_token(<token>)` to set a default auth token to use for sessions.
- Added [ngrok error codes](https://ngrok.com/docs/errors/reference/) as 3rd argument to thrown Errors, when available.

## 0.8.1:

- Cleanly return from a listener forward call after a `session.close()`

## 0.8.0:

- Add `Session.client_info()`
- Rename to `ngrok-python`

## 0.7.0:

- Add `ngrok.disconnect(url)` and `ngrok.kill()` convenience functions.
- Add examples for [Tornado](https://www.tornadoweb.org), [OpenPlayground](https://github.com/nat/openplayground), [Streamlit](https://streamlit.io/), [GPT4All](https://github.com/nomic-ai/gpt4all).
- Handle protocols in `addr`, and dots in `connect` option keys, for ease-of-use.
- [Docs](https://ngrok.github.io/ngrok-python/) improvements

## 0.6.0:

- Add `ngrok.connect(options)` convenience function.
- Add [Gradio](https://gradio.app/) example.
- Consolidate multiple listener classes into one.

## 0.5.0:

- Add ngrok-asgi ASGI runner.
- Migrate `ca_cert` to the upstream call in `ngrok-rust`.

## 0.4.0:

- Django, auto-format, and windows support in examples.

## 0.3.0:

- Add wrapper convenience functions.
- Add `close`, `ca_cert`, `handle_disconnection`, `handle_heartbeat` to Session.

## 0.2.3:

- Initial public release.
