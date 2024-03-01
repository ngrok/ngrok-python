from sanic import Sanic
from sanic.response import text

app = Sanic("MyHelloWorldApp")

"""Added by ngrok"""
@app.listener("before_server_start")
async def start_ngrok(app, loop):
    loop.create_task(setup("localhost:8000"))

async def setup(listen):
    import ngrok
    listener = await ngrok.default()
    print(f"Forwarding to {listen} from ingress url: {listener.url()}")
    listener.forward(listen)
"""End Added by ngrok"""

@app.get("/")
async def hello_world(request):
    return text("Hello, world.")
