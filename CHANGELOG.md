## 0.8.0:

* Add `NgrokSession.client_info()`
* Rename to `ngrok-python`

## 0.7.0:

* Add `ngrok.disconnect(url)` and `ngrok.kill()` convenience functions.
* Add examples for [Tornado](https://www.tornadoweb.org), [OpenPlayground](https://github.com/nat/openplayground), [Streamlit](https://streamlit.io/), [GPT4All](https://github.com/nomic-ai/gpt4all).
* Handle protocols in `addr`, and dots in `connect` option keys, for ease-of-use.
* [Docs](https://ngrok.github.io/ngrok-python/) improvements

## 0.6.0:

* Add `ngrok.connect(options)` convenience function.
* Add [Gradio](https://gradio.app/) example.
* Consolidate multiple tunnel classes into one.

## 0.5.0:

* Add ngrok-asgi ASGI runner.
* Migrate `ca_cert` to the upstream call in `ngrok-rust`.

## 0.4.0:

* Django, auto-format, and windows support in examples.

## 0.3.0:

* Add wrapper convenience functions.
* Add `close`, `ca_cert`, `handle_disconnection`, `handle_heartbeat` to NgrokSession.

## 0.2.3:

* Initial public release.
