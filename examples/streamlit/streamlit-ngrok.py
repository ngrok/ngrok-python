#!/usr/bin/env python
#
# For basic launch, run:
# python streamlit-ngrok.py

import asyncio, ngrok
import click

from streamlit.web.bootstrap import run 

app_port = 8501

async def setup_tunnel():
    listen = f"localhost:{app_port}"
    session = await ngrok.NgrokSessionBuilder().authtoken_from_env().connect()
    tunnel = await (
        session.http_endpoint()
        # .domain('<name>.ngrok.app') # if on a paid plan, set a custom static domain
        .listen()
    )
    click.secho(f"Forwarding to {listen} from ingress url: {tunnel.url()}", fg="green", bold=True)
    tunnel.forward_tcp(listen)


try:
    # Check if asyncio loop is already running. If so, piggyback on it to run the ngrok tunnel.
    running_loop = asyncio.get_running_loop()
    running_loop.create_task(setup_tunnel())
except RuntimeError:
    # No existing loop is running, so we can run the ngrok tunnel on a new loop.
    asyncio.run(setup_tunnel())

run('streamlit-demo.py', command_line=None, args=[], flag_options={})