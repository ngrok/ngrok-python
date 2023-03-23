#!/usr/bin/env python

import asyncio
from http.server import BaseHTTPRequestHandler
import logging
import ngrok
import socketserver
import threading

logging.basicConfig(level=logging.INFO)
pipe = ngrok.pipe_name()

async def create_tunnel():
  # create session
  session = (await ngrok.NgrokSessionBuilder().authtoken_from_env()
    # .authtoken("<authtoken>")
    .metadata("Online in One Line")
    .connect()
  )
  # create tunnel
  tunnel = (await session.tls_endpoint()
    # .allow_cidr("0.0.0.0/0")
    # .deny_cidr("10.1.1.1/32")
    # .domain("<somedomain>.ngrok.io")
    # .forwards_to("example python")
    # .mutual_tlsca(load_file("ca.crt"))
    # .proxy_proto("") # One of: "", "1", "2"
    .termination(load_file("domain.crt"), load_file("domain.key"))
    .metadata("example tunnel metadata from python")
    .listen()
  )
  await tunnel.forward_pipe(pipe)

def load_file(name):
  with open("examples/{}".format(name), "r") as crt:
    return bytearray(crt.read().encode())

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
