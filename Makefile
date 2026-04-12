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

################################################################################
# Admin \
ADMIN::  ## ##################################################################
.PHONY: init-env
init-env:  ## init-env
	@rm -fr ~/xxx/*
	@mkdir -p "$(HOME)/xxx/ankiview-test/"
	cp -r ankiview/tests/fixtures/test_collection/* "$(HOME)/xxx/ankiview-test/"
	cp -v ankiview/examples/image-test.md.ori ankiview/examples/image-test.md
	cp -v ./ankiview/tests/fixtures/munggoggo.png ./ankiview/examples/munggoggo.png

.PHONY: create-note
create-note: init-env  ## create a note from markdown
	# cargo run --bin ankiview -- -c "$(HOME)/xxx/ankiview-test/User 1/collection.anki2" view 1695797540371
	~/dev/s/private/ankiview/ankiview/target/debug/ankiview -c "$(HOME)/xxx/ankiview-test/User 1/collection.anki2" collect ./ankiview/examples/image-test.md
	~/dev/s/private/ankiview/ankiview/target/debug/ankiview -c "$(HOME)/xxx/ankiview-test/User 1/collection.anki2" list | grep 'This is an image test!'

	@echo
	@echo "---- Following test should fail ---"
	@echo
	cp ./ankiview/tests/fixtures/gh_activity.png ./ankiview/examples/munggoggo.png
	~/dev/s/private/ankiview/ankiview/target/debug/ankiview -c "$(HOME)/xxx/ankiview-test/User 1/collection.anki2" collect ./ankiview/examples/image-test.md

.PHONY: anki
anki:  ## anki
	-pkill anki
	# specify base folder with -b
	open /Applications/Anki.app --args -b $(HOME)/xxx/ankiview-test

.PHONY: test
test:  ## tests, single-threaded (all functionality)
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

.PHONY: all
all: clean build install  ## all
	:

.PHONY: all-fast
all-fast: clean build-fast install-debug  ## all-debug: debug build
	:

.PHONY: doc
doc:  ## doc
	@rustup doc --std
	pushd $(pkg_src) && cargo doc --open

.PHONY: upload
upload:  ## upload
	@echo "anki not on crate.io, so cannot publish"

.PHONY: build
build:  ## build release version
	pushd $(pkg_src) && cargo build --release

.PHONY: build-fast
build-fast:  ## build debug version
	pushd $(pkg_src) && cargo build

# macOS Code Signing Fix:
# When Rust's linker builds a binary, it creates an adhoc linker-signed signature.
# When copied with `cp`, macOS preserves this signature but it becomes invalid
# because the hash was computed for the original path/inode. macOS AMFI (Apple
# Mobile File Integrity) detects the mismatch and kills the process with SIGKILL
# (signal 9, exit code 137). Re-signing with `codesign --force --sign -` creates
# a fresh adhoc signature valid for the new location.

.PHONY: install-debug
install-debug: uninstall  ## install-debug (no release version)
	@VERSION=$(shell cat VERSION) && \
		echo "-M- Installing $$VERSION" && \
		cp -vf ankiview/target/debug/$(BINARY) ~/bin/$(BINARY)$$VERSION && \
		codesign --force --sign - ~/bin/$(BINARY)$$VERSION && \
		ln -vsf ~/bin/$(BINARY)$$VERSION ~/bin/$(BINARY)
		# ~/bin/$(BINARY) completion bash > ~/.bash_completions/ankiview

.PHONY: install
install: uninstall  ## install
	@VERSION=$(shell cat VERSION) && \
		echo "-M- Installing $$VERSION" && \
		cp -vf ankiview/target/release/$(BINARY) ~/bin/$(BINARY)$$VERSION && \
		codesign --force --sign - ~/bin/$(BINARY)$$VERSION && \
		ln -vsf ~/bin/$(BINARY)$$VERSION ~/bin/$(BINARY)

.PHONY: uninstall
uninstall:  ## uninstall
	-@test -f ~/bin/$(BINARY) && rm -v ~/bin/$(BINARY)
	rm -vf ~/.bash_completions/ankiview

.PHONY: bump-major
bump-major:  check-github-token  ## bump-major, tag and push
	bump-my-version bump --commit --tag major
	git push
	git push --tags
	@$(MAKE) create-release

.PHONY: bump-minor
bump-minor:  check-github-token  ## bump-minor, tag and push
	bump-my-version bump --commit --tag minor
	git push
	git push --tags
	@$(MAKE) create-release

.PHONY: bump-patch
bump-patch:  check-github-token  ## bump-patch, tag and push
	bump-my-version bump --commit --tag patch
	git push
	git push --tags
	@$(MAKE) create-release

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

.PHONY: fix-version
fix-version: check-github-token  ## fix-version of Cargo.toml, re-connect with HEAD
	git add ankiview/Cargo.lock
	git commit --amend --no-edit
	git tag -f "v$(VERSION)"
	git push --force-with-lease
	git push --tags --force

.PHONY: format
format:  ## format
	pushd $(pkg_src) && cargo fmt

.PHONY: lint
lint:  ## lint and fix
	pushd $(pkg_src) && cargo clippy --fix -- -A unused_imports
	pushd $(pkg_src) && cargo fix --lib -p ankiview --tests


################################################################################
# Clean \
CLEAN:  ## ############################################################

.PHONY: clean
clean:clean-rs  ## clean all
	:

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
