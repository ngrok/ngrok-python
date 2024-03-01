from enum import Enum

import gunicorn.app.base
import ngrok

from typing import Awaitable, Dict
import typing as t
from flask import Flask
from flask import typing as ft
from ngrok import Listener
from ngrok_extra.policy.policy_builder import PolicyBuilder, PolicyRule

T_route = t.TypeVar("T_route", bound=ft.RouteCallable)


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
    policy: PolicyBuilder
    
    def __init__(self, name: str, *args: object, ngrok_options: Dict[str, object] | None = None,
                 gunicorn_options: Dict[str, object] | None = None,
                 runner: RunnerType = RunnerType.Werkzeug,
                 **kwargs: object):
        self.ngrok_options = ngrok_options or {}
        self.gunicorn_options = gunicorn_options or {}
        self.runner = runner
        self.policy = PolicyBuilder()
        super().__init__(name, *args, **kwargs)

    def add_inbound_rule(self, rule: PolicyRule):
        self.policy = self.policy.with_inbound_policy_rule(rule)

    def add_outbound_rule(self, rule: PolicyRule):
        self.policy = self.policy.with_outbound_policy_rule(rule)

    def route(self, route: str, **options: t.Any) -> t.Callable[[T_route], T_route]:
        def decorator(f: T_route) -> T_route:
            endpoint = options.pop("endpoint", None)
            inbound_rule: PolicyRule | None = options.pop("inbound_rule", None)
            outbound_rule: PolicyRule | None = options.pop("outbound_rule", None)

            if inbound_rule is not None:
                inbound_rule = inbound_rule.with_expression("req.URL.contains('{}')".format(route))
                self.add_inbound_rule(inbound_rule)

            if outbound_rule is not None:
                outbound_rule = outbound_rule.with_expression("req.URL.contains('{}')".format(route))
                self.add_outbound_rule(outbound_rule)

            self.add_url_rule(route, endpoint, f, **options)

            return f

        return decorator

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

        policy = self.ngrok_options.get('policy', None)
        if policy is None and len(self.policy) != 0:
            policy = self.policy.build()

        if policy is not None:
            self.ngrok_options['policy'] = policy

        if self.runner == self.RunnerType.Gunicorn:
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


Werkzeug = NgrokFlask.RunnerType.Werkzeug
Gunicorn = NgrokFlask.RunnerType.Gunicorn