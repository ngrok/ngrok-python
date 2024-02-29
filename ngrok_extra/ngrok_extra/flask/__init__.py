from enum import Enum

import gunicorn.app.base
import ngrok

from typing import Awaitable, Dict
from flask import Flask
from ngrok import Listener


class NgrokFlask(Flask):
    class RunnerType(Enum):
        Werkzeug = 1
        Gunicorn = 2

    class _GunicornApp(gunicorn.app.base.BaseApplication):
        def __init__(self, app, options=None):
            self.application = app
            self.options = options or dict()
            super().__init__()

        def load_config(self):
            config = {key: value for key, value in self.options.items()
                      if key in self.cfg.settings and value is not None}
            for key, value in config.items():
                self.cfg.set(key.lower(), value)

        def load(self):
            return self.application

    listener: Listener | Awaitable[Listener]
    ngrok_options: Dict[str, object]
    runner: RunnerType

    def __init__(self, name: str, *args, ngrok_options: Dict[str, object] | None = None, gunicorn_options: Dict[str, object] | None = None, runner: RunnerType = RunnerType.Werkzeug, **kwargs):
        self.ngrok_options = ngrok_options or {}
        self.gunicorn_options = gunicorn_options or {}
        self.runner = runner
        super().__init__(name, *args, **kwargs)

    def run(self, **kwargs):
        host, port = '127.0.0.1', 5000

        if self.config.get('host') is not None:
            host = self.config['host']
        if self.config.get('port') is not None:
            port = int(self.config['port'])

        if kwargs.get('host') is not None:
            host = kwargs['host']
        if kwargs.get('port') is not None:
            port = int(kwargs['port'])

        bind = '{}:{}'.format(host, port)

        if self.runner == self.RunnerType.Gunicorn.value:
            if self.gunicorn_options.get('bind') is not None:
                bind = str(self.gunicorn_options['bind'])
            else:
                self.gunicorn_options['bind'] = bind

            self.listener = ngrok.forward(addr=bind, authtoken_from_env=True, **self.ngrok_options)
            print("Ngrok tunnel available at {}".format(self.listener.url()))
            self._GunicornApp(self, self.gunicorn_options).run()
        else:
            self.listener = ngrok.forward(addr=bind, authtoken_from_env=True, **self.ngrok_options)
            print("Ngrok tunnel available at {}".format(self.listener.url()))
            super(NgrokFlask, self).run(**kwargs)


Werkzeug = NgrokFlask.RunnerType.Werkzeug.value
Gunicorn = NgrokFlask.RunnerType.Gunicorn.value