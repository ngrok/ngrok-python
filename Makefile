all: install run

install: venv
	: # Activate venv
	. .env/bin/activate && pip install -r requirements.txt

venv:
	: # Create venv if it doesn't exist
	test -d venv || python3 -m venv .env

develop: install
	. .env/bin/activate && maturin develop

build: install
	. .env/bin/activate && maturin build

run: develop
	. .env/bin/activate && ./examples/ngrok-http-minimal.py

runaio: develop
	. .env/bin/activate && python ./examples/aiohttp-test.py

runflask: develop
	. .env/bin/activate && python ./examples/flask-test.py

runfull: develop
	. .env/bin/activate && ./examples/ngrok-http-full.py

runlabeled: develop
	. .env/bin/activate && ./examples/ngrok-labeled.py

runtcp: develop
	. .env/bin/activate && ./examples/ngrok-tcp.py

runtls: develop
	. .env/bin/activate && ./examples/ngrok-tls.py

runuvi: develop
	. .env/bin/activate && python ./examples/uvicorn-test.py

# e.g.: make test=TestNgrok.test_gzip_tunnel test
test: develop
	. .env/bin/activate && python ./test/test.py $(test)

# e.g.: make test=TestNgrok.test_gzip_tunnel testonly
testonly:
	. .env/bin/activate && python ./test/test.py $(test)

# testfast is called by github workflow in ci.yml
testfast: develop
	. .env/bin/activate && py.test -n 4 ./test/test.py

testpublish:
	. .env/bin/activate && maturin publish --repository testpypi

docs: develop
	. .env/bin/activate && sphinx-build -b html doc_source/ docs/

clean:
	rm -rf .env target/
