from ngrok_extra.flask import NgrokFlask, Gunicorn, Werkzeug

app = NgrokFlask(__name__, runner=Gunicorn)


@app.route('/')
def hello_world():
    return 'Hello World!'


if __name__ == '__main__':
    app.run()