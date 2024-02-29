import json
from django.conf import settings
from typing import Dict

from ngrok_extra.policy import policy_builder


async def setup(listen="localhost:8000"):
    import ngrok

    # Note (james): This is where we can get settings for this app and pass them in if we want to.
    # Hot reloading of this config is only supported when a domain is specified.
    ngrok_config = getattr(settings, "NGROK_CONFIG", {})
    listener = None
    if ngrok_config:
        listener = await listener_from_config(ngrok_config)
    else:
        listener = await ngrok.default()
    print(f"Forwarding to {listen} from ingress url: {listener.url()}")
    listener.forward(listen)


async def listener_from_config(config: Dict):
    import ngrok

    session = await ngrok.SessionBuilder().authtoken_from_env().connect()
    listener = session.http_endpoint()
    domain = config.get("domain", "")
    if domain:
        listener = listener.domain(domain)
    policies: policy_builder.PolicyBuilder = config.get("policies", None)
    if policies:
        listener = listener.policy(policies.build())
    return await listener.listen()
