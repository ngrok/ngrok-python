#!/usr/bin/env python

import asyncio
from http.server import BaseHTTPRequestHandler
import logging
import ngrok
import os
import socketserver
import threading

pipe = ngrok.pipe_name()
logging.basicConfig(level=logging.INFO, \
  format='%(asctime)-15s %(levelname)s %(name)s %(filename)s:%(lineno)d %(message)s')

# To enable more verbose logging:
# logging.getLogger().setLevel(5)
# ngrok.log_level("TRACE")

def on_stop():
  print("on_stop")

def on_restart():
  print("on_restart")

def on_update(version, permit_major_version):
  print("on_update, version: {}, permit_major_version: {}".format(version, permit_major_version))

async def create_tunnel():
  # create session
  session = (await ngrok.NgrokSessionBuilder().authtoken_from_env()
    # .authtoken("<authtoken>")
    .metadata("Online in One Line")
    .handle_stop_command(on_stop)
    .handle_restart_command(on_restart)
    .handle_update_command(on_update)
    .connect()
  )
  # create tunnel
  tunnel = (await session.http_endpoint()
    # .allow_cidr("0.0.0.0/0")
    # .basic_auth("ngrok", "online1line")
    # .circuit_breaker(0.5)
    # .compression()
    # .deny_cidr("10.1.1.1/32")
    # .domain("<somedomain>.ngrok.io")
    # .mutual_tlsca(load_file("ca.crt"))
    # .oauth("google", ["<user>@<domain>"], ["<domain>"], ["<scope>"])
    # .oidc("<url>", "<id>", "<secret>", ["<user>@<domain>"], ["<domain>"], ["<scope>"])
    # .proxy_proto("") # One of: "", "1", "2"
    # .remove_request_header("X-Req-Nope")
    # .remove_response_header("X-Res-Nope")
    # .request_header("X-Req-Yup", "true")
    # .response_header("X-Res-Yup", "true")
    # .scheme("HTTPS")
    # .websocket_tcp_conversion()
    # .webhook_verification("twilio", "asdf")
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
