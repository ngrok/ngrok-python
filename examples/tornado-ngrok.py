import asyncio
import tornado

import ngrok

app_port = 8889


class MainHandler(tornado.web.RequestHandler):
    def get(self):
        self.write("Hello, world")


def make_app():
    return tornado.web.Application(
        [
            (r"/", MainHandler),
        ]
    )


async def setup_tunnel():
    listen = f"localhost:{app_port}"
    session = await ngrok.NgrokSessionBuilder().authtoken_from_env().connect()
    tunnel = await (
        session.http_endpoint()
        # .domain('<name>.ngrok.app') # if on a paid plan, set a custom static domain
        .listen()
    )
    print(f"Forwarding to {listen} from ingress url: {tunnel.url()}")
    tunnel.forward(listen)


async def main():
    app = make_app()
    app.listen(app_port)
    await asyncio.Event().wait()


if __name__ == "__main__":
    asyncio.run(setup_tunnel())
    asyncio.run(main())
