import logging
import os

import flask
from pygpt4all.models.gpt4all import GPT4All

logging.basicConfig(level=logging.INFO)

# Load model path from environment.
model_path = os.environ["MODEL_PATH"]
# Preload the model.
model = GPT4All(model_path)

app = flask.Flask(__name__)

tunnel_endpoint = None

def run_server(tunnel_url: str):
    global tunnel_endpoint
    tunnel_endpoint = tunnel_url
    app.run()


@app.route("/")
def index():
    return flask.render_template("index.html", tunnel_url=tunnel_endpoint)


@app.route("/prompt", methods=["POST"])
def prompt():
    prompt = flask.request.args.get("prompt")
    resp = [token for token in model.generate(prompt)]
    return flask.jsonify("".join(resp))
