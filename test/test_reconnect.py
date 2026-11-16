"""SessionBuilder's reconnect options, checked without connecting."""

import re
import unittest

import ngrok


class TestReconnectBackoff(unittest.TestCase):
    # The range rules are ngrok-rust's; the binding passes its error through,
    # after the call and its arguments.
    def test_rejects_ranges_ngrok_rust_refuses(self):
        cases = [
            ("a start of 0 ms", 0, 1000, "the start must be at least 1 ms"),
            ("a start above the cap", 2000, 1000, "the start must not be above the cap"),
            ("a cap above one day", 1, 86_400_001, "the cap must be at most one day"),
            # Above 2**32, so a 32-bit argument would wrap it to 50 ms and
            # accept it.
            ("a cap of 2**32 + 50 ms", 1, 2**32 + 50, "the cap must be at most one day"),
        ]
        for name, initial_ms, max_ms, rule in cases:
            with self.subTest(name):
                with self.assertRaisesRegex(
                    ValueError,
                    rf"^reconnect_backoff\(initial_ms={initial_ms}, max_ms={max_ms}\): "
                    rf".*{re.escape(rule)}$",
                ):
                    ngrok.SessionBuilder().reconnect_backoff(initial_ms, max_ms)

    # An int a u64 cannot hold is refused before ngrok-rust sees it, naming
    # the argument.
    def test_rejects_ints_out_of_u64_range(self):
        cases = [
            ("a negative start", -1, 1000, "initial_ms", "-1"),
            ("a negative cap", 1, -1, "max_ms", "-1"),
            ("a start of 2**64", 2**64, 1000, "initial_ms", "18446744073709551616"),
        ]
        for name, initial_ms, max_ms, argument, shown in cases:
            with self.subTest(name):
                with self.assertRaisesRegex(
                    ValueError,
                    rf"^{argument} must be from 0 to 18446744073709551615, got {shown}$",
                ):
                    ngrok.SessionBuilder().reconnect_backoff(initial_ms, max_ms)

    def test_rejects_what_is_not_an_int(self):
        cases = [
            ("a fractional start", 0.5, 1000, "initial_ms"),
            ("a fractional cap", 1, 1000.5, "max_ms"),
            ("a string cap", 1, "1000", "max_ms"),
        ]
        for name, initial_ms, max_ms, argument in cases:
            with self.subTest(name):
                with self.assertRaisesRegex(TypeError, rf"^argument '{argument}': "):
                    ngrok.SessionBuilder().reconnect_backoff(initial_ms, max_ms)

    def test_accepts_1_ms_to_one_day(self):
        builder = ngrok.SessionBuilder()
        self.assertIs(builder.reconnect_backoff(1, 86_400_000), builder)

    def test_a_refused_call_leaves_the_builder_usable(self):
        builder = ngrok.SessionBuilder()
        with self.assertRaises(ValueError):
            builder.reconnect_backoff(2000, 1000)
        self.assertIs(builder.reconnect_backoff(100, 5000), builder)


class TestReconnectJitter(unittest.TestCase):
    def test_returns_the_builder(self):
        builder = ngrok.SessionBuilder()
        self.assertIs(builder.reconnect_jitter(False), builder)
        self.assertIs(builder.reconnect_jitter(True), builder)


class TestReconnectMaxAttempts(unittest.TestCase):
    def test_rejects_ints_out_of_range(self):
        for attempts in [0, -1, 2**32]:
            with self.subTest(attempts):
                with self.assertRaisesRegex(
                    ValueError,
                    rf"^attempts must be from 1 to 4294967295, got {attempts}$",
                ):
                    ngrok.SessionBuilder().reconnect_max_attempts(attempts)

    def test_rejects_what_is_not_an_int(self):
        with self.assertRaisesRegex(TypeError, r"^argument 'attempts': "):
            ngrok.SessionBuilder().reconnect_max_attempts(2.5)

    def test_accepts_1_to_4294967295(self):
        builder = ngrok.SessionBuilder()
        self.assertIs(builder.reconnect_max_attempts(1), builder)
        self.assertIs(builder.reconnect_max_attempts(2**32 - 1), builder)


if __name__ == "__main__":
    unittest.main()
