import json
from django.conf import settings
from typing import Dict

from django.urls import get_resolver

from ngrok_extra.django.urls import RoutePolicies
from ngrok_extra.policy.policy_builder import PolicyBuilder


async def setup(listen="localhost:8000"):
    import ngrok

    # Hot reloading of this config is only supported when a domain is specified.
    ngrok_config = getattr(settings, "NGROK_CONFIG", {})
    listener = None
    route_policies = None
    domain = ngrok_config.get("domain", "")
    if domain:
        # We only add route policies from the URLs if a domain is specified
        route_policies = RoutePolicies([], [], domain)
        route_policies.add_routes(
            get_resolver().url_patterns
        )  # Doing this forces the loading of all routes
    if ngrok_config or route_policies:
        listener = await listener_from_config(ngrok_config, route_policies)
    else:
        listener = await ngrok.default()
    print(f"Forwarding to {listen} from ingress url: {listener.url()}")
    listener.forward(listen)


async def listener_from_config(config: Dict, route_policies: RoutePolicies):
    import ngrok

    session = await ngrok.SessionBuilder().authtoken_from_env().connect()
    listener = session.http_endpoint()
    domain = config.get("domain", "")
    if domain:
        listener = listener.domain(domain)
    policies: PolicyBuilder = config.get("policies", PolicyBuilder())
    if route_policies is not None and route_policies.has_policies():
        for policy in route_policies.inbound_rules:
            policies = policies.with_inbound_policy_rule(policy)
        for policy in route_policies.outbound_rules:
            policies = policies.with_outbound_policy_rule(policy)
    print(f"Applied Policies:")
    print(json.dumps(json.loads(policies.build()), indent=2))
    listener = listener.policy(policies.build())
    return await listener.listen()
