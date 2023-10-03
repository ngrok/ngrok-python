#!/usr/bin/env python

import flask, logging, ngrok

logging.basicConfig(level=logging.INFO)
listener = ngrok.werkzeug_develop()

app = flask.Flask(__name__)


@app.route("/")
def hello():
    return "Hello, World!"


app.run()
