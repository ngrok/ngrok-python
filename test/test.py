from aiohttp import web, ClientSession
from aiohttp.web_runner import GracefulExit
from collections import defaultdict
from http.server import HTTPServer, BaseHTTPRequestHandler
import asyncio
import logging
import ngrok
import http
import os
import random
import requests
import socketserver
import threading
import unittest

expected = "Hello"

class HelloHandler(BaseHTTPRequestHandler):
  def do_GET(self):
    body = bytes(expected, "utf-8")
    self.protocol_version = "HTTP/1.1"
    code = 200
    if self.path == "/headers":
      if self.headers['foo'] != 'bar' or self.headers['baz'] is not None:
        code = 555

    self.send_response(code)
    self.send_header("Content-Length", len(body))
    self.send_header("python", "sss")
    self.end_headers()
    self.wfile.write(body)

class UnixSocketHttpServer(socketserver.UnixStreamServer):
  def get_request(self):
    request, client_address = super(UnixSocketHttpServer, self).get_request()
    return (request, ["local", 0])

  def server_close(self):
    super().server_close()
    if os.path.exists(self.listen_to):
      os.remove(self.listen_to)

def make_http(use_unix_socket=False):
  server = None
  if use_unix_socket:
    listen_to = "tun-{}".format(random.randrange(0,1000000))
    server = UnixSocketHttpServer(listen_to, HelloHandler)
    server.listen_to = listen_to
  else:
    server = HTTPServer(("localhost",0), HelloHandler)
    addr = server.server_address
    server.listen_to = "{}:{}".format(addr[0], addr[1])

  thread = threading.Thread(target=server.serve_forever, daemon=True)
  thread.start()
  return server

async def make_session():
  return await ngrok.NgrokSessionBuilder().authtoken_from_env().connect()

async def make_http_and_session(use_unix_socket=False):
  return make_http(use_unix_socket), await make_session()

async def shutdown(tunnel, http_server):
  await tunnel.close()
  http_server.shutdown()
  http_server.server_close()

