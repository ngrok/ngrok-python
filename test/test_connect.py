import ngrok
import os
import requests
import unittest
import test


def shutdown(url, http_server):
    ngrok.disconnect(url)
    http_server.shutdown()
    http_server.server_close()


class TestNgrok(unittest.IsolatedAsyncioTestCase):
    def validate_http_request(self, url, requests_config=dict()):
        response = requests.get(url, **requests_config)
        self.assertEqual(200, response.status_code)
        self.assertEqual(test.expected, response.text)
        return response

    def validate_shutdown(self, http_server, tunnel, url, requests_config=dict()):
        response = self.validate_http_request(url, requests_config)
        shutdown(url, http_server)
        return response

    async def test_https_tunnel_async(self):
        http_server = test.make_http()
        tunnel = await ngrok.connect(
            http_server.listen_to,
            authtoken_from_env=True,
            forwards_to="http forwards to",
            metadata="http metadata",
        )

        self.assertIsNotNone(tunnel.id())
        self.assertIsNotNone(tunnel.url())
        self.assertTrue(tunnel.url().startswith("https://"))
        self.assertEqual("http forwards to", tunnel.forwards_to())
        self.assertEqual("http metadata", tunnel.metadata())
        self.validate_shutdown(http_server, tunnel, tunnel.url())

    def test_https_tunnel(self):
        http_server = test.make_http()
        ngrok.set_auth_token(os.environ["NGROK_AUTHTOKEN"])
        tunnel = ngrok.connect(
            http_server.listen_to,
            forwards_to="http forwards to",
            metadata="http metadata",
        )

        self.assertIsNotNone(tunnel.id())
        self.assertIsNotNone(tunnel.url())
        self.assertTrue(tunnel.url().startswith("https://"))
        self.assertEqual("http forwards to", tunnel.forwards_to())
        self.assertEqual("http metadata", tunnel.metadata())
        self.assertTrue(len(ngrok.get_tunnels()) >= 1)

        self.validate_shutdown(http_server, tunnel, tunnel.url())

    def test_connect_number(self):
        http_server = test.make_http()
        tunnel = ngrok.connect(
            int(http_server.listen_to.split(":")[1]), authtoken_from_env=True
        )
        self.validate_shutdown(http_server, tunnel, tunnel.url())

    def test_connect_addr_protocol(self):
        http_server = test.make_http()
        tunnel = ngrok.connect(
            f"http://{http_server.listen_to}",  # http:// should be ignored
            authtoken_from_env=True,
            authtoken=None,  # None's should be ignored
            basic_auth=None,
            circuit_breaker=None,
            mutual_tls_cas=None,
        )
        self.validate_shutdown(http_server, tunnel, tunnel.url())

    def test_connect_dots(self):
        http_server = test.make_http()
        options = {"authtoken.from.env": True}
        tunnel = ngrok.connect(http_server.listen_to, **options)
        self.validate_shutdown(http_server, tunnel, tunnel.url())

    def test_connect_vectorize(self):
        http_server = test.make_http()
        tunnel = ngrok.connect(
            http_server.listen_to,
            authtoken_from_env=True,
            basic_auth="ngrok:online1line",
            ip_restriction_allow_cidrs="0.0.0.0/0",
            ip_restriction_deny_cidrs="10.1.1.1/32",
            request_header_remove="X-Req-Nope2",
            response_header_remove="X-Res-Nope2",
            request_header_add="X-Req-Yup2:true2",
            response_header_add="X-Res-Yup2:true2",
            schemes="HTTPS",
        )
        config = {"auth": ("ngrok", "online1line")}
        response = self.validate_shutdown(http_server, tunnel, tunnel.url(), config)
        self.assertEqual("true2", response.headers["x-res-yup2"])

    async def test_tcp_tunnel(self):
        http_server = test.make_http()
        tunnel = await ngrok.connect(
            http_server.listen_to,
            authtoken_from_env=True,
            forwards_to="tcp forwards to",
            metadata="tcp metadata",
            proto="tcp",
        )

        self.assertEqual("tcp forwards to", tunnel.forwards_to())
        self.assertEqual("tcp metadata", tunnel.metadata())

        self.validate_shutdown(
            http_server, tunnel, tunnel.url().replace("tcp:", "http:")
        )

    async def test_tls_tunnel(self):
        http_server = test.make_http()
        tunnel = await ngrok.connect(
            http_server.listen_to,
            authtoken_from_env=True,
            forwards_to="tls forwards to",
            metadata="tls metadata",
            proto="tls",
        )

        self.assertEqual("tls forwards to", tunnel.forwards_to())
        self.assertEqual("tls metadata", tunnel.metadata())

        tunnel.forward_tcp(http_server.listen_to)

        error = None
        try:
            response = requests.get(tunnel.url().replace("tls:", "https:"))
        except requests.exceptions.SSLError as err:
            error = err
        self.assertIsInstance(error, requests.exceptions.SSLError)

        shutdown(tunnel.url(), http_server)


if __name__ == "__main__":
    unittest.main()
