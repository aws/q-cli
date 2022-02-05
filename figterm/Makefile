ROOT:=$(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))

export BUILT_PRODUCTS_DIR ?= $(ROOT)/build

# Add brew executables to PATH
export PATH := /opt/homebrew/bin:/opt/homebrew/sbin:$(PATH)
# Add cargo executables to PATH
export PATH := $(HOME)/.cargo/bin:$(PATH)

SHELL := env PATH=$(PATH) /bin/bash

# check if cargo exists
CARGO := $(shell command -v cargo 2> /dev/null)

CARGO_FLAGS ?= --release --locked

ifndef CARGO
    $(error "cargo is not in PATH $(PATH)")
endif

$(shell mkdir -p $(BUILT_PRODUCTS_DIR))

PROGS =	figterm fig_get_shell

PLATFORM := $(shell uname -s)

all: build-figterm fig_get_shell

build-figterm-arm: 
	cargo build --target=aarch64-apple-darwin $(CARGO_FLAGS)
	cp $(ROOT)/target/aarch64-apple-darwin/release/figterm $(BUILT_PRODUCTS_DIR)/figterm-arm

build-figterm-x86: 
	cargo build --target=x86_64-apple-darwin $(CARGO_FLAGS)
	cp $(ROOT)/target/x86_64-apple-darwin/release/figterm $(BUILT_PRODUCTS_DIR)/figterm-x86

fig_get_shell-arm: get_shell.c
	$(CC) get_shell.c $(CFLAGS) -o $(BUILT_PRODUCTS_DIR)/fig_get_shell-arm -target arm64-apple-macos11

fig_get_shell-x86: get_shell.c
	$(CC) get_shell.c $(CFLAGS) -o $(BUILT_PRODUCTS_DIR)/fig_get_shell-x86 -target x86_64-apple-macos10.12

ifeq "$(PLATFORM)" "Darwin"
build-figterm: build-figterm-x86 build-figterm-arm
	lipo -create -output $(BUILT_PRODUCTS_DIR)/figterm $(BUILT_PRODUCTS_DIR)/figterm-x86 $(BUILT_PRODUCTS_DIR)/figterm-arm

fig_get_shell: fig_get_shell-x86 fig_get_shell-arm
	lipo -create -output $(BUILT_PRODUCTS_DIR)/fig_get_shell $(BUILT_PRODUCTS_DIR)/fig_get_shell-x86 $(BUILT_PRODUCTS_DIR)/fig_get_shell-arm
else
build-figterm:
	cargo build $(CARGO_FLAGS)
	cp $(ROOT)/target/release/figterm $(BUILT_PRODUCTS_DIR)/figterm

fig_get_shell: get_shell.c
	$(CC) get_shell.c $(CFLAGS) -o $(BUILT_PRODUCTS_DIR)/fig_get_shell
endif

install: all
	mkdir -p $(HOME)/.fig/bin; \
	cd $(HOME)/.fig/bin && \
		rm -rf $(PROGS) *figterm* && \
		cd $(BUILT_PRODUCTS_DIR) && \
		cp -p $(PROGS) $(HOME)/.fig/bin && \
		cp -p $(HOME)/.fig/bin/figterm $(HOME)/.fig/bin/zsh\ \(figterm\) && \
		cp -p $(HOME)/.fig/bin/figterm $(HOME)/.fig/bin/fish\ \(figterm\) && \
		cp -p $(HOME)/.fig/bin/figterm $(HOME)/.fig/bin/bash\ \(figterm\); \
	# Add fake fig binary on linux or if fig not installed that just logs
	# commands to fig.
	command -v ~/.fig/bin/fig > /dev/null 2>&1 || ( \
		printf "#!/bin/bash\necho \"\$$@\" >> ~/.fig/fig.log" > $(HOME)/.fig/bin/fig && \
		chmod +x $(HOME)/.fig/bin/fig)

clean:
	rm -rf $(ROOT)/target
	rm -rf $(BUILT_PRODUCTS_DIR)