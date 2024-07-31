from aiohttp import web, ClientSession
from aiohttp.web_runner import GracefulExit
from collections import defaultdict
from http.server import HTTPServer, BaseHTTPRequestHandler
from requests.adapters import HTTPAdapter
from requests.packages.urllib3.util.retry import Retry
import asyncio
import ngrok
import os
import random
import requests
import socketserver
import threading
import unittest
import logging

logging.basicConfig(level=logging.DEBUG)

expected = "Hello"


def retry_request():
    s = requests.Session()
    retries = Retry(
        total=5, backoff_factor=1, status_forcelist=[404, 429, 500, 502, 503, 504]
    )
    adapter = HTTPAdapter(max_retries=retries)
    s.mount("http://", adapter)
    s.mount("https://", adapter)
    return s


class HelloHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        body = bytes(expected, "utf-8")
        self.protocol_version = "HTTP/1.1"
        code = 200
        if self.path == "/headers":
            if self.headers["foo"] != "bar" or self.headers["baz"] is not None:
                code = 555

        self.send_response(code)
        self.send_header("Content-Length", len(body))
        self.send_header("python", "sss")
        self.end_headers()
        self.wfile.write(body)


class UnixSocketHttpServer(socketserver.UnixStreamServer):
    def get_request(self):
        request, _ = super(UnixSocketHttpServer, self).get_request()
        return (request, ["local", 0])

    def server_close(self):
        super().server_close()
        if os.path.exists(self.listen_to):
            os.remove(self.listen_to)


def make_http(use_unix_socket=False):
    server = None
    if use_unix_socket:
        listen_to = "tun-{}".format(random.randrange(0, 1000000))
        server = UnixSocketHttpServer(listen_to, HelloHandler)
        server.listen_to = listen_to
    else:
        server = HTTPServer(("localhost", 0), HelloHandler)
        addr = server.server_address
        server.listen_to = "{}:{}".format(addr[0], addr[1])

    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    return server


async def make_session():
    return await ngrok.SessionBuilder().authtoken_from_env().connect()


async def make_http_and_session(use_unix_socket=False):
    return make_http(use_unix_socket), await make_session()


async def shutdown(listener, http_server):
    await listener.close()
    http_server.shutdown()
    http_server.server_close()


