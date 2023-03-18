#!/usr/bin/env python

import asyncio
from aiohttp import web
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
async def hello(request):
  return web.Response(text="Hello, world")

app = web.Application()
app.add_routes([web.get('/', hello)])
web.run_app(app, path=UNIX_SOCKET, loop=loop)
