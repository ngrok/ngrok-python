#!/usr/bin/env python

import asyncio
import flask
from ngrok import NgrokSessionBuilder
import os

# start tunnel
async def create_tunnel():
  session = await NgrokSessionBuilder().authtoken_from_env().connect()
  tunnel = await session.http_endpoint().listen()
  print("tunnel at: {}".format(tunnel.url()))
  tunnel.forward_tcp('localhost:9000')

loop = asyncio.new_event_loop()
loop.run_until_complete(create_tunnel())

if __name__ == "__main__":
  app = flask.Flask(__name__)
  @app.route('/')
  def hello():
    return 'Hello, World!'
  app.run(host='localhost', port=9000, debug=True)