class TestNgrok(unittest.IsolatedAsyncioTestCase):
  async def validate_http_request(self, url, requests_config=dict()):
    response = requests.get(url, **requests_config);
    self.assertEqual(200, response.status_code);
    self.assertEqual(expected, response.text);
    return response;

  async def forward_validate_shutdown(self, http_server, tunnel, url, requests_config=dict()):
    tunnel.forward_tcp(http_server.listen_to)
    response = await self.validate_http_request(url, requests_config);
    await shutdown(tunnel, http_server);
    return response;

  async def test_import(self):
    session_builder = ngrok.NgrokSessionBuilder()
    self.assertIsNotNone(session_builder)

  async def test_https_tunnel(self):
    http_server, session = await make_http_and_session()
    tunnel = (await session.http_endpoint()
      .forwards_to("http forwards to")
      .metadata("http metadata")
      .listen())

    self.assertIsNotNone(tunnel.id());
    self.assertIsNotNone(tunnel.url());
    self.assertTrue(tunnel.url().startswith("https://"))
    self.assertEqual("http forwards to", tunnel.forwards_to());
    self.assertEqual("http metadata", tunnel.metadata());
    await self.forward_validate_shutdown(http_server, tunnel, tunnel.url())

  async def test_http_tunnel(self):
    http_server, session = await make_http_and_session()
    tunnel = await session.http_endpoint().scheme("hTtP").listen()
    self.assertTrue(tunnel.url().startswith("http://"))
    await self.forward_validate_shutdown(http_server, tunnel, tunnel.url())

  async def test_pipe_socket(self):
    http_server, session = await make_http_and_session(True)
    tunnel = await session.http_endpoint().listen()
    self.assertTrue(http_server.listen_to.startswith("tun-"))
    tunnel.forward_pipe(http_server.listen_to)
    response = await self.validate_http_request(tunnel.url())
    await shutdown(tunnel, http_server)

  async def test_gzip_tunnel(self):
    http_server, session = await make_http_and_session()
    tunnel = (await session.http_endpoint()
      .compression()
      .listen())

    tunnel.forward_tcp(http_server.listen_to)

    response = requests.get(tunnel.url());
    self.assertEqual("gzip", response.headers["content-encoding"]);
    await shutdown(tunnel, http_server)

  async def test_http_headers(self):
    http_server, session = await make_http_and_session()
    tunnel = (await session.http_endpoint()
      .request_header("foo", "bar")
      .remove_request_header("baz")
      .response_header("spam", "eggs")
      .remove_response_header("python")
      .listen())

    config = defaultdict(dict)
    config['headers']['baz'] = 'req'
    await self.forward_validate_shutdown(http_server, tunnel,
      "{}/headers".format(tunnel.url()), config)

  async def test_basic_auth(self):
    http_server, session = await make_http_and_session()
    tunnel = (await session.http_endpoint()
      .basic_auth("ngrok", "online1line")
      .listen())

    tunnel.forward_tcp(http_server.listen_to)

    config = dict()
    config['auth'] = ('ngrok', 'online1line')
    response = await self.forward_validate_shutdown(http_server, tunnel, tunnel.url(),
      config)

  async def test_oauth(self):
    http_server, session = await make_http_and_session()
    tunnel = (await session.http_endpoint()
      .oauth("google")
      .listen())

    tunnel.forward_tcp(http_server.listen_to)
    response = requests.get(tunnel.url());
    self.assertEqual(200, response.status_code)
    self.assertTrue("google-site-verification" in response.text)
    await shutdown(tunnel, http_server)

  async def test_custom_domain(self):
    domain = "d{}.ngrok.io".format(random.randrange(0,1000000))
    http_server, session = await make_http_and_session()
    tunnel = (await session.http_endpoint()
      .domain(domain)
      .listen())

    self.assertEqual("https://" + domain, tunnel.url());

    await self.forward_validate_shutdown(http_server, tunnel, tunnel.url())

  async def test_proxy_proto(self):
    class ProxyHandler(socketserver.StreamRequestHandler):
      read_value = None
      def handle(self):
        ProxyHandler.read_value = self.rfile.read(10)

    tcp_server = socketserver.TCPServer(('localhost', 0), ProxyHandler)
    thread = threading.Thread(target=tcp_server.serve_forever, daemon=True)
    thread.start()
    session = await make_session()
    tunnel = (await session.http_endpoint()
      .proxy_proto("1")
      .listen())

    addr = tcp_server.server_address
    tcp_server.listen_to = "{}:{}".format(addr[0], addr[1])
    tunnel.forward_tcp(tcp_server.listen_to)
    try:
      resp = requests.get(tunnel.url(), timeout=1)
    except requests.exceptions.ReadTimeout as err:
      pass
    self.assertEqual(b'PROXY TCP4', ProxyHandler.read_value)
    await shutdown(tunnel, tcp_server)

  async def test_ip_restriction_http(self):
    http_server, session = await make_http_and_session()
    error = await self.ip_restriction(http_server, session.http_endpoint())
    self.assertEqual(403, error.status_code)

  async def test_ip_restriction_tcp(self):
    http_server, session = await make_http_and_session()
    error = await self.ip_restriction(http_server, session.tcp_endpoint())
    self.assertIsInstance(error, requests.exceptions.ConnectionError)

  async def ip_restriction(self, http_server, tunnel_builder):
    tunnel = (await tunnel_builder
      .allow_cidr("127.0.0.1/32")
      .deny_cidr("0.0.0.0/0")
      .listen())

    tunnel.forward_tcp(http_server.listen_to)
    error = None
    try:
      error = requests.get(tunnel.url().replace('tcp:','http:'))
    except requests.exceptions.ConnectionError as err:
      error = err
    await shutdown(tunnel, http_server)
    return error;

  async def test_websocket_conversion(self):
    http_server, session = await make_http_and_session()
    tunnel = (await session.http_endpoint()
      .websocket_tcp_conversion()
      .listen())

    tunnel.forward_tcp(http_server.listen_to)

    response = requests.get(tunnel.url())
    self.assertEqual(400, response.status_code)
    # ERR_NGROK_3206: Expected a websocket request with a "Connection: upgrade" header
    # but did not receive one.
    self.assertEqual("ERR_NGROK_3206", response.headers["ngrok-error-code"]);

    await shutdown(tunnel, http_server)

  async def test_tcp_tunnel(self):
    http_server, session = await make_http_and_session()
    tunnel = (await session.tcp_endpoint()
      .forwards_to("tcp forwards to")
      .metadata("tcp metadata")
      .listen())

    self.assertEqual("tcp forwards to", tunnel.forwards_to())
    self.assertEqual("tcp metadata", tunnel.metadata())

    await self.forward_validate_shutdown(http_server, tunnel, tunnel.url().replace("tcp:", "http:"))

  async def test_tls_tunnel(self):
    http_server, session = await make_http_and_session()
    tunnel = (await session.tls_endpoint()
      .forwards_to("tcp forwards to")
      .metadata("tcp metadata")
      .listen())

    self.assertEqual("tcp forwards to", tunnel.forwards_to())
    self.assertEqual("tcp metadata", tunnel.metadata())

    tunnel.forward_tcp(http_server.listen_to)

    error = None
    try:
      response = requests.get(tunnel.url().replace("tls:", "https:"))
    except requests.exceptions.SSLError as err:
      error = err
    self.assertIsInstance(error, requests.exceptions.SSLError)

    await shutdown(tunnel, http_server)


  async def test_standard_listen(self):
    http_server = make_http()
    tunnel1 = await ngrok.listen()
    tunnel2 = await ngrok.listen(tunnel=tunnel1)
    self.assertEqual(tunnel1.url(), tunnel2.url());
    tunnel3 = await ngrok.listen(http_server, tunnel2)
    self.assertEqual(tunnel2.url(), tunnel3.url());
    await self.forward_validate_shutdown(http_server, tunnel3, tunnel3.url())

  async def test_standard_listen_server(self):
    http_server = make_http()
    tunnel = await ngrok.listen(http_server)
    await self.forward_validate_shutdown(http_server, tunnel, tunnel.url())

  def test_aiohttp_listen(self):
    async def hello(request):
      return web.Response(text=expected);

    async def shutdown(request):
      # workaround of not having a close after run_app
      raise GracefulExit();

    loop = asyncio.new_event_loop();
    app = web.Application();
    app.add_routes([web.get('/', hello)]);
    app.add_routes([web.get('/shutdown', shutdown)]);
    tunnel = ngrok.listen();

    async def validate():
      # have to use an async http client
      async with ClientSession() as client:
        # test that tunnel to server works
        async with client.get(tunnel.url()) as response:
          self.assertEqual(200, response.status);
          self.assertEqual(expected, await response.text());
        # shutdown server
        await client.get("{}/shutdown".format(tunnel.url()))
      # shutdown tunnel
      await tunnel.close();

    loop.create_task(validate())
    try:
      web.run_app(app, sock=tunnel, loop=loop)
    except GracefulExit:
      pass

  def test_pipe_name(self):
    pipe_name = ngrok.pipe_name();
    self.assertTrue('tun' in pipe_name);

  async def test_werkzeug_develop(self):
    tunnel = await ngrok.werkzeug_develop();
    self.assertIsNotNone(tunnel.fd);
    self.assertEqual(os.environ["WERKZEUG_SERVER_FD"], str(tunnel.fd));
    self.assertEqual(os.environ["WERKZEUG_RUN_MAIN"], "true");
    await tunnel.close();

  async def test_default(self):
    session = await make_session();
    tunnel = await ngrok.default(session);
    self.assertTrue('http' in tunnel.url());
    await session.close();

  async def test_getsockname(self):
    session = await make_session();
    sockname = await ngrok.getsockname(session);
    self.assertTrue('tun' in sockname);
    await session.close();

  async def test_fd(self):
    session = await make_session();
    fd = await ngrok.fd(session);
    self.assertIsNotNone(fd);
    self.assertTrue(fd > 0);
    await session.close();

  async def test_tcp_multipass(self):
    http_server, session1 = await make_http_and_session()
    session2 = await make_session()
    tunnel1 = await session1.http_endpoint().listen()
    tunnel2 = await session1.http_endpoint().listen()
    tunnel3 = await session2.http_endpoint().listen()
    tunnel4 = await session2.tcp_endpoint().listen()

    tunnel1.forward_tcp(http_server.listen_to)
    tunnel2.forward_tcp(http_server.listen_to)
    tunnel3.forward_tcp(http_server.listen_to)
    tunnel4.forward_tcp(http_server.listen_to)

    await self.validate_http_request(tunnel1.url())
    await self.validate_http_request(tunnel2.url())
    await self.validate_http_request(tunnel3.url())
    await self.validate_http_request(tunnel4.url().replace("tcp:", "http:"))
    await shutdown(tunnel1, http_server)
    await tunnel2.close()
    await tunnel3.close()
    await tunnel4.close()

  async def test_pipe_multipass(self):
    http_server, session1 = await make_http_and_session(True)
    session2 = await make_session()
    tunnel1 = await session1.http_endpoint().listen()
    tunnel2 = await session1.http_endpoint().listen()
    tunnel3 = await session2.http_endpoint().listen()
    tunnel4 = await session2.tcp_endpoint().listen()

    tunnel1.forward_pipe(http_server.listen_to)
    tunnel2.forward_pipe(http_server.listen_to)
    tunnel3.forward_pipe(http_server.listen_to)
    tunnel4.forward_pipe(http_server.listen_to)

    await self.validate_http_request(tunnel1.url())
    await self.validate_http_request(tunnel2.url())
    await self.validate_http_request(tunnel3.url())
    await self.validate_http_request(tunnel4.url().replace("tcp:", "http:"))
    await shutdown(tunnel1, http_server)
    await tunnel2.close()
    await tunnel3.close()
    await tunnel4.close()

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
    builder = ngrok.NgrokSessionBuilder()
    (builder
      .handle_heartbeat(on_heartbeat)
      .handle_disconnection(on_disconn))
    await builder.connect()
    self.assertTrue(test_latency > 0)
    self.assertEqual(None, disconn_addr)

  async def test_ca_cert(self):
    error = None
    cert = None
    with open("examples/domain.crt", "r") as crt:
        cert = bytearray(crt.read().encode())
    try:
      await ngrok.NgrokSessionBuilder().ca_cert(cert).connect()
    except ValueError as err:
      error = err
    self.assertIsInstance(error, ValueError)
    self.assertTrue("cert" in f"{error}")

if __name__ == '__main__':
  unittest.main()
