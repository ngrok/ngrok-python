#!/usr/bin/env python

from http.server import HTTPServer, BaseHTTPRequestHandler
import logging, ngrok, os
import asyncio


class HelloHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        body = bytes("Hello", "utf-8")
        self.protocol_version = "HTTP/1.1"
        self.send_response(200)
        self.send_header("Content-Length", len(body))
        self.end_headers()
        self.wfile.write(body)


logging.basicConfig(level=logging.INFO)
server = HTTPServer(("localhost", 8080), HelloHandler)
listener = ngrok.forward("localhost:8080", authtoken_from_env=True)
server.serve_forever()
