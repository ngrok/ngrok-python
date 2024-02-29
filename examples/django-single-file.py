#!/usr/bin/env python
# Adapted from: https://arunrocks.com/django-application-in-one-file/

from django.conf import settings
from django.core.management import execute_from_command_line
from django.http import HttpResponse
from django.urls import path


async def home(request):
    response = HttpResponse("Hello")
    return response


settings.configure(
    ALLOWED_HOSTS=["*"],  # Disable host header validation
    DEBUG=True,
    MIDDLEWARE=[
        "django.middleware.common.CommonMiddleware"
    ],  # CommonMiddleware adds Content-Length header
    INSTALLED_APPS=[
        "ngrok_extra.django",
    ],
    ROOT_URLCONF=__name__,
    SECRET_KEY="a-bad-secret",  # Insecure! Change this
)
urlpatterns = [path("", home)]

execute_from_command_line([__name__, "runserver"])
