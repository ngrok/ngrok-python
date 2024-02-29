from django.conf import settings

async def setup(listen="localhost:8000"):
    import ngrok
    # Note (james): This is where we can get settings for this app and pass them in if we want to.
    # ngrok_config = getattr(settings, "NGROK_CONFIG", {})
    # If hot reload is on though we'll need to handle that some other way.
    listener = await ngrok.default()
    print(f"Forwarding to {listen} from ingress url: {listener.url()}")
    listener.forward(listen)
