script_input=$(CURDIR)/../tools/script-input

DIFF ?= diff
RSYNC ?= rsync
TOOL ?= cargo run -q --

EXPECTED = expected
OUTPUT = output
PROTO_PATH = proto

SUITES ?= python java js rust
TARGETS ?= test
FILTERED ?=

java_out = ${OUTPUT}/java
python_out = ${OUTPUT}/python
js_out = ${OUTPUT}/js
rust_out = ${OUTPUT}/rust

python_extra ?=
java_extra ?= -m builder
js_extra ?=
rust_extra ?=

java_suite := -b java $(java_extra)
java_project := -b java -m fasterxml -o workdir-java/target/generated-sources/reproto

js_suite := -b js $(js_extra)
js_project := -b js -o workdir-js/generated

python_suite := -b python $(python_extra)
python_project := -b python -o workdir-python/generated

rust_suite := -b rust $(rust_extra)
rust_project := -b rust -o workdir-rust/src

# projects that are filtered
FILTERED_PROJECTS ?=
# projects that are supported after checking that necessary tools are available
SUPPORTED_PROJECTS ?=

SUITES := $(filter-out $(FILTERED),$(SUITES))
PROJECTS := $(filter $(SUPPORTED_PROJECTS),$(filter-out $(FILTERED_PROJECTS),$(SUITES)))

PACKAGES := $(TARGETS:%=--package %)

PROJECT_TARGETS := $(PROJECTS:%=project-%)
SUITE_TARGETS := $(SUITES:%=suite-%)
PROJECTDIFFS := $(PROJECTS:%=projectdiff-%)
PROJECTUPDATES := $(PROJECTS:%=projectupdate-%)

DEBUG ?= no

ifeq ($(DEBUG),yes)
O :=
reproto := $(TOOL) --debug
else
O := @
reproto := $(TOOL)
endif

.PHONY: all clean suites projects update update-projects

all:
	$Omake suites
	$Omake projects

update:
	$Omake update-it
	$Omake update-projects

clean:
	$Orm -rf workdir-*
	$Orm -rf output-*
	$Orm -rf output

suites: $(SUITE_TARGETS) diff
projects: $(PROJECT_TARGETS) $(PROJECTDIFFS)

update-projects: $(PROJECT_TARGETS) $(PROJECTUPDATES)

update-it: $(SUITE_TARGETS)
	$Oecho "Updating Suites"
	$O$(RSYNC) -ra $(OUTPUT)/ $(EXPECTED)/

diff:
	$Oecho "Verifying Diffs"
	$O$(DIFF) -ur $(EXPECTED) $(OUTPUT)

# rule to diff a projects expected output, with actual.
projectdiff-%:
	$Oecho "Diffing Project: $*"
	$O$(DIFF) -ur expected-$* output-$*

# rule to update a projects expected output, with its actual
projectupdate-%:
	$Oecho "Updating Project: $*"
	$O$(RSYNC) -ra output-$*/ expected-$*/

# rule to build output for a project
project-%:
	$O$(RSYNC) -ra ../$*/ workdir-$*
	$O$(reproto) compile $($*_project) --path ${PROTO_PATH} ${PACKAGES}
	$Ocd workdir-$* && make
	$O${script_input} workdir-$*/script.sh

# rule to build suite output
suite-%:
	$Oecho "Suite: $*"
	$O${reproto} compile $($*_suite) -o $($*_out) --path ${PROTO_PATH} ${PACKAGES}
