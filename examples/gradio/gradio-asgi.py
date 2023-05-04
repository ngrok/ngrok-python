#!/usr/bin/env python
#
# Run this with:
# ngrok-asgi uvicorn gradio-asgi:demo.app --port 7860 --reload

import asyncio, ngrok, os, sys
import gradio as gr


def greet(name):
    return "Hello " + name + "!"


demo = gr.Interface(fn=greet, inputs="text", outputs="text")

try:
    # 'gradio' command line already has a loop running via uvicorn
    running_loop = asyncio.get_running_loop()
except RuntimeError:
    # only call launch if not in uvicorn already
    demo.launch()
