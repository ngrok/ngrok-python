import unittest

import ngrok

import ngrok_extra.utils as n
import socket
import subprocess
from urllib.parse import urlparse


class TestNgrokUtils(unittest.TestCase):
    def test_scoped_endpoint(self):
        with n.scoped_endpoint(9000, authtoken_from_env=True, proto="tcp") as endpoint:
            endpoint: ngrok.Listener
            nc = subprocess.Popen(
                ["nc", "-l", "9000"], stdout=subprocess.PIPE, stderr=subprocess.PIPE
            )
            url = urlparse(endpoint.url())
            subprocess.check_call(
                ["bash", "-c", f"echo 'doot doot' | nc {url.hostname} {url.port}"]
            )
            stdout, stderr = nc.communicate()
            self.assertEqual("doot doot", stdout.decode("utf-8").strip())

    def test_decorator_scoped_endpoint(self):
        @n.scoped_endpoint(9000, authtoken_from_env=True)
        def foo():
            print("tunnel lives for the lifetime of the function")
            print(
                "mostly only useful if you don't need the endpoint object to get its URL. reserved domains etc?"
            )

        foo()


if __name__ == "__main__":
    unittest.main()
