# system python interpreter. used only to create virtual environment
PY = python3
VENV = .env
BIN=$(VENV)/bin

# make it work on windows too
ifeq ($(OS), Windows_NT)
	BIN=$(VENV)/Scripts
	PY=python
endif

all: venv run

venv:
	: # Create venv if it doesn't exist
	test -d $(VENV) || ($(PY) -m venv $(VENV) && $(BIN)/pip install -r requirements.txt)

install:
	. $(BIN)/activate && pip install -r requirements.txt

develop: venv
	. $(BIN)/activate && maturin develop

build: venv
	. $(BIN)/activate && maturin build

run: develop
	. $(BIN)/activate && ./examples/ngrok-http-minimal.py

runaio: develop
	. $(BIN)/activate && python ./examples/aiohttp-test.py

runflask: develop
	. $(BIN)/activate && python ./examples/flask-test.py

runfull: develop
	. $(BIN)/activate && ./examples/ngrok-http-full.py

runlabeled: develop
	. $(BIN)/activate && ./examples/ngrok-labeled.py

runtcp: develop
	. $(BIN)/activate && ./examples/ngrok-tcp.py

runtls: develop
	. $(BIN)/activate && ./examples/ngrok-tls.py

runuvi: develop
	. $(BIN)/activate && python ./examples/uvicorn-test.py

# e.g.: make test=TestNgrok.test_gzip_tunnel test
test: develop
	. $(BIN)/activate && python ./test/test.py $(test)

# e.g.: make test=TestNgrok.test_gzip_tunnel testonly
testonly:
	. $(BIN)/activate && python ./test/test.py $(test)

# testfast is called by github workflow in ci.yml
testfast: develop
	. $(BIN)/activate && py.test -n 4 ./test/test.py

testpublish:
	. $(BIN)/activate && maturin publish --repository testpypi

docs: develop black
	. $(BIN)/activate && sphinx-build -b html doc_source/ docs/

black: develop
	. $(BIN)/activate && black examples

clean:
	rm -rf $(VENV) target/
