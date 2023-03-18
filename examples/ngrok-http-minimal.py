#!/usr/bin/env python

import asyncio
from http.server import HTTPServer, BaseHTTPRequestHandler
from ngrok import NgrokSessionBuilder

class HelloHandler(BaseHTTPRequestHandler):
  def do_GET(self):
    body = bytes("Hello", "utf-8")
    self.protocol_version = "HTTP/1.1"
    self.send_response(200)
    self.send_header("Content-Length", len(body))
    self.end_headers()
    self.wfile.write(body)

async def create_tunnel():
  session = await NgrokSessionBuilder().authtoken_from_env().connect()
  tunnel = await session.http_endpoint().listen()
  print("established tunnel at: {}".format(tunnel.url()))
  tunnel.forward_tcp("localhost:9000")
  HTTPServer(('localhost',9000), HelloHandler).serve_forever()

asyncio.run(create_tunnel())
