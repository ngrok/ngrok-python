import asyncio
import logging
import ngrok
from ngrok import ngrok_parser
import os
import re
import shlex
import sys

DEFAULT_HOST = "localhost"
DEFAULT_PORT = "8000"


def configure_session(args):
    builder = ngrok.NgrokSessionBuilder()
    if args.authtoken:
        builder.authtoken(args.authtoken)
    else:
        builder.authtoken_from_env()

    if args.session_metadata:
        builder.metadata(args.session_metadata)
    return builder


def configure_tunnel(session, args):
    builder = session.http_endpoint()
    if args.allow_cidr:
        for cidr in args.allow_cidr:
            builder.allow_cidr(cidr)
    if args.basic_auth:
        builder.basic_auth(args.basic_auth[0], args.basic_auth[1])
    if args.circuit_breaker:
        builder.circuit_breaker(float(args.circuit_breaker))
    if args.compression:
        builder.compression()
    if args.deny_cidr:
        for cidr in args.deny_cidr:
            builder.deny_cidr(cidr)
    if args.domain:
        builder.domain(args.domain)
    if args.forwards_to:
        builder.forwards_to(args.forwards_to)
    if args.metadata:
        builder.metadata(args.metadata)
    if args.mutual_tlsca:
        with open(args.mutual_tlsca, "r") as crt:
            builder.mutual_tlsca(bytearray(crt.read().encode()))
    if args.oauth_provider:
        builder.oauth(
            args.oauth_provider, args.allow_emails, args.allow_domains, args.scopes
        )
    if args.oidc:
        builder.oidc(
            args.oidc[0],
            args.oidc[1],
            args.oidc[2],
            args.allow_emails,
            args.allow_domains,
            args.scopes,
        )
    if args.proxy_proto:
        builder.proxy_proto(args.proxy_proto)
    if args.remove_request_header:
        for header in args.remove_request_header:
            builder.remove_request_header(header)
    if args.remove_response_header:
        for header in args.remove_response_header:
            builder.remove_response_header(header)
    if args.request_header:
        for header in args.request_header:
            builder.request_header(header[0], header[1])
    if args.response_header:
        for header in args.response_header:
            builder.response_header(header[0], header[1])
    if args.scheme:
        builder.scheme(args.scheme)
    if args.webhook_verification:
        builder.webhook_verification(
            args.webhook_verification[0], args.webhook_verification[1]
        )
    if args.websocket_tcp_conversion:
        builder.websocket_tcp_conversion(args.websocket_tcp_conversion)
    return builder


def gunicorn():
    from gunicorn.app.wsgiapp import run

    run()


def uvicorn():
    import uvicorn

    uvicorn.main()


def get_pipe_string(args):
    pipe_string = None
    if args.uds:
        pipe_string = args.uds
    if args.bind and args.bind.startswith("unix:"):
        pipe_string = args.bind[5:]
    return pipe_string


def fallback_port(args):
    fallback_port = DEFAULT_PORT
    if args.command == "gunicorn" and "PORT" in os.environ:
        fallback_port = os.getenv("PORT")
    return fallback_port


def get_tcp_string(args):
    tcp_string = None
    if args.host and args.port:
        tcp_string = args.host + ":" + str(args.port)
    elif args.host:
        tcp_string = args.host + ":" + fallback_port(args)
    elif args.port:
        tcp_string = DEFAULT_HOST + ":" + str(args.port)
    elif (
        args.bind
        and not args.bind.startswith("fd://")
        and not args.bind.startswith("unix:")
    ):
        tcp_string = args.bind
        # check if missing host
        if re.search("^:\d+$", tcp_string):
            tcp_string = DEFAULT_HOST + tcp_string
        # check if missing port
        if not re.search(":\d+$", tcp_string):
            tcp_string += ":" + fallback_port(args)
    return tcp_string


def setup_forwarding(tunnel, args, tcp_string=None):
    if tcp_string:
        tunnel.forward_tcp(tcp_string)
        return True

    # prefer pipe over tcp
    pipe_string = get_pipe_string(args)
    if pipe_string:
        tunnel.forward_pipe(pipe_string)
    else:
        tcp_string = get_tcp_string(args)
        if not tcp_string:
            return False
        tunnel.forward_tcp(tcp_string)

    return True


async def bind(parser, args):
    session = await configure_session(args).connect()
    tunnel = await configure_tunnel(session, args).listen()
    tunnel_success = setup_forwarding(tunnel, args)

    # if we don't have what we need, check gunicorn environment variable
    if (
        not tunnel_success
        and args.command == "gunicorn"
        and "GUNICORN_CMD_ARGS" in os.environ
    ):
        env_cmd_args = shlex.split(os.getenv("GUNICORN_CMD_ARGS"))
        env_args, unknown = parser.parse_known_args(env_cmd_args)
        tunnel_success = setup_forwarding(tunnel, env_args)

    # fallback to the default host and port for these runners
    if not tunnel_success:
        tunnel_success = setup_forwarding(
            tunnel, args, tcp_string=f"{DEFAULT_HOST}:{fallback_port(args)}"
        )

    # give up
    if not tunnel_success:
        logging.fatal("No tunnel created. Exiting.")
        sys.exit(1)

    return tunnel


def main(args):
    logging.basicConfig(level=logging.INFO)

    # argument parsing
    parser = ngrok_parser.get_parser()
    args, unknown = parser.parse_known_args()

    # validation
    if args.config:
        logging.fatal("Config file not supported. Exiting.")
        sys.exit(2)
    if args.fd or (args.bind and args.bind.startswith("fd://")):
        logging.fatal("File Descriptor not supported. Exiting.")
        sys.exit(3)

    # bind to ngrok
    tunnel = asyncio.run(bind(parser, args))

    # now pretend we don't exist
    pass_args = [args.command]
    # pass through some args
    pass_through_args = ngrok_parser.get_pass_through_args()
    for key, val in vars(args).items():
        if key in pass_through_args and val:
            pass_args.append(f"--{key}")
            pass_args.append(str(val))
    # pass through all unknown args
    sys.argv = pass_args + unknown
    if args.command == "gunicorn":
        gunicorn()
    elif args.command == "uvicorn":
        uvicorn()


def asgi_cli():
    main(sys.argv)


if __name__ == "__main__":
    main(sys.argv)
