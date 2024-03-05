import asyncio
import ngrok
import logging
from hypercorn.config import Config
from hypercorn.asyncio import serve

config = Config()


async def app(scope, receive, send):
    await send(
        {
            "type": "http.response.start",
            "status": 200,
            "headers": [
                (b"content-type", b"text/plain"),
            ],
        }
    )
    await send(
        {
            "type": "http.response.body",
            "body": b"hello",
        }
    )


logging.basicConfig(level=logging.INFO)
ngrok.forward(addr="localhost:8080", authtoken_from_env=True, app_protocol="http2")
config.bind = ["localhost:8080"]
asyncio.run(serve(app, config))
