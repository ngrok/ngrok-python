#!/usr/bin/env python

import asyncio, logging, ngrok, os, socketserver, threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

pipe = ngrok.pipe_name()
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)-15s %(levelname)s %(name)s %(filename)s:%(lineno)d %(message)s",
)


def on_stop():
    print("stop command")


def on_restart():
    print("restart command")


def on_update(version, permit_major_version):
    print(
        f"update command, version: {version}, permit_major_version: {permit_major_version}"
    )


def on_heartbeat(latency):
    print(f"heartbeat, latency: {latency} milliseconds")


def on_disconnection(addr, error):
    print(f"connecting, addr: {addr} error: {error}")


async def create_tunnel(httpd):
    # create session
    session = (
        await ngrok.NgrokSessionBuilder()
        .authtoken_from_env()
        # .authtoken("<authtoken>")
        .metadata("Online in One Line")
        .handle_stop_command(on_stop)
        .handle_restart_command(on_restart)
        .handle_update_command(on_update)
        .handle_heartbeat(on_heartbeat)
        .handle_disconnection(on_disconnection)
        .connect()
    )
    # create tunnel
    tunnel = (
        await session.http_endpoint()
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
        .metadata("example tunnel metadata from python").listen()
    )
    sock = httpd.server_address
    tunnel.forward_pipe(sock) if os.name != "nt" else tunnel.forward_tcp(
        f"localhost:{sock[1]}"
    )


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


httpd = ThreadingHTTPServer(("localhost", 0), HelloHandler)
if os.name != "nt":
    # Set up a unix socket wrapper around standard http server
    class UnixSocketHttpServer(socketserver.UnixStreamServer):
        def get_request(self):
            request, client_address = super(UnixSocketHttpServer, self).get_request()
            return (request, ["local", 0])

    httpd = UnixSocketHttpServer((ngrok.pipe_name()), HelloHandler)

# To enable more verbose logging:
# logging.getLogger().setLevel(5)
# ngrok.log_level("TRACE")

logging.basicConfig(level=logging.INFO)
asyncio.run(create_tunnel(httpd))
httpd.serve_forever()
