import requests
import socket
import threading
import unittest
from test import tls_passthrough_request

OFFLINE = "ERR_NGROK_3200"


def response(status_code, ngrok_error_code=None):
    r = requests.Response()
    r.status_code = status_code
    if ngrok_error_code:
        r.headers["ngrok-error-code"] = ngrok_error_code
    return r


class FakeEdge:
    """Stands in for ngrok's edge, which a hermetic test cannot reach.

    Answers each GET with the next step: a requests.Response, or "handshake"
    to raise the SSLError that TLS passthrough to a plain-HTTP server gives.
    """

    def __init__(self, *steps):
        self.steps = list(steps)
        self.requests = 0

    def get(self, url, timeout):
        self.requests += 1
        step = self.steps.pop(0)
        if step == "handshake":
            raise requests.exceptions.SSLError("wrong version number")
        return step


def no_sleep(seconds):
    pass


class TestTlsPassthroughRequest(unittest.TestCase):
    def request(self, edge, timeout=10):
        return tls_passthrough_request(
            "https://example.ngrok.app", timeout=timeout, get=edge.get, sleep=no_sleep
        )

    def test_offline_answers_are_retried_until_the_handshake_fails(self):
        edge = FakeEdge(response(404, OFFLINE), response(404, OFFLINE), "handshake")
        result = self.request(edge)
        self.assertIsInstance(result, requests.exceptions.SSLError)
        self.assertEqual(3, edge.requests)

    def test_handshake_failure_returns_at_once(self):
        edge = FakeEdge("handshake")
        self.assertIsInstance(self.request(edge), requests.exceptions.SSLError)
        self.assertEqual(1, edge.requests)

    def test_404_without_the_offline_code_is_not_retried(self):
        edge = FakeEdge(response(404), "handshake")
        self.assertEqual(404, self.request(edge).status_code)
        self.assertEqual(1, edge.requests)

    def test_terminated_tls_is_returned_not_retried(self):
        edge = FakeEdge(response(200), "handshake")
        self.assertEqual(200, self.request(edge).status_code)
        self.assertEqual(1, edge.requests)

    def test_offline_past_the_deadline_returns_the_404(self):
        edge = FakeEdge(response(404, OFFLINE), "handshake")
        result = self.request(edge, timeout=-1)
        self.assertEqual(404, result.status_code)
        self.assertEqual(1, edge.requests)


class TestTlsPassthroughRequestStall(unittest.TestCase):
    def test_a_stalled_handshake_times_out_instead_of_hanging(self):
        # A local server that accepts and never answers stands in for an edge
        # that stalls mid-handshake; the real requests.get talks to it.
        with socket.socket() as server:
            server.bind(("127.0.0.1", 0))
            server.listen()
            url = f"https://127.0.0.1:{server.getsockname()[1]}"
            raised = []

            def call():
                try:
                    tls_passthrough_request(url, timeout=1)
                except requests.exceptions.Timeout as err:
                    raised.append(err)

            caller = threading.Thread(target=call, daemon=True)
            caller.start()
            caller.join(10)  # bounds the test if the request never returns
            self.assertFalse(caller.is_alive(), "the request was still hanging")
            self.assertIsInstance(raised[0], requests.exceptions.Timeout)
