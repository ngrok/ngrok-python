#!/usr/bin/env python

from http.server import HTTPServer, BaseHTTPRequestHandler
import logging, ngrok, os
import asyncio


class HelloHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        body = bytes("Hello!", "utf-8")
        self.protocol_version = "HTTP/1.1"
        self.send_response(200)
        self.send_header("Content-Length", len(body))
        self.end_headers()
        self.wfile.write(body)


def load_file(name):
    with open("examples/{}".format(name), "r") as crt:
        return bytearray(crt.read().encode())


logging.basicConfig(level=logging.INFO)
server = HTTPServer(("localhost", 8080), HelloHandler)
listener = ngrok.forward(
    # session configuration
    addr="localhost:8080",
    # allow_user_agent="^mozilla.*",
    # authtoken="<authtoken>",
    authtoken_from_env=True,
    session_metadata="Online in One Line",
    # listener configuration
    basic_auth=["ngrok:online1line"],
    circuit_breaker=0.1,
    compression=True,
    # deny_user_agent="^curl.*",
    # domain="<domain>",
    allow_cidr="0.0.0.0/0",
    deny_cidr="10.1.1.1/32",
    metadata="example listener metadata from python",
    # mutual_tls_cas=load_file("ca.crt"),
    # oauth_provider="google",
    # oauth_allow_domains=["<domain>"],
    # oauth_allow_emails=["<email>"],
    # oauth_scopes=["<scope>"],
    # oauth_client_id="<id>",
    # oauth_client_secret="<secret>",
    # oidc_issuer_url="<url>",
    # oidc_client_id="<id>",
    # oidc_client_secret="<secret>",
    # oidc_allow_domains=["<domain>"],
    # oidc_allow_emails=["<email>"],
    # oidc_scopes=["<scope>"],
    proxy_proto="",  # One of: "", "1", "2"
    request_header_remove="X-Req-Nope",
    response_header_remove="X-Res-Nope",
    request_header_add="X-Req-Yup:true",
    response_header_add="X-Res-Yup:true",
    schemes=["HTTPS"],
    # verify_upstream_tls=True,
    # verify_webhook_provider="twilio",
    # verify_webhook_secret="asdf",
    # websocket_tcp_converter=True,
)
server.serve_forever()
