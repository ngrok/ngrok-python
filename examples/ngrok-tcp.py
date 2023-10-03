#!/usr/bin/env python

import asyncio, logging, ngrok, os, socketserver, threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from typing import Union
from socketserver import TCPServer, UnixStreamServer


async def create_listener() -> ngrok.Listener:
    # create session
    session: ngrok.Session = (
        await ngrok.SessionBuilder()
        .authtoken_from_env()
        .metadata("Online in One Line")
        .connect()
    )
    # create listener
    return (
        await session.tcp_endpoint()
        # .allow_cidr("0.0.0.0/0")
        # .deny_cidr("10.1.1.1/32")
        # .forwards_to("example python")
        # .proxy_proto("") # One of: "", "1", "2"
        # .remote_addr("<n>.tcp.ngrok.io:<p>")
        .metadata("example listener metadata from python").listen()
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
    class UnixSocketHttpServer(UnixStreamServer):
        def get_request(self):
            request, client_address = super(UnixSocketHttpServer, self).get_request()
            return (request, ["local", 0])

    httpd = UnixSocketHttpServer((ngrok.pipe_name()), HelloHandler)

logging.basicConfig(level=logging.INFO)
listener = asyncio.run(create_listener())
ngrok.listen(httpd, listener)
httpd.serve_forever()
