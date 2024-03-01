from ngrok_extra.flask import NgrokFlask, Gunicorn, Werkzeug
from ngrok_extra.policy.policy_builder import PolicyRule
from ngrok_extra.policy.policy_configs import DenyConfig, AddHeadersConfig

app = NgrokFlask(__name__, runner=Gunicorn)


@app.route("/")
def hello_world():
    return "Hello World!"


@app.route("/bad", inbound_rule=PolicyRule().with_deny(DenyConfig(400)))
def bad():
    return "you should never see these words"


@app.route(
    "/headers",
    outbound_rule=PolicyRule().with_add_headers(
        AddHeadersConfig({"X-Ngrok-foo": "Bar"})
    ),
)
def add_headers():
    return "this has extra headers"


if __name__ == "__main__":
    app.run()