class TestNgrok(unittest.IsolatedAsyncioTestCase):
    async def validate_http_request(self, url, requests_config=dict()):
        response = retry_request().get(url, **requests_config)
        self.assertEqual(200, response.status_code)
        self.assertEqual(expected, response.text)
        return response

    async def forward_validate_shutdown(
        self, http_server, listener, url, requests_config=dict()
    ):
        listener.forward(http_server.listen_to)
        response = await self.validate_http_request(url, requests_config)
        await shutdown(listener, http_server)
        return response

    async def test_import(self):
        session_builder = ngrok.SessionBuilder()
        self.assertIsNotNone(session_builder)

    async def test_https_listener(self):
        http_server, session = await make_http_and_session()
        listener = (
            await session.http_endpoint()
            .forwards_to("http forwards to")
            .metadata("http metadata")
            .listen()
        )

        self.assertIsNotNone(listener.id())
        self.assertIsNotNone(listener.url())
        self.assertTrue(listener.url().startswith("https://"))
        self.assertEqual("http forwards to", listener.forwards_to())
        self.assertEqual("http metadata", listener.metadata())
        listener_list = await session.get_listeners()
        self.assertEqual(1, len(listener_list))
        self.assertEqual(listener.id(), listener_list[0].id())
        self.assertEqual(listener.url(), listener_list[0].url())

        await self.forward_validate_shutdown(http_server, listener, listener.url())

    async def test_http_listener(self):
        http_server, session = await make_http_and_session()
        listener = await session.http_endpoint().scheme("hTtP").listen()
        self.assertTrue(listener.url().startswith("http://"))
        await self.forward_validate_shutdown(http_server, listener, listener.url())

    async def test_https_listener_with_policy(self):
        policy = """
        {
          "inbound": [],
          "outbound": [
            {
              "expressions": [],
              "name": "",
              "actions": [
                {
                  "type": "add-headers",
                  "config": {
                    "headers": {
                      "added-header": "added-header-value"
                    }
                  }
                }
              ]
            }
          ]
        }
        """

        http_server, session = await make_http_and_session()
        listener = await session.http_endpoint().policy(policy).listen()
        listener.forward(http_server.listen_to)
        response = retry_request().get(listener.url())
        self.assertEqual("added-header-value", response.headers["added-header"])
        await shutdown(listener, http_server)

    async def test_https_listener_with_traffic_policy(self):
        traffic_policy = """
        {
          "inbound": [],
          "outbound": [
            {
              "expressions": [],
              "name": "",
              "actions": [
                {
                  "type": "add-headers",
                  "config": {
                    "headers": {
                      "added-header": "added-header-value"
                    }
                  }
                }
              ]
            }
          ]
        }
        """

        http_server, session = await make_http_and_session()
        listener = await session.http_endpoint().traffic_policy(traffic_policy).listen()
        listener.forward(http_server.listen_to)
        response = retry_request().get(listener.url())
        self.assertEqual("added-header-value", response.headers["added-header"])
        await shutdown(listener, http_server)

    async def test_https_listener_with_invalid_policy_json(self):
        error = None
        try:
            _, session = await make_http_and_session()
            await session.http_endpoint().policy('{ "inbound": "not valid" }').listen()
        except ValueError as err:
            error = err
        self.assertIsInstance(error, ValueError)
        self.assertTrue("provided is invalid" in f"{error}")

    async def test_https_listener_with_invalid_traffic_policy_json(self):
        error = None
        try:
            _, session = await make_http_and_session()
            await session.http_endpoint().traffic_policy(
                '{ "inbound": "not valid" }'
            ).listen()
        except ValueError as err:
            error = err
        self.assertIsInstance(error, ValueError)
        self.assertTrue("provided is invalid" in f"{error}")

    async def test_https_listener_with_invalid_policy_action(self):
        policy = """
        {
          "inbound": [],
          "outbound": [
            {
              "expressions": [],
              "name": "",
              "actions": [
                {
                  "type": "not-real-action",
                  "config": {
                    "headers": {
                      "added-header": "added-header-value"
                    }
                  }
                }
              ]
            }
          ]
        }
        """

        try:
            http_server, session = await make_http_and_session()
            listener = await session.http_endpoint().policy(policy).listen()
            listener.forward(http_server.listen_to)
            _ = retry_request().get(listener.url())
        except ValueError as err:
            error = err
        self.assertIsInstance(error, ValueError)
        self.assertTrue("Invalid policy action type 'not-real-action'." in f"{error}")

    async def test_https_listener_with_invalid_traffic_policy_action(self):
        traffic_policy = """
        {
          "inbound": [],
          "outbound": [
            {
              "expressions": [],
              "name": "",
              "actions": [
                {
                  "type": "not-real-action",
                  "config": {
                    "headers": {
                      "added-header": "added-header-value"
                    }
                  }
                }
              ]
            }
          ]
        }
        """

        try:
            http_server, session = await make_http_and_session()
            listener = (
                await session.http_endpoint().traffic_policy(traffic_policy).listen()
            )
            listener.forward(http_server.listen_to)
            _ = retry_request().get(listener.url())
        except ValueError as err:
            error = err
        self.assertIsInstance(error, ValueError)
        self.assertTrue("Invalid policy action type 'not-real-action'." in f"{error}")

    async def test_unix_socket(self):
        http_server, session = await make_http_and_session(use_unix_socket=True)
        listener = await session.http_endpoint().listen()
        self.assertTrue(http_server.listen_to.startswith("tun-"))

        listener.forward(f"unix:{http_server.listen_to}")

        await self.validate_http_request(listener.url())
        await shutdown(listener, http_server)

    async def test_listen_and_serve(self):
        http_server, session = await make_http_and_session()
        listener = await session.http_endpoint().listen_and_serve(http_server)
        await self.validate_http_request(listener.url())
        await shutdown(listener, http_server)

    async def test_listen_and_serve_unix(self):
        http_server, session = await make_http_and_session(use_unix_socket=True)
        listener = await session.http_endpoint().listen_and_serve(http_server)
        await self.validate_http_request(listener.url())
        await shutdown(listener, http_server)

    async def test_gzip_listener(self):
        http_server, session = await make_http_and_session()
        listener = await session.http_endpoint().compression().listen()

        listener.forward(http_server.listen_to)

        response = retry_request().get(listener.url())
        self.assertEqual("gzip", response.headers["content-encoding"])
        await shutdown(listener, http_server)

    async def test_tls_backend(self):
        session = await make_session()
        listener = await session.http_endpoint().listen_and_forward(
            "https://dashboard.ngrok.com"
        )

        response = retry_request().get(listener.url())
        self.assertEqual(421, response.status_code)
        self.assertTrue("different Host" in response.text)
        await listener.close()

    async def test_tls_backend_no_verify(self):
        session = await make_session()
        listener = (
            await session.http_endpoint()
            .verify_upstream_tls(False)
            .listen_and_forward("https://dashboard.ngrok.com")
        )

        response = retry_request().get(listener.url())
        self.assertEqual(421, response.status_code)
        self.assertTrue("different Host" in response.text)
        await listener.close()

    async def test_http_headers(self):
        http_server, session = await make_http_and_session()
        listener = (
            await session.http_endpoint()
            .request_header("foo", "bar")
            .remove_request_header("baz")
            .response_header("spam", "eggs")
            .remove_response_header("python")
            .listen()
        )

        config = defaultdict(dict)
        config["headers"]["baz"] = "req"
        await self.forward_validate_shutdown(
            http_server, listener, "{}/headers".format(listener.url()), config
        )

    async def test_basic_auth(self):
        http_server, session = await make_http_and_session()
        listener = (
            await session.http_endpoint().basic_auth("ngrok", "online1line").listen()
        )
        listener.forward(http_server.listen_to)

        config = {"auth": ("ngrok", "online1line")}
        response = await self.forward_validate_shutdown(
            http_server, listener, listener.url(), config
        )

    async def test_oauth(self):
        http_server, session = await make_http_and_session()
        listener = await session.http_endpoint().oauth("google").listen()

        listener.forward(http_server.listen_to)
        response = retry_request().get(listener.url())
        self.assertEqual(200, response.status_code)
        text = response.text[0:15000]
        print(f'-------- text: "{text}"')
        self.assertTrue(
            "google-site-verification" in response.text
            or "accounts.google.com" in response.text
        )
        await shutdown(listener, http_server)

    async def test_custom_domain(self):
        domain = "d{}.ngrok.io".format(random.randrange(0, 1000000))
        http_server, session = await make_http_and_session()
        listener = await session.http_endpoint().domain(domain).listen()

        self.assertEqual("https://" + domain, listener.url())

        await self.forward_validate_shutdown(http_server, listener, listener.url())

    async def test_proxy_proto(self):
        class ProxyHandler(socketserver.StreamRequestHandler):
            read_value = None

            def handle(self):
                ProxyHandler.read_value = self.rfile.read(10)

        tcp_server = socketserver.TCPServer(("localhost", 0), ProxyHandler)
        thread = threading.Thread(target=tcp_server.serve_forever, daemon=True)
        thread.start()
        session = await make_session()
        listener = await session.http_endpoint().proxy_proto("1").listen()

        addr = tcp_server.server_address
        tcp_server.listen_to = "{}:{}".format(addr[0], addr[1])
        listener.forward(tcp_server.listen_to)
        try:
            resp = requests.get(listener.url(), timeout=3)
        except requests.exceptions.ReadTimeout as err:
            pass
        self.assertEqual(b"PROXY TCP4", ProxyHandler.read_value)
        await shutdown(listener, tcp_server)

    async def test_ip_restriction_http(self):
        http_server, session = await make_http_and_session()
        error = await self.ip_restriction(http_server, session.http_endpoint())
        self.assertEqual(403, error.status_code)

    async def test_ip_restriction_tcp(self):
        http_server, session = await make_http_and_session()
        error = await self.ip_restriction(http_server, session.tcp_endpoint())
        self.assertIsInstance(error, requests.exceptions.ConnectionError)

    async def ip_restriction(self, http_server, listener_builder):
        listener = (
            await listener_builder.allow_cidr("127.0.0.1/32")
            .deny_cidr("0.0.0.0/0")
            .listen()
        )

        listener.forward(http_server.listen_to)
        error = None
        try:
            error = requests.get(listener.url().replace("tcp:", "http:"))
        except requests.exceptions.ConnectionError as err:
            error = err
        await shutdown(listener, http_server)
        return error

    async def test_websocket_conversion(self):
        http_server, session = await make_http_and_session()
        listener = await session.http_endpoint().websocket_tcp_conversion().listen()

        listener.forward(http_server.listen_to)

        response = retry_request().get(listener.url())
        self.assertEqual(400, response.status_code)
        # ERR_NGROK_3206: Expected a websocket request with a "Connection: upgrade" header
        # but did not receive one.
        self.assertEqual("ERR_NGROK_3206", response.headers["ngrok-error-code"])

        await shutdown(listener, http_server)

    async def test_user_agent(self):
        http_server, session = await make_http_and_session()
        listener = (
            await session.http_endpoint()
            .allow_user_agent("^curl.*")
            .deny_user_agent(".*")
            .listen()
        )

        listener.forward(http_server.listen_to)

        response = retry_request().get(listener.url())
        self.assertEqual(403, response.status_code)
        # ERR_NGROK_3206: The server does not authorize requests from your user-agent
        self.assertEqual("ERR_NGROK_3211", response.headers["ngrok-error-code"])

        await shutdown(listener, http_server)

    async def test_tcp_listener(self):
        http_server, session = await make_http_and_session()
        listener = (
            await session.tcp_endpoint()
            .forwards_to("tcp forwards to")
            .metadata("tcp metadata")
            .listen()
        )

        self.assertEqual("tcp forwards to", listener.forwards_to())
        self.assertEqual("tcp metadata", listener.metadata())

        await self.forward_validate_shutdown(
            http_server, listener, listener.url().replace("tcp:", "http:")
        )

    async def test_tls_listener(self):
        http_server, session = await make_http_and_session()
        listener = (
            await session.tls_endpoint()
            .forwards_to("tls forwards to")
            .metadata("tls metadata")
            .listen()
        )

        self.assertEqual("tls forwards to", listener.forwards_to())
        self.assertEqual("tls metadata", listener.metadata())

        listener.forward(http_server.listen_to)

        error = None
        try:
            response = requests.get(listener.url().replace("tls:", "https:"))
        except requests.exceptions.SSLError as err:
            error = err
        self.assertIsInstance(error, requests.exceptions.SSLError)

        await shutdown(listener, http_server)

    async def test_standard_listen(self):
        http_server = make_http()
        listener1 = await ngrok.listen()
        listener2 = await ngrok.listen(listener=listener1)
        self.assertEqual(listener1.url(), listener2.url())
        listener3 = await ngrok.listen(http_server, listener2)
        self.assertEqual(listener2.url(), listener3.url())
        await self.forward_validate_shutdown(http_server, listener3, listener3.url())

    async def test_standard_listen_server(self):
        http_server = make_http()
        listener = await ngrok.listen(http_server)
        await self.forward_validate_shutdown(http_server, listener, listener.url())

    def test_aiohttp_listen(self):
        async def hello(request):
            return web.Response(text=expected)

        async def shutdown(request):
            # workaround of not having a close after run_app
            raise GracefulExit()

        loop = asyncio.new_event_loop()
        app = web.Application()
        app.add_routes([web.get("/", hello)])
        app.add_routes([web.get("/shutdown", shutdown)])
        listener = ngrok.listen()

        async def validate():
            # have to use an async http client
            async with ClientSession() as client:
                # test that listener to server works
                async with client.get(listener.url()) as response:
                    self.assertEqual(200, response.status)
                    self.assertEqual(expected, await response.text())
                # shutdown server
                await client.get("{}/shutdown".format(listener.url()))
            # shutdown listener
            await listener.close()

        loop.create_task(validate())
        try:
            web.run_app(app, sock=listener, loop=loop)
        except GracefulExit:
            pass

    def test_pipe_name(self):
        pipe_name = ngrok.pipe_name()
        self.assertTrue("tun" in pipe_name)

    async def test_werkzeug_develop(self):
        listener = await ngrok.werkzeug_develop()
        self.assertIsNotNone(listener.fd)
        self.assertEqual(os.environ["WERKZEUG_SERVER_FD"], str(listener.fd))
        self.assertEqual(os.environ["WERKZEUG_RUN_MAIN"], "true")
        await listener.close()

    async def test_default(self):
        session = await make_session()
        listener = await ngrok.default(session)
        self.assertTrue("http" in listener.url())
        await session.close()

    async def test_getsockname(self):
        session = await make_session()
        sockname = await ngrok.getsockname(session)
        self.assertTrue("tun" in sockname)
        await session.close()

    async def test_fd(self):
        session = await make_session()
        fd = await ngrok.fd(session)
        self.assertIsNotNone(fd)
        self.assertTrue(fd > 0)
        await session.close()

    async def test_listen_and_forward_multipass(self):
        http_server, session1 = await make_http_and_session()
        session2 = await make_session()
        url = "tcp://" + http_server.listen_to
        listener1 = await session1.http_endpoint().listen_and_forward(url)
        listener2 = await session1.http_endpoint().listen_and_forward(url)
        listener3 = await session2.http_endpoint().listen_and_forward(url)
        listener4 = await session2.tcp_endpoint().listen_and_forward(url)

        self.assertEqual(2, len(await session1.get_listeners()))
        self.assertEqual(2, len(await session2.get_listeners()))
        self.assertTrue(len(await ngrok.get_listeners()) >= 4)

        await self.validate_http_request(listener1.url())
        await self.validate_http_request(listener2.url())
        await self.validate_http_request(listener3.url())
        await self.validate_http_request(listener4.url().replace("tcp:", "http:"))
        await shutdown(listener1, http_server)
        await listener2.close()
        await listener3.close()
        await listener4.close()

    async def test_tcp_multipass(self):
        http_server, session1 = await make_http_and_session()
        session2 = await make_session()
        listener1 = await session1.http_endpoint().listen()
        listener2 = await session1.http_endpoint().listen()
        listener3 = await session2.http_endpoint().listen()
        listener4 = await session2.tcp_endpoint().listen()

        listener1.forward(http_server.listen_to)
        listener2.forward(http_server.listen_to)
        listener3.forward(http_server.listen_to)
        listener4.forward(http_server.listen_to)

        self.assertEqual(2, len(await session1.get_listeners()))
        self.assertEqual(2, len(await session2.get_listeners()))
        self.assertTrue(len(await ngrok.get_listeners()) >= 4)

        await self.validate_http_request(listener1.url())
        await self.validate_http_request(listener2.url())
        await self.validate_http_request(listener3.url())
        await self.validate_http_request(listener4.url().replace("tcp:", "http:"))
        await shutdown(listener1, http_server)
        await listener2.close()
        await listener3.close()
        await listener4.close()

    async def test_unix_multipass(self):
        http_server, session1 = await make_http_and_session(use_unix_socket=True)
        session2 = await make_session()
        listener1 = await session1.http_endpoint().listen()
        listener2 = await session1.http_endpoint().listen()
        listener3 = await session2.http_endpoint().listen()
        listener4 = await session2.tcp_endpoint().listen()

        listener1.forward(f"unix:{http_server.listen_to}")
        listener2.forward(f"unix:{http_server.listen_to}")
        listener3.forward(f"unix:{http_server.listen_to}")
        listener4.forward(f"unix:{http_server.listen_to}")

        await self.validate_http_request(listener1.url())
        await self.validate_http_request(listener2.url())
        await self.validate_http_request(listener3.url())
        await self.validate_http_request(listener4.url().replace("tcp:", "http:"))

        await shutdown(listener1, http_server)
        await listener2.close()
        await listener3.close()
        await listener4.close()

    async def test_connect_heartbeat_callbacks(self):
        global disconn_addr
        global test_latency
        disconn_addr = None

        def on_heartbeat(latency):
            global test_latency
            test_latency = latency

        def on_disconn(addr, err):
            global disconn_addr
            disconn_addr = addr

        builder = ngrok.SessionBuilder()
        builder.authtoken_from_env()
        builder.client_info("test_connect_heartbeat_callbacks", "1.2.3")
        (builder.handle_heartbeat(on_heartbeat).handle_disconnection(on_disconn))
        await builder.connect()
        self.assertTrue(test_latency > 0)
        self.assertEqual(None, disconn_addr)

    async def test_ca_cert(self):
        error = None
        cert = None
        with open("examples/domain.crt", "r") as crt:
            cert = bytearray(crt.read().encode())
        try:
            await ngrok.SessionBuilder().ca_cert(cert).connect()
        except ValueError as err:
            error = err
        self.assertIsInstance(error, ValueError)
        self.assertTrue("tls" in f"{error}")

    async def test_invalid_authtoken(self):
        error = None
        try:
            await ngrok.SessionBuilder().authtoken("notvalid").connect()
        except ValueError as err:
            error = err
        self.assertIsInstance(error, ValueError)
        self.assertEqual("ERR_NGROK_105", error.args[2])

    async def test_invalid_domain(self):
        session = await make_session()
        error = None
        try:
            await session.http_endpoint().domain("1.21 gigawatts").listen()
        except ValueError as err:
            error = err
        self.assertIsInstance(error, ValueError)
        self.assertEqual("ERR_NGROK_326", error.args[2])


if __name__ == "__main__":
    unittest.main()
