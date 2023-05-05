#!/usr/bin/env python

import argparse
import logging

import ngrok

default_port = 5000

def create_ngrok_tunnel():
    parser = argparse.ArgumentParser()
    parser.add_argument("--host", help="Hostname or IP address", default='localhost')
    parser.add_argument("--port", help="Port number", type=int, default=default_port)

    args, _ = parser.parse_known_args()
    logging.basicConfig(level=logging.INFO)

    return ngrok.connect(f"{args.host}:{args.port}", authtoken_from_env=True)

# setup ngrok
tunnel = create_ngrok_tunnel()

# import server after tunnel creation because Flask will override our logger
from server import run_server

run_server(tunnel.url())