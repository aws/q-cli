ROOT:=$(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))

export BUILT_PRODUCTS_DIR ?= $(ROOT)/build
export ARTIFACTS ?= $(ROOT)/build

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
$(shell mkdir -p $(ARTIFACTS))

PROGS =	figterm fig_get_shell
SRC = Cargo.toml $(wildcard figterm/**/*) $(wildcard alacritty_terminal/**/*) $(wildcard  fig-color/**/*)

PLATFORM := $(shell uname -s)

all: figterm fig_get_shell

$(ARTIFACTS)/figterm-aarch64: $(SRC)
	cargo build --target=aarch64-apple-darwin $(CARGO_FLAGS)
	mkdir -p $(dir $@)
	cp $(ROOT)/target/aarch64-apple-darwin/release/figterm $@

$(ARTIFACTS)/figterm-x86_64: $(SRC)
	cargo build --target=x86_64-apple-darwin $(CARGO_FLAGS)
	mkdir -p $(dir $@)
	cp $(ROOT)/target/x86_64-apple-darwin/release/figterm $@

$(ARTIFACTS)/fig_get_shell-aarch64: get_shell.c
	mkdir -p $(dir $@)
	$(CC) get_shell.c $(CFLAGS) -o $@ -target arm64-apple-macos11

$(ARTIFACTS)/fig_get_shell-x86_64: get_shell.c
	mkdir -p $(dir $@)
	$(CC) get_shell.c $(CFLAGS) -o $@ -target x86_64-apple-macos10.12

figterm-x86_64: $(ARTIFACTS)/figterm-x86_64
	cp $^ $(BUILT_PRODUCTS_DIR)
figterm-aarch64: $(ARTIFACTS)/figterm-aarch64
	cp $^ $(BUILT_PRODUCTS_DIR)
figterm: $(ARTIFACTS)/figterm
	cp $^ $(BUILT_PRODUCTS_DIR)

fig_get_shell-x86_64: $(ARTIFACTS)/fig_get_shell-x86_64
	cp $^ $(BUILT_PRODUCTS_DIR)
fig_get_shell-aarch64: $(ARTIFACTS)/fig_get_shell-aarch64
	cp $^ $(BUILT_PRODUCTS_DIR)
fig_get_shell: $(ARTIFACTS)/fig_get_shell
	cp $^ $(BUILT_PRODUCTS_DIR)


ifeq "$(PLATFORM)" "Darwin"
$(ARTIFACTS)/figterm: $(ARTIFACTS)/figterm-x86_64 $(ARTIFACTS)/figterm-aarch64
	lipo -create -output $(ARTIFACTS)/figterm $(ARTIFACTS)/figterm-x86_64 $(ARTIFACTS)/figterm-aarch64

$(ARTIFACTS)/fig_get_shell: $(ARTIFACTS)/fig_get_shell-x86_64 $(ARTIFACTS)/fig_get_shell-aarch64
	lipo -create -output $(ARTIFACTS)/fig_get_shell $(ARTIFACTS)/fig_get_shell-x86_64 $(ARTIFACTS)/fig_get_shell-aarch64
else
$(ARTIFACTS)/figterm:
	cargo build $(CARGO_FLAGS)
	cp $(ROOT)/target/release/figterm $(ARTIFACTS)/figterm

$(ARTIFACTS)/fig_get_shell: get_shell.c
	$(CC) get_shell.c $(CFLAGS) -o $(ARTIFACTS)/fig_get_shell
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
	rm -rf $(ARTIFACTS)
	rm -rf $(BUILT_PRODUCTS_DIR)