import typing as t
from contextlib import contextmanager
import ngrok
import functools
import asyncio


@contextmanager
def scoped_endpoint(*args, **kwds):
    listener = ngrok.forward(*args, **kwds)
    try:
        yield listener
    finally:
        ngrok.disconnect(listener.url())
