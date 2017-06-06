EXPECTED = expected
OUTPUT = output
PROTO_PATH = proto

JAVA_OUT = ${OUTPUT}/java
PYTHON_OUT = ${OUTPUT}/python
JS_OUT = ${OUTPUT}/js

SUITES ?= python java js
TOOL ?= cargo run -q --
TARGET ?= test

python_args ?=
java_args ?= -m builder
js_args ?=

.PHONY: all it update clean ${SUITES}

all: clean it

it: ${SUITES}
	@echo "Verifying"
	@diff -ur $(EXPECTED) $(OUTPUT)

update: ${SUITES}
	@rsync -rav $(OUTPUT)/ $(EXPECTED)/
	git add $(EXPECTED)

clean:
	@${RM} -rf output

python:
	@echo "Building Python"
	@${TOOL} compile -b python ${python_args} -o ${PYTHON_OUT} --path ${PROTO_PATH} --package ${TARGET}

js:
	@echo "Building JavaScript"
	@${TOOL} compile -b js ${js_args} -o ${JS_OUT} --path ${PROTO_PATH} --package ${TARGET}

java:
	@echo "Building Java"
	@${TOOL} compile -b java ${java_args} -o ${JAVA_OUT} --path ${PROTO_PATH} --package ${TARGET}
