#!/usr/bin/env python

import logging, ngrok, uvicorn

async def app(scope, receive, send):
  assert scope['type'] == 'http'

  await send({
    'type': 'http.response.start',
    'status': 200,
    'headers': [[b'content-type', b'text/plain']],
  })
  await send({
    'type': 'http.response.body',
    'body': b'Hello, world!',
  })

logging.basicConfig(level=logging.INFO)
uvicorn.run(app=app, fd=ngrok.fd())
