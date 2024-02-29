import os
from django.core.asgi import get_asgi_application as _get_asgi_application
from ngrok_extra.django.listener import setup

def get_asgi_application():
    # This block handles 'make run-django-uvicorn' and 'make run-django-gunicorn' which uses this asgi.py as the entry point.
    # Set env variable to protect against the gunicorn autoreloader.
    if os.getenv("NGROK_LISTENER_RUNNING") is None:
        os.environ["NGROK_LISTENER_RUNNING"] = "true"
        import asyncio

        try:
            running_loop = asyncio.get_running_loop()
            running_loop.create_task(setup())
        except RuntimeError:
            # no running loop, run on its own
            asyncio.run(setup())

    return _get_asgi_application()