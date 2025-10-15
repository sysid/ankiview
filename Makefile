.DEFAULT_GOAL := help
#MAKEFLAGS += --no-print-directory

# You can set these variables from the command line, and also from the environment for the first two.
PREFIX ?= /usr/local
BINPREFIX ?= "$(PREFIX)/bin"

VERSION       = $(shell cat VERSION)

SHELL	= bash
.ONESHELL:

app_root := $(if $(PROJ_DIR),$(PROJ_DIR),$(CURDIR))
pkg_src =  $(app_root)/ankiview
tests_src = $(app_root)/ankiview/tests
BINARY = ankiview

# Makefile directory
CODE_DIR := $(dir $(abspath $(lastword $(MAKEFILE_LIST))))

# define files
MANS = $(wildcard ./*.md)
MAN_HTML = $(MANS:.md=.html)
MAN_PAGES = $(MANS:.md=.1)
# avoid circular targets
MAN_BINS = $(filter-out ./tw-extras.md, $(MANS))

################################################################################
# Admin \
ADMIN::  ## ##################################################################
.PHONY: init-env
init-env:  ## init-env
	@rm -fr ~/xxx/*
	@mkdir -p ~/xxx

.PHONY: test
test:  ## Run all tests (unit, integration, and doc tests) with debug logging
	pushd $(pkg_src) && RUST_LOG=INFO cargo test --all-features --all-targets -- --test-threads=1  #--nocapture

.PHONY: refresh-test-fixture
refresh-test-fixture:  ## Refresh test fixture from golden dataset
	@echo "Refreshing test fixture from golden dataset..."
	./ankiview/tests/fixtures/copy_golden_dataset.sh

.PHONY: test-verbose
test-verbose:  ## Run tests with verbose logging
	pushd $(pkg_src) && RUST_LOG=debug cargo test --all-features --all-targets -- --test-threads=1 --nocapture

################################################################################
# Building, Deploying \
BUILDING:  ## ##################################################################

.PHONY: doc
doc:  ## doc
	@rustup doc --std
	pushd $(pkg_src) && cargo doc --open

.PHONY: all
all: clean build install  ## all
	:

.PHONY: upload
upload:  ## upload
	@echo "anki not on crate.io, so cannt publish"
	#@if [ -z "$$CARGO_REGISTRY_TOKEN" ]; then \
	#	echo "Error: CARGO_REGISTRY_TOKEN is not set"; \
	#	exit 1; \
	#fi
	#@echo "CARGO_REGISTRY_TOKEN is set"
	#pushd $(pkg_src) && cargo release publish --execute

.PHONY: build
build:  ## build
	pushd $(pkg_src) && cargo build --release

.PHONY: build-fast
build-fast:  ## build debug version
	pushd $(pkg_src) && cargo build

.PHONY: install-debug
install-debug: uninstall  ## install-debug (no release version)
	@VERSION=$(shell cat VERSION) && \
		echo "-M- Installing $$VERSION" && \
		cp -vf ankiview/target/debug/$(BINARY) ~/bin/$(BINARY)$$VERSION && \
		ln -vsf ~/bin/$(BINARY)$$VERSION ~/bin/$(BINARY)
		# ~/bin/$(BINARY) completion bash > ~/.bash_completions/ankiview

#.PHONY: install
#install: uninstall  ## install
	#@cp -vf $(pkg_src)/target/release/$(BINARY) ~/bin/$(BINARY)
.PHONY: install
install: uninstall  ## install
	@VERSION=$(shell cat VERSION) && \
		echo "-M- Installagin $$VERSION" && \
		cp -vf ankiview/target/release/$(BINARY) ~/bin/$(BINARY)$$VERSION && \
		ln -vsf ~/bin/$(BINARY)$$VERSION ~/bin/$(BINARY)

.PHONY: uninstall
uninstall:  ## uninstall
	-@test -f ~/bin/$(BINARY) && rm -v ~/bin/$(BINARY)
	rm -vf ~/.bash_completions/ankiview

.PHONY: bump-major
bump-major:  ## bump-major, tag and push
	bump-my-version bump --commit --tag major
	git push
	git push --tags
	@$(MAKE) create-release

.PHONY: bump-minor
bump-minor:  ## bump-minor, tag and push
	bump-my-version bump --commit --tag minor
	git push
	git push --tags
	@$(MAKE) create-release

.PHONY: bump-patch
bump-patch:  ## bump-patch, tag and push
	bump-my-version bump --commit --tag patch
	git push
	git push --tags
	@$(MAKE) create-release

.PHONY: style
style:  ## style
	pushd $(pkg_src) && cargo fmt

.PHONY: lint
lint:  ## lint
	pushd $(pkg_src) && cargo clippy

.PHONY: create-release
create-release: check-github-token  ## create a release on GitHub via the gh cli
	@if ! command -v gh &>/dev/null; then \
		echo "You do not have the GitHub CLI (gh) installed. Please create the release manually."; \
		exit 1; \
	else \
		echo "Creating GitHub release for v$(VERSION)"; \
		gh release create "v$(VERSION)" --generate-notes --latest; \
	fi

.PHONY: check-github-token
check-github-token:  ## Check if GITHUB_TOKEN is set
	@if [ -z "$$GITHUB_TOKEN" ]; then \
		echo "GITHUB_TOKEN is not set. Please export your GitHub token before running this command."; \
		exit 1; \
	fi
	@echo "GITHUB_TOKEN is set"
	#@$(MAKE) fix-version  # not working: rustrover deleay

.PHONY: fix-version
fix-version:  ## fix-version of Cargo.toml, re-connect with HEAD
	git add bkmr/Cargo.lock
	git commit --amend --no-edit
	git tag -f "v$(VERSION)"
	git push --force-with-lease
	git push --tags --force


################################################################################
# Clean \
CLEAN:  ## ############################################################

.PHONY: clean
clean:clean-rs  ## clean all
	:

.PHONY: clean-build
clean-build: ## remove build artifacts
	rm -fr build/
	rm -fr dist/
	rm -fr .eggs/
	find . \( -path ./env -o -path ./venv -o -path ./.env -o -path ./.venv \) -prune -o -name '*.egg-info' -exec rm -fr {} +
	find . \( -path ./env -o -path ./venv -o -path ./.env -o -path ./.venv \) -prune -o -name '*.egg' -exec rm -f {} +

.PHONY: clean-pyc
clean-pyc: ## remove Python file artifacts
	find . -name '*.pyc' -exec rm -f {} +
	find . -name '*.pyo' -exec rm -f {} +
	find . -name '*~' -exec rm -f {} +
	find . -name '__pycache__' -exec rm -fr {} +

.PHONY: clean-rs
clean-rs:  ## clean-rs
	pushd $(pkg_src) && cargo clean -v

################################################################################
# Misc \
MISC:  ## ############################################################

define PRINT_HELP_PYSCRIPT
import re, sys

for line in sys.stdin:
	match = re.match(r'^([%a-zA-Z0-9_-]+):.*?## (.*)$$', line)
	if match:
		target, help = match.groups()
		if target != "dummy":
			print("\033[36m%-20s\033[0m %s" % (target, help))
endef
export PRINT_HELP_PYSCRIPT

.PHONY: help
help:
	@python -c "$$PRINT_HELP_PYSCRIPT" < $(MAKEFILE_LIST)

debug:  ## debug
	@echo "-D- CODE_DIR: $(CODE_DIR)"


.PHONY: list
list: *  ## list
	@echo $^

.PHONY: list2
%: %.md  ## list2
	@echo $^


%-plan:  ## call with: make <whatever>-plan
	@echo $@ : $*
	@echo $@ : $^
