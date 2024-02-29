import copy
import json

class PolicyBuilder:
    def __init__(self, inbound_policy_rules=None, outbound_policy_rules=None, enabled=True):
        if inbound_policy_rules is None:
            self.inbound_policy_rules = []
        else:
            self.inbound_policy_rules = inbound_policy_rules

        if outbound_policy_rules is None:
            self.outbound_policy_rules = []
        else:
            self.outbound_policy_rules = inbound_policy_rules

        self.enabled = enabled
            
    def with_inbound_policy_rule(self, rule: str):
        builder_copy = self.copy()
        builder_copy.inbound_policy_rules.append(rule)
        return builder_copy

    def with_outbound_policy_rule(self, rule: str):
        builder_copy = self.copy()
        builder_copy.outbound_policy_rules.append(rule)
        return builder_copy

    def with_enabled(self, enabled: bool):
        builder_copy = self.copy()
        builder_copy.enabled = enabled
        return builder_copy

    def copy(self):
        builder_copy = PolicyBuilder()
        builder_copy.inbound_policy_rules = copy.deepcopy(self.inbound_policy_rules)
        builder_copy.outbound_policy_rules = copy.deepcopy(self.outbound_policy_rules)
        builder_copy.enabled = self.enabled
        return builder_copy
    
    def encode(self):
        return {
            "inbound": self.inbound_policy_rules,
            "outbound": self.outbound_policy_rules,
            "enabled": self.enabled
        }
    
    def build(self):
        return json.dumps(self.encode(), default=lambda o: o._as_json() if hasattr(o, "_as_json") else o.__dict__)


class PolicyRule:
    def __init__(self, expressions=None, actions=None):
        if expressions is None:
            self.expressions = []
        else:
            self.expressions = expressions

        if actions is None:
            self.actions = []
        else:
            self.actions = actions

    def with_expression(self, expression: str):
        rule_copy = self.copy()
        rule_copy.expressions.append(expression)
        return rule_copy

    def with_action(self, type: str, config: dict):
        rule_copy = self.copy()
        rule_copy.actions.append({"type": type, "config": config})
        return rule_copy

    def with_custom_response(self, config: CustomResponseConfig):
        return self.with_action("custom-response", config)
    
    def with_log(self, config: LogConfig):
        return self.with_action("log", config)

    def with_url_rewrite(self, config: URLRewriteConfig):
        return self.with_action("url-rewrite", config)

    def with_add_headers(self, config: AddHeadersConfig):
        return self.with_action("add-headers", config)

    def with_remove_headers(self, config: RemoveHeadersConfig):
        return self.with_action("remove-headers", config)

    def with_deny(self, config: DenyConfig):
        return self.with_action("deny", config)

    def with_jwt_validation(self, config: JWTValidationConfig):
        return self.with_action("url-rewrite", config)
    
    def with_rate_limit(self, config: RateLimitConfig):
        return self.with_action("rate-limit", config)
    
    def copy(self):
        rule_copy = PolicyRule()
        rule_copy.expressions = copy.deepcopy(self.expressions)
        rule_copy.actions = copy.deepcopy(self.actions)
        return rule_copy