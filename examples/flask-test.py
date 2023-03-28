#!/usr/bin/env python

import flask, logging, ngrok, sys

logging.basicConfig(level=logging.INFO)
tunnel = ngrok.werkzeug_develop()

if __name__ == "__main__":
  app = flask.Flask(__name__)
  @app.route('/')
  def hello():
    return 'Hello, World!'
  app.run()
