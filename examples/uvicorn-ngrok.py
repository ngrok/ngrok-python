#!/usr/bin/env python

import asyncio, logging, ngrok, os, uvicorn

logging.basicConfig(level=logging.INFO)


async def app(scope, receive, send):
    assert scope["type"] == "http"

    await send(
        {
            "type": "http.response.start",
            "status": 200,
            "headers": [[b"content-type", b"text/plain"]],
        }
    )
    await send(
        {
            "type": "http.response.body",
            "body": b"Hello, world!",
        }
    )


if os.name == "nt":  # windows

    async def setup():
        tunnel = await ngrok.default()
        tunnel.forward_tcp("localhost:8000")

    asyncio.run(setup())
    uvicorn.run(app=app)
    exit()

uvicorn.run(app=app, fd=ngrok.fd())
