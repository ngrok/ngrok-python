import asyncio
import os
from django.conf import settings

from django.core.management.commands.runserver import Command as RunserverCommand
from django.apps import apps

from ngrok_extra.django.listener import setup

if apps.is_installed("django.contrib.staticfiles"):
    # If the user wants to use the staticfiles app, we need to call the overriden the runserver command.
    from django.contrib.staticfiles.management.commands.runserver import (
        Command as RunserverCommand,
    )


class Command(RunserverCommand):
    help = "Starts up ngrok and forwards to the django lightweight web server for development."

    def run(self, **options):
        """Start the ngrok connection and hand off to django server."""
        # This block handles 'make rundjango' 'make rundjangosite' which uses the INSTALLED_APPS 'ngrok.django' as the entry point.

        use_reloader = options["use_reloader"] and self.config_supports_reload()
        if not use_reloader or os.environ.get("RUN_MAIN") == "true":
            # we're either running without reloads or we're in the child autoreload process.
            if os.getenv("NGROK_LISTENER_RUNNING") is None:
                # Set env variable to protect against the autoreloader.
                os.environ["NGROK_LISTENER_RUNNING"] = "true"

                listen = f"{self.addr}:{self.port}"  # RunserverCommand.handle sets this value for us.
                asyncio.run(setup(listen))
                print(f"Delegating django runserver to {RunserverCommand.__module__}")
        elif use_reloader:
            print("Reloader is on waiting for child process to start ngrok.")
        else:
            print("ngrok already running.")
        super().run(**options)

    @staticmethod
    def config_supports_reload():
        ngrok_config = getattr(settings, "NGROK_CONFIG", {})
        return ngrok_config and ngrok_config.get("domain", "") != ""
