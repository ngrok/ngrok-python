#!/usr/bin/env python
#
# For basic launch, run:
# python streamlit-ngrok.py

import asyncio, ngrok
import click

from streamlit.web.bootstrap import run

app_port = 8501


async def setup_listener():
    listen = f"localhost:{app_port}"
    session = await ngrok.SessionBuilder().authtoken_from_env().connect()
    listener = await (
        session.http_endpoint()
        # .domain('<name>.ngrok.app') # if on a paid plan, set a custom static domain
        .listen()
    )
    click.secho(
        f"Forwarding to {listen} from ingress url: {listener.url()}",
        fg="green",
        bold=True,
    )
    listener.forward(listen)


try:
    # Check if asyncio loop is already running. If so, piggyback on it to run the ngrok listener.
    running_loop = asyncio.get_running_loop()
    running_loop.create_task(setup_listener())
except RuntimeError:
    # No existing loop is running, so we can run the ngrok listener on a new loop.
    asyncio.run(setup_listener())

run("streamlit-demo.py", command_line=None, args=[], flag_options={})
