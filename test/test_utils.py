import unittest

import ngrok

import ngrok.utils
import socket
import subprocess
from urllib.parse import urlparse


class TestNgrokUtils(unittest.TestCase):
    def test_scoped_endpoint(self):
        with ngrok.utils.scoped_endpoint(
            11222, authtoken_from_env=True, proto="tcp"
        ) as endpoint:
            endpoint: ngrok.Listener
            nc_recv = subprocess.Popen(
                ["nc", "-l", "11222"], stdout=subprocess.PIPE, stderr=subprocess.PIPE
            )
            url = urlparse(endpoint.url())

            s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            try:
                s.connect((url.hostname, url.port))
                s.send("doot doot".encode("utf-8"))
            finally:
                s.close()
            stdout, stderr = nc_recv.communicate()
            self.assertEqual("doot doot", stdout.decode("utf-8").strip())

    def test_decorator_scoped_endpoint(self):
        @ngrok.utils.scoped_endpoint(11222, authtoken_from_env=True)
        def foo():
            # tunnel lives for the lifetime of the function
            # mostly only useful if you don't need the endpoint object to get its URL. reserved domains etc
            pass

        foo()


if __name__ == "__main__":
    unittest.main()
