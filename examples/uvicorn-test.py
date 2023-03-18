#!/usr/bin/env python

import asyncio
from uvicorn import Config, Server
from ngrok import NgrokSessionBuilder
import os

UNIX_SOCKET = "/tmp/http.socket"
if os.path.exists(UNIX_SOCKET):
    os.remove(UNIX_SOCKET)

# start tunnel
async def create_tunnel():
  session = await NgrokSessionBuilder().authtoken_from_env().connect()
  tunnel = await session.http_endpoint().listen()
  print("established tunnel at: {}".format(tunnel.url()))
  await tunnel.forward_pipe(UNIX_SOCKET)

loop = asyncio.new_event_loop()
loop.create_task(create_tunnel())

# start web server
async def app(scope, receive, send):
  assert scope['type'] == 'http'

  await send({
    'type': 'http.response.start',
    'status': 200,
    'headers': [
      [b'content-type', b'text/plain'],
    ],
  })
  await send({
    'type': 'http.response.body',
    'body': b'Hello, world!',
  })

# cannot pass in the loop instance like aiohttp allows
server = Server(Config(app=app, uds=UNIX_SOCKET, loop="none"))
loop.run_until_complete(server.serve())
