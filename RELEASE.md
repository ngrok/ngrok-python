Release Instructions
--------------------
1. `git checkout main; git pull origin main`
1. `git checkout -b <username>/<version>`
1. Bump patch number in `version` in `Cargo.toml`
1. `git commit -m '<version>'`
1. `git push origin <username>/<version>`
1. Create a pull request off this branch
    - Make sure the name of the pull request is also just the `<version>` so the workflow will know to do a release
1. Merge the pull request
1. Verify release appears on [PyPI](https://pypi.org/project/ngrok/#history)
