#!/usr/bin/env python
#
# For basic launch, run:
# python gradio-ngrok.py
#
# For uvicorn with reloading, run:
# gradio gradio-ngrok.py

import asyncio, ngrok, os, sys
import gradio as gr


def greet(name):
    return "Hello " + name + "!"


demo = gr.Interface(fn=greet, inputs="text", outputs="text")


async def setup_listener():
    listen = "localhost:7860"
    session = await ngrok.SessionBuilder().authtoken_from_env().connect()
    listener = await (
        session.http_endpoint()
        # .domain('<name>.ngrok.app') # if on a paid plan, set a custom static domain
        .listen()
    )
    print(f"Forwarding to {listen} from ingress url: {listener.url()}")
    listener.forward(listen)


try:
    # 'gradio' command line already has a loop running via uvicorn
    running_loop = asyncio.get_running_loop()
    running_loop.create_task(setup_listener())
except RuntimeError:
    asyncio.run(setup_listener())
    # only call launch if not in uvicorn already
    demo.launch()
