import os

from django.core.management.commands.runserver import Command as RunserverCommand
from django.apps import apps

from ngrok_extra.django.listener import setup

if apps.is_installed('django.contrib.staticfiles'):
    # If the user wants to use the staticfiles app, we need to call the overriden the runserver command.
    from django.contrib.staticfiles.management.commands.runserver import Command as RunserverCommand

class Command(RunserverCommand):
    help = (
        "Starts up ngrok and forwards to the django lightweight web server for development."
    )

    def run(self, **options):
        """Start the ngrok connection and hand off to django server."""
        # This block handles 'make rundjango' 'make rundjangosite' which uses the INSTALLED_APPS 'ngrok.django' as the entry point.
        # Set env variable to protect against the autoreloader.
        if os.getenv("NGROK_LISTENER_RUNNING") is None:
            os.environ["NGROK_LISTENER_RUNNING"] = "true"
            import asyncio
            listen = f"{self.addr}:{self.port}" # RunserverCommand.handle sets this value for us.
            asyncio.run(setup(listen))
            print(f"Delegating django runserver to {RunserverCommand.__module__}")
        super().run(**options)
