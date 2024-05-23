#!/usr/bin/env python

from aiohttp import web
import logging, ngrok


async def hello(request):
    return web.Response(text="Online in One Line")


logging.basicConfig(level=logging.INFO)
app = web.Application()
app.add_routes([web.get("/", hello)])
web.run_app(app, sock=ngrok.listen())
