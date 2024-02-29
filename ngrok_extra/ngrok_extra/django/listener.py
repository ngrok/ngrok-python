import json
from django.conf import settings

async def setup(listen="localhost:8000"):
    import ngrok
    # Note (james): This is where we can get settings for this app and pass them in if we want to.
    # If hot reload is on though we'll need to handle that some other way.
    ngrok_config = getattr(settings, "NGROK_CONFIG", {})
    listener = None
    if ngrok_config:
        listener = await listener_from_config(ngrok_config)
    else:
        listener = await ngrok.default()
    print(f"Forwarding to {listen} from ingress url: {listener.url()}")
    listener.forward(listen)

async def listener_from_config(config):
    import ngrok
    session = await ngrok.SessionBuilder().authtoken_from_env().connect()
    listener = session.http_endpoint()
    policies = config.get("policies", {})
    if policies:
        listener = listener.policy(json.dumps(policies))
    return await listener.listen()
