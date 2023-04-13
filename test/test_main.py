from ngrok import __main__, ngrok_parser
import os
import unittest


class TestNgrokMain(unittest.IsolatedAsyncioTestCase):
    def parser_test(self, args_str, pipe_expect, tcp_expect):
        parser = ngrok_parser.get_parser()
        args, unknown = parser.parse_known_args(args_str.split())
        self.assertEqual(pipe_expect, __main__.get_pipe_string(args))
        self.assertEqual(tcp_expect, __main__.get_tcp_string(args))

    def test_uvicorn(self):
        self.parser_test("uvicorn", None, None)
        self.parser_test("uvicorn --host 0.0.0.0", None, "0.0.0.0:8000")
        self.parser_test("uvicorn --host 127.0.0.1", None, "127.0.0.1:8000")
        self.parser_test("uvicorn --host 192.168.2.1", None, "192.168.2.1:8000")
        self.parser_test("uvicorn --host localhost", None, "localhost:8000")
        self.parser_test("uvicorn --port 1234", None, "localhost:1234")
        self.parser_test("uvicorn --host localhost --port 1234", None, "localhost:1234")
        # ipv6
        self.parser_test("uvicorn --host ::", None, ":::8000")
        self.parser_test("uvicorn --host :: --port 1234", None, ":::1234")
        # pipe
        self.parser_test("uvicorn --uds /uvicorn.sock", "/uvicorn.sock", None)
        self.parser_test("uvicorn --fd 42", None, None)

    def test_gunicorn(self):
        self.parser_test("gunicorn", None, None)
        self.parser_test("gunicorn --bind 0.0.0.0", None, "0.0.0.0:8000")
        self.parser_test("gunicorn --bind 127.0.0.1", None, "127.0.0.1:8000")
        self.parser_test("gunicorn --bind 192.168.2.1", None, "192.168.2.1:8000")
        self.parser_test("gunicorn --bind localhost", None, "localhost:8000")
        self.parser_test("gunicorn --bind :1234", None, "localhost:1234")
        self.parser_test("gunicorn --bind localhost:1234", None, "localhost:1234")
        os.environ["PORT"] = "1234"
        self.parser_test("gunicorn --bind localhost", None, "localhost:1234")
        del os.environ["PORT"]
        # ipv6
        self.parser_test("gunicorn --bind [::]", None, "[::]:8000")
        self.parser_test("gunicorn --bind [::]:1234", None, "[::]:1234")
        # pipe
        self.parser_test("gunicorn --bind unix:/uvicorn.sock", "/uvicorn.sock", None)
        self.parser_test("gunicorn --bind fd://42", None, None)
