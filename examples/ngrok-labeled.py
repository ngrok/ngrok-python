#!/usr/bin/env python

import asyncio, logging, ngrok, os, socketserver, threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer


async def create_tunnel():
    # create session
    session = (
        await ngrok.NgrokSessionBuilder()
        .authtoken_from_env()
        .metadata("Online in One Line")
        .connect()
    )
    # create tunnel
    return (
        await session.labeled_tunnel()
        .label("edge", "edghts_<edge_id>")
        .metadata("example tunnel metadata from python")
        .listen()
    )


# Example of using the ngrok.connect convenience function for labeled ingress
def create_tunnel_connect(addr):
    if addr.startswith("/"):
        addr = f"pipe:{addr}"
    ngrok.connect(
        addr,
        authtoken_from_env=True,
        labels="edge:edghts_<edge_id>",
        proto="labeled",
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


httpd = ThreadingHTTPServer(("localhost", 0), HelloHandler)
if os.name != "nt":
    # Set up a unix socket wrapper around standard http server
    class UnixSocketHttpServer(socketserver.UnixStreamServer):
        def get_request(self):
            request, client_address = super(UnixSocketHttpServer, self).get_request()
            return (request, ["local", 0])

    httpd = UnixSocketHttpServer((ngrok.pipe_name()), HelloHandler)

logging.basicConfig(level=logging.INFO)
# create_tunnel_connect(httpd.server_address)
tunnel = asyncio.run(create_tunnel())
ngrok.listen(httpd, tunnel)
httpd.serve_forever()
