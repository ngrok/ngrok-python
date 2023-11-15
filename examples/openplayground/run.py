#!/usr/bin/env python

import argparse, logging, ngrok
from server import app

# setup ngrok
parser = argparse.ArgumentParser()
parser.add_argument("--host", help="Hostname or IP address", default="localhost")
parser.add_argument("--port", help="Port number", type=int, default=5432)
args, unknown = parser.parse_known_args()
logging.basicConfig(level=logging.INFO)
listener = ngrok.forward(f"{args.host}:{args.port}", authtoken_from_env=True)

# call the runner
app.run()
