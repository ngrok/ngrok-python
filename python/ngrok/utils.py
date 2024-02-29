import typing as t
from contextlib import contextmanager
import ngrok


@contextmanager
def scoped_endpoint(*args, **kwds):
    listener = ngrok.forward(*args, **kwds)
    try:
        yield listener
    finally:
        ngrok.disconnect(listener.url())
