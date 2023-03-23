#!/usr/bin/env python

import asyncio, logging, ngrok, socketserver, threading
from http.server import BaseHTTPRequestHandler

logging.basicConfig(level=logging.INFO)
pipe = ngrok.pipe_name()

async def create_tunnel():
  # create session
  session = (await ngrok.NgrokSessionBuilder().authtoken_from_env()
    .metadata("Online in One Line")
    .connect()
  )
  # create tunnel
  tunnel = (await session.labeled_tunnel()
    .label("edge", "edghts_<edge_id>")
    .metadata("example tunnel metadata from python")
    .listen()
  )
  await tunnel.forward_pipe(pipe)

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

httpd = UnixSocketHttpServer((pipe), HelloHandler)
threading.Thread(target=httpd.serve_forever, daemon=True).start()
asyncio.run(create_tunnel())
