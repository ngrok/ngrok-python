#!/usr/bin/env python

import flask, ngrok

tunnel = ngrok.werkzeug_develop()
print("tunnel established at: {}".format(tunnel.url()))

if __name__ == "__main__":
  app = flask.Flask(__name__)
  @app.route('/')
  def hello():
    return 'Hello, World!'
  app.run()
