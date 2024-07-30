import ngrok
import os
import requests
import unittest
import test
from test import retry_request


def shutdown(url, http_server):
    ngrok.disconnect(url)
    http_server.shutdown()
    http_server.server_close()


class TestNgrokConnect(unittest.IsolatedAsyncioTestCase):
    def validate_http_request(self, url, requests_config=dict()):
        response = retry_request().get(url, **requests_config)
        self.assertEqual(200, response.status_code)
        self.assertEqual(test.expected, response.text)
        return response

    def validate_shutdown(self, http_server, listener, url, requests_config=dict()):
        response = self.validate_http_request(url, requests_config)
        shutdown(url, http_server)
        return response

    async def test_connect_https_listener_async(self):
        http_server = test.make_http()
        listener = await ngrok.connect(
            http_server.listen_to,
            authtoken_from_env=True,
            forwards_to="http forwards to",
            metadata="http metadata",
        )

        self.assertIsNotNone(listener.id())
        self.assertIsNotNone(listener.url())
        self.assertTrue(listener.url().startswith("https://"))
        self.assertEqual("http forwards to", listener.forwards_to())
        self.assertEqual("http metadata", listener.metadata())
        self.validate_shutdown(http_server, listener, listener.url())

    async def test_https_listener_async(self):
        http_server = test.make_http()
        listener = await ngrok.forward(
            http_server.listen_to,
            authtoken_from_env=True,
            forwards_to="http forwards to",
            metadata="http metadata",
        )

        self.assertIsNotNone(listener.id())
        self.assertIsNotNone(listener.url())
        self.assertTrue(listener.url().startswith("https://"))
        self.assertEqual("http forwards to", listener.forwards_to())
        self.assertEqual("http metadata", listener.metadata())
        self.validate_shutdown(http_server, listener, listener.url())

    async def test_https_listener_with_config(self):
        http_server = test.make_http()
        listener = await ngrok.forward(
            http_server.listen_to,
            authtoken_from_env=True,
            forwards_to="http forwards to",
            metadata="http metadata",
        )

        self.assertIsNotNone(listener.id())
        self.assertIsNotNone(listener.url())
        self.assertTrue(listener.url().startswith("https://"))
        self.assertEqual("http forwards to", listener.forwards_to())
        self.assertEqual("http metadata", listener.metadata())
        response = self.validate_shutdown(http_server, listener, listener.url())
        print(response.headers)
        print(response.headers)
        print(response.headers)

    def test_https_listener(self):
        http_server = test.make_http()
        ngrok.set_auth_token(os.environ["NGROK_AUTHTOKEN"])
        listener = ngrok.forward(
            http_server.listen_to,
            forwards_to="http forwards to",
            metadata="http metadata",
        )

        self.assertIsNotNone(listener.id())
        self.assertIsNotNone(listener.url())
        self.assertTrue(listener.url().startswith("https://"))
        self.assertEqual("http forwards to", listener.forwards_to())
        self.assertEqual("http metadata", listener.metadata())
        self.assertTrue(len(ngrok.get_listeners()) >= 1)

        self.validate_shutdown(http_server, listener, listener.url())

    def test_tls_backend(self):
        ngrok.set_auth_token(os.environ["NGROK_AUTHTOKEN"])
        listener = ngrok.forward("https://dashboard.ngrok.com")

        response = retry_request().get(listener.url())
        self.assertEqual(421, response.status_code)
        self.assertTrue("different Host" in response.text)
        ngrok.disconnect(listener.url())

    def test_tls_backend_no_verify(self):
        ngrok.set_auth_token(os.environ["NGROK_AUTHTOKEN"])
        listener = ngrok.forward(
            "https://dashboard.ngrok.com", verify_upstream_tls=False
        )

        response = retry_request().get(listener.url())
        self.assertEqual(421, response.status_code)
        self.assertTrue("different Host" in response.text)
        ngrok.disconnect(listener.url())

    def test_forward_number(self):
        http_server = test.make_http()
        listener = ngrok.forward(
            int(http_server.listen_to.split(":")[1]), authtoken_from_env=True
        )
        self.validate_shutdown(http_server, listener, listener.url())

    def test_forward_addr_protocol(self):
        http_server = test.make_http()
        listener = ngrok.forward(
            f"http://{http_server.listen_to}",  # http:// should be ignored
            authtoken_from_env=True,
            authtoken=None,  # None's should be ignored
            basic_auth=None,
            circuit_breaker=None,
            mutual_tls_cas=None,
        )
        self.validate_shutdown(http_server, listener, listener.url())

    def test_forward_dots(self):
        http_server = test.make_http()
        options = {"authtoken.from.env": True}
        listener = ngrok.forward(http_server.listen_to, **options)
        self.validate_shutdown(http_server, listener, listener.url())

    def test_forward_vectorize(self):
        http_server = test.make_http()
        listener = ngrok.forward(
            http_server.listen_to,
            authtoken_from_env=True,
            basic_auth="ngrok:online1line",
            allow_cidr="0.0.0.0/0",
            deny_cidr="10.1.1.1/32",
            request_header_remove="X-Req-Nope2",
            response_header_remove="X-Res-Nope2",
            request_header_add="X-Req-Yup2:true2",
            response_header_add="X-Res-Yup2:true2",
            schemes="HTTPS",
        )
        config = {"auth": ("ngrok", "online1line")}
        response = self.validate_shutdown(http_server, listener, listener.url(), config)
        self.assertEqual("true2", response.headers["x-res-yup2"])

    async def test_tcp_listener(self):
        http_server = test.make_http()
        listener = await ngrok.forward(
            http_server.listen_to,
            authtoken_from_env=True,
            forwards_to="tcp forwards to",
            metadata="tcp metadata",
            proto="tcp",
        )

        self.assertEqual("tcp forwards to", listener.forwards_to())
        self.assertEqual("tcp metadata", listener.metadata())

        self.validate_shutdown(
            http_server, listener, listener.url().replace("tcp:", "http:")
        )

    async def test_tls_listener(self):
        http_server = test.make_http()
        listener = await ngrok.forward(
            http_server.listen_to,
            authtoken_from_env=True,
            forwards_to="tls forwards to",
            metadata="tls metadata",
            proto="tls",
        )

        self.assertEqual("tls forwards to", listener.forwards_to())
        self.assertEqual("tls metadata", listener.metadata())

        listener.forward(http_server.listen_to)

        error = None
        try:
            requests.get(listener.url().replace("tls:", "https:"))
        except requests.exceptions.SSLError as err:
            error = err
        self.assertIsInstance(error, requests.exceptions.SSLError)

        shutdown(listener.url(), http_server)

    async def test_connect_policy(self):
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

        http_server = test.make_http()
        listener = await ngrok.connect(
            http_server.listen_to,
            authtoken_from_env=True,
            policy=policy,
        )
        response = retry_request().get(listener.url())
        self.assertEqual("added-header-value", response.headers["added-header"])
        self.validate_shutdown(http_server, listener, listener.url())

    async def test_connect_traffic_policy(self):
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

        http_server = test.make_http()
        listener = await ngrok.connect(
            http_server.listen_to,
            authtoken_from_env=True,
            traffic_policy=traffic_policy,
        )
        response = retry_request().get(listener.url())
        self.assertEqual("added-header-value", response.headers["added-header"])
        self.validate_shutdown(http_server, listener, listener.url())

    async def test_invalid_connect_policy(self):
        http_server = test.make_http()
        try:
            listener = await ngrok.connect(
                http_server.listen_to,
                authtoken_from_env=True,
                policy="{{",
            )
        except ValueError as err:
            error = err
        self.assertIsInstance(error, ValueError)
        self.assertTrue("provided is invalid" in f"{error}")
        shutdown(None, http_server)

    async def test_invalid_connect_traffic_policy(self):
        http_server = test.make_http()
        try:
            listener = await ngrok.connect(
                http_server.listen_to,
                authtoken_from_env=True,
                traffic_policy="{{",
            )
        except ValueError as err:
            error = err
        self.assertIsInstance(error, ValueError)
        self.assertTrue("provided is invalid" in f"{error}")
        shutdown(None, http_server)

    def test_root_cas(self):
        http_server = test.make_http()
        error = None
        # tls error connecting to marketing site
        try:
            listener = ngrok.connect(
                http_server.listen_to,
                authtoken_from_env=True,
                force_new_session=True,
                root_cas="trusted",
                server_addr="ngrok.com:443",
            )
        except ValueError as err:
            error = err
        self.assertIsInstance(error, ValueError)
        self.assertTrue("tls handshake" in f"{error}", error)

        # non-tls error connecting to marketing site with "host" root_cas
        try:
            listener = ngrok.connect(
                http_server.listen_to,
                authtoken_from_env=True,
                force_new_session=True,
                root_cas="host",
                server_addr="ngrok.com:443",
            )
        except ValueError as err:
            error = err
        self.assertIsInstance(error, ValueError)
        self.assertFalse("tls handshake" in f"{error}", error)


if __name__ == "__main__":
    unittest.main()
