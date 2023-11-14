#!/usr/bin/env python

import argparse
import logging
import os

import gradio as gr
import ngrok
from pygpt4all.models.gpt4all import GPT4All

logging.basicConfig(level=logging.INFO)

default_port = 7860


def create_ngrok_listener():
    parser = argparse.ArgumentParser()
    parser.add_argument("--host", help="Hostname or IP address", default="localhost")
    parser.add_argument("--port", help="Port number", type=int, default=default_port)

    args, _ = parser.parse_known_args()

    return ngrok.forward(f"{args.host}:{args.port}", authtoken_from_env=True)


# setup ngrok
listener = create_ngrok_listener()

# Load model path from environment.
if "MODEL_PATH" not in os.environ:
    raise Exception(
        "MODEL_PATH environment variable not found. Please set it to the path of your model."
    )

model_path = os.environ["MODEL_PATH"]
model = GPT4All(model_path)


# Create model function
def gpt4all(prompt):
    resp = [token for token in model.generate(prompt)]
    return "".join(resp).strip()


# Create a Gradio interface
demo = gr.Interface(fn=gpt4all, inputs="text", outputs="text")
demo.launch()
