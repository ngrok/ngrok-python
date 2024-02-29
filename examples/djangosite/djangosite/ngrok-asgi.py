"""
ASGI config for djangosite project.

It exposes the ASGI callable as a module-level variable named ``application``.

For more information on this file, see
https://docs.djangoproject.com/en/4.1/howto/deployment/asgi/
"""

import os

# All a user needs to do is import from `ngrok_extra.django.asgi` rather than `django.core.asgi` to get the ngrok functionality.
from ngrok_extra.django.asgi import get_asgi_application

os.environ.setdefault("DJANGO_SETTINGS_MODULE", "djangosite.settings")

# This block handles 'make run-django-uvicorn' and 'make run-django-gunicorn' which uses this asgi.py as the entry point.
application = get_asgi_application()
