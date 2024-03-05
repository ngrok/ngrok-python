#!/usr/bin/env python

import asyncio, logging, ngrok, os, socketserver, threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from typing import Union
from socketserver import TCPServer, UnixStreamServer


async def create_listener() -> ngrok.Listener:
    # create session
    session = (
        await ngrok.SessionBuilder()
        .authtoken_from_env()
        .metadata("Online in One Line")
        .connect()
    )
    # create listener
    return (
        await session.labeled_listener()
        .label("edge", "edghts_<edge_id>")
        .metadata("example listener metadata from python")
        # Set the application protocol to "http1" or "http2"
        # .app_protocol("http2")
        .listen()
    )


# Example of using the ngrok.forward convenience function for labeled ingress
def create_listener_connect(addr):
    if addr.startswith("/"):
        addr = f"unix:{addr}"
    ngrok.forward(
        addr,
        authtoken_from_env=True,
        labels="edge:edghts_<edge_id>",
        proto="labeled",
        # Set the application protocol to "http1" or "http2"
        # app_protocol="http2",
    )


class HelloHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        body = bytes("<html><body>Hello</body></html>", "utf-8")
        self.protocol_version = "HTTP/1.1"
        self.send_response(200)
        self.send_header("Content-type", "text/html")
        self.send_header("Content-Length", len(body))
        self.end_headers()
        self.wfile.write(body)


httpd: Union[TCPServer, UnixStreamServer] = ThreadingHTTPServer(
    ("localhost", 0), HelloHandler
)
if os.name != "nt":
    # Set up a unix socket wrapper around standard http server
    class UnixSocketHttpServer(socketserver.UnixStreamServer):
        def get_request(self):
            request, client_address = super(UnixSocketHttpServer, self).get_request()
            return (request, ["local", 0])

    httpd = UnixSocketHttpServer((ngrok.pipe_name()), HelloHandler)

logging.basicConfig(level=logging.INFO)
# create_listener_connect(httpd.server_address)
listener = asyncio.run(create_listener())
ngrok.listen(httpd, listener)
httpd.serve_forever()
