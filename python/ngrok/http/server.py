import ngrok.utils
import subprocess
import sys
import argparse

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    http_server_opts = parser.add_argument_group("http.server Options")
    http_server_opts.add_argument(
        "--bind",
        type=str,
        help="Bind address, default is localhost.",
        default="localhost",
    )
    http_server_opts.add_argument(
        "--directory", type=str, help="Directory, default is pwd.", default="."
    )
    http_server_opts.add_argument(
        "--port", type=int, help="Port to bind to, default is 8000.", default=8000
    )

    access_options = parser.add_argument_group("Access Control Options")
    access_options.add_argument(
        "--oauth-emails",
        type=str,
        help="Oauth email addresses to allow in. Example: --oauth-emails foo@example.com,bar@example.com",
    )

    opts = parser.parse_args()

    kwargs = {}
    if opts.oauth_emails and opts.oauth_emails.strip():
        kwargs["oauth_provider"] = "google"
        kwargs["oauth_allow_emails"] = opts.oauth_emails.split(",")

    with ngrok.utils.scoped_endpoint(
        8000, authtoken_from_env=True, **kwargs
    ) as endpoint:
        cmd = [
            "python3",
            "-m",
            "http.server",
            str(opts.port),
            "--bind",
            opts.bind,
            "-d",
            opts.directory,
        ]
        http_server = subprocess.Popen(cmd)
        print(f"Available externally as: {endpoint.url()}")
        http_server.wait()
