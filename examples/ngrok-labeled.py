#!/usr/bin/env python

import asyncio
from http.server import HTTPServer, BaseHTTPRequestHandler
import logging
import io
from ngrok import NgrokSessionBuilder, log_level
import os
import socket
import socketserver
import threading
import time

UNIX_SOCKET = "/tmp/http.socket"

async def create_tunnel():
  builder = NgrokSessionBuilder()
  session = (await builder.authtoken_from_env()
    .metadata("Online in One Line")
    .connect()
  )
  # create tunnel
  tunnel = (await session.labeled_tunnel()
    .label("edge", "edghts_<edge_id>")
    .metadata("example tunnel metadata from python")
    .listen()
  )
  print("established tunnel at: {}".format(tunnel.labels()))
  await tunnel.forward_pipe(UNIX_SOCKET)

class HelloHandler(BaseHTTPRequestHandler):
  def do_GET(self):
    body = bytes("<html><body>Hello</body></html>", "utf-8")
    self.protocol_version = "HTTP/1.1"
    self.send_response(200)
    self.send_header("Content-type", "text/html")
    self.send_header("Content-Length", len(body))
    self.end_headers()
    self.wfile.write(body)

# Set up a unix socket wrapper around standard http server
class UnixSocketHttpServer(socketserver.UnixStreamServer):
    def get_request(self):
        request, client_address = super(UnixSocketHttpServer, self).get_request()
        return (request, ["local", 0])

def start_unix_http_server():
  if os.path.exists(UNIX_SOCKET):
    os.remove(UNIX_SOCKET)
  httpd = UnixSocketHttpServer((UNIX_SOCKET), HelloHandler)
  thread = threading.Thread(target=httpd.serve_forever, daemon=True)
  thread.start()

start_unix_http_server()
loop = asyncio.new_event_loop()
loop.run_until_complete(create_tunnel())
loop.close()
