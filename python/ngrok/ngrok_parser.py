import argparse


def get_pass_through_args():
    return {"host", "port", "uds", "fd", "config", "bind"}


def get_parser():
    # argument parsing
    parser = argparse.ArgumentParser(
        prog="ngrok-asgi",
        description="ASGI wrapper for ngrok",
        epilog="Online in One Line",
    )
    parser.add_argument(
        "command", choices=["gunicorn", "uvicorn"], help="gunicorn or uvicorn"
    )
    # ngrok session options
    parser.add_argument(
        "--authtoken",
        help="Ngrok authtoken, otherwise uses NGROK_AUTH_TOKEN environment variable",
    )
    parser.add_argument(
        "--session-metadata",
        help="Configures the opaque, machine-readable metadata string for this session.",
    )

    # ngrok tunnel options
    parser.add_argument(
        "--allow-cidr",
        action="append",
        help="Restriction placed on the origin of incoming connections to the edge to only allow these CIDR ranges. Call multiple times to add additional CIDR ranges.",
    )
    parser.add_argument(
        "--basic-auth",
        nargs=2,
        metavar=("username", "password"),
        help="Credentials for basic authentication.",
    )
    parser.add_argument(
        "--circuit-breaker",
        help="Reject requests when 5XX responses exceed this ratio. Disabled when 0.",
    )
    parser.add_argument(
        "--compression",
        action="store_true",
        help="Enable gzip compression for HTTP responses.",
    )
    parser.add_argument(
        "--deny-cidr",
        action="append",
        help="Restriction placed on the origin of incoming connections to the edge to deny these CIDR ranges. Call multiple times to add additional CIDR ranges.",
    )
    parser.add_argument("--domain", help="The domain to request for this edge.")
    parser.add_argument(
        "--forwards-to",
        help="Tunnel backend metadata. Viewable via the dashboard and API, but has no bearing on tunnel behavior.",
    )
    parser.add_argument(
        "--metadata", help="Tunnel-specific opaque metadata. Viewable via the API."
    )
    parser.add_argument(
        "--mutual-tlsca",
        help="Filename of certificates to use for client authentication at the ngrok edge.",
    )
    parser.add_argument("--oauth-provider", help="OAuth provider configuration.")
    parser.add_argument(
        "--oidc",
        nargs=3,
        metavar=("issuer-url", "client-id", "client-secret"),
        help="OIDC configuration.",
    )
    parser.add_argument(
        "--allow-emails", action="append", help="OAuth/OIDC configuration."
    )
    parser.add_argument(
        "--allow-domains", action="append", help="OAuth/OIDC configuration."
    )
    parser.add_argument("--scopes", action="append", help="OAuth/OIDC configuration.")
    parser.add_argument(
        "--proxy-proto",
        choices=["", "1", "2"],
        help="The version of PROXY protocol to use with this tunnel “1”, “2”, or “” if not using.",
    )
    parser.add_argument(
        "--remove-request-header",
        action="append",
        help="Removes a header from requests to this edge. Call multiple times to add additional values.",
    )
    parser.add_argument(
        "--remove-response-header",
        action="append",
        help="Removes a header from responses from this edge Call multiple times to add additional values..",
    )
    parser.add_argument(
        "--request-header",
        action="append",
        nargs=2,
        metavar=("header", "value"),
        help="Adds a header to all requests to this edge. Call multiple times to add additional values.",
    )
    parser.add_argument(
        "--response-header",
        action="append",
        nargs=2,
        metavar=("header", "value"),
        help="Adds a header to all responses coming from this edge. Call multiple times to add additional values.",
    )
    parser.add_argument(
        "--scheme",
        choices=["HTTPS", "HTTP"],
        help="The scheme that this edge should use. Defaults to HTTPS.",
    )
    parser.add_argument(
        "--webhook-verification",
        nargs=2,
        metavar=("provider", "secret"),
        help="WebhookVerification configuration.",
    )
    parser.add_argument(
        "--websocket-tcp-conversion",
        action="store_true",
        help="Convert incoming websocket connections to TCP-like streams.",
    )
    # uvicorn options
    parser.add_argument("--host", help="Hostname or IP address")
    parser.add_argument("--port", help="Port number", type=int)
    parser.add_argument("--uds", help="Unix domain socket")
    parser.add_argument("--fd", help="File descriptor")
    # gunicorn options
    parser.add_argument("--config", "-c", help="Config file not supported")
    parser.add_argument(
        "--bind",
        "-b",
        help="Specify a server socket to bind. Server sockets can be any of $(HOST), $(HOST):$(PORT), "
        + "fd://$(FD), or unix:$(PATH). An IP is a valid $(HOST).",
    )
    return parser
