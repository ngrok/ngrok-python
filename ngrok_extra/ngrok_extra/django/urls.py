from dataclasses import dataclass
from typing import List
from ngrok_extra.policy.policy_builder import PolicyRule
from django.urls import path as django_path
from django.urls import re_path as django_re_path


@dataclass
class RoutePolicies:
    inbound_rules: List[PolicyRule]
    outbound_rules: List[PolicyRule]
    base_domain: str = ""

    def __post_init__(self):
        if not self.base_domain.endswith("/"):
            self.base_domain = self.base_domain + "/"

    def add_inbound_rule(self, policy):
        self.inbound_rules.append(policy)

    def add_outbound_rule(self, policy):
        self.outbound_rules.append(policy)

    def has_policies(self):
        return bool(self.inbound_rules or self.outbound_rules)

    def add_routes(self, url_patterns, parent=None):
        for pattern in url_patterns:
            if hasattr(pattern, "url_patterns"):
                new_parent = self.add_route_policies(
                    pattern, parent
                )  # new parent from current route
                self.add_routes(pattern.url_patterns, new_parent)
            else:
                self.add_route_policies(pattern, parent)

    def add_route_policies(self, pattern, parent=None):
        regex_pattern = self.fix_regex(pattern.pattern.regex.pattern, parent)
        expression = "req.URL.matches('" + regex_pattern + "')"
        if hasattr(pattern, "inbound_rule"):
            self.add_inbound_rule(pattern.inbound_rule.with_expression(expression))
        if hasattr(pattern, "outbound_rule"):
            self.add_outbound_rule(pattern.outbound_rule.with_expression(expression))
        return regex_pattern

    def fix_regex(self, pattern, parent):
        if parent:
            pattern = parent + pattern
        else:
            # if ^ appears early in the regex, it's we replace with self.base_domain
            pattern = pattern.replace("^", self.base_domain, 1)
        # otherwise all other occurence of ^ are removed
        pattern = pattern.replace("^", "")
        pattern = pattern.replace("\\Z", "$")
        return pattern


def path(route, view, kwargs=None, name=None, inbound_rule=None, outbound_rule=None):
    url = django_path(route, view, kwargs, name)
    if inbound_rule:
        url.inbound_rule = inbound_rule
    if outbound_rule:
        url.outbound_rule = outbound_rule
    return url


def re_path(route, view, kwargs=None, name=None, inbound_rule=None, outbound_rule=None):
    url = django_re_path(route, view, kwargs, name)
    if inbound_rule:
        url.inbound_rule = inbound_rule
    if outbound_rule:
        url.outbound_rule = outbound_rule
    return url
