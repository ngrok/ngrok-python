Release Instructions
--------------------

Pre-Commit
----------
1. fix-n-fmt
1. make test

Release
-------
1. `git checkout main; git pull origin main`
1. `git checkout -b <username>/<version>`
1. Bump version number in `version` in `Cargo.toml`
1. Update CHANGELOG.md
1. make docs
1. `git add .`
1. `git commit -m '<version>'`
1. `git push origin <username>/<version>`
    - Or with graphite: `gt branch track` and `gt stack submit`
1. Create a pull request off this branch
1. Merge the pull request
1. Run the `Publish All` workflow on `main`: Actions → Publish All → Run workflow
    - The workflow builds the wheels, publishes them to PyPI, and pushes the `v<version>` tag
    - PyPI trusts this workflow through [trusted publishing](https://docs.pypi.org/trusted-publishers/) in the `pypi` environment, so no API token is needed
1. Verify release appears on [PyPI](https://pypi.org/project/ngrok/#history)
