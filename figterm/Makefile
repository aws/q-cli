FIGTERM_DIR:=$(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))
MACOS_DIR:=$(shell dirname $(FIGTERM_DIR))

export BUILT_PRODUCTS_DIR ?= $(FIGTERM_DIR)/build
export ARTIFACTS ?= $(FIGTERM_DIR)/build

TARGET_DIR ?= $(MACOS_DIR)/target

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

SRC := $(shell find $(FIGTERM_DIR)/src -name '*.rs' -o -name '*.toml')\
	   $(shell find $(MACOS_DIR)/rust-lib -name '*.rs' -o -name '*.toml' -o -name '*.c' -o -name '*.h')\
	   $(shell find $(MACOS_DIR)/proto -name '*.proto')\
	   $(MACOS_DIR)/Cargo.toml $(MACOS_DIR)/Cargo.lock

PLATFORM := $(shell uname -s)

all: figterm 

$(ARTIFACTS)/figterm-aarch64: $(SRC)
	cargo build --target=aarch64-apple-darwin $(CARGO_FLAGS)
	mkdir -p $(dir $@)
	install $(TARGET_DIR)/aarch64-apple-darwin/release/figterm $@

$(ARTIFACTS)/figterm-x86_64: $(SRC)
	cargo build --target=x86_64-apple-darwin $(CARGO_FLAGS)
	mkdir -p $(dir $@)
	install $(TARGET_DIR)/x86_64-apple-darwin/release/figterm $@

figterm-x86_64: $(ARTIFACTS)/figterm-x86_64
	install $^ $(BUILT_PRODUCTS_DIR)
figterm-aarch64: $(ARTIFACTS)/figterm-aarch64
	install $^ $(BUILT_PRODUCTS_DIR)
figterm: $(ARTIFACTS)/figterm
	install $^ $(BUILT_PRODUCTS_DIR)

ifeq "$(PLATFORM)" "Darwin"
$(ARTIFACTS)/figterm: $(ARTIFACTS)/figterm-x86_64 $(ARTIFACTS)/figterm-aarch64
	lipo -create -output $(ARTIFACTS)/figterm $(ARTIFACTS)/figterm-x86_64 $(ARTIFACTS)/figterm-aarch64
else
$(ARTIFACTS)/figterm:
	cargo build $(CARGO_FLAGS)
	install $(TARGET_DIR)/release/figterm $(ARTIFACTS)/figterm
endif

install: all
	mkdir -p $(HOME)/.fig/bin; \
	cd $(HOME)/.fig/bin && \
		rm -rf $(PROGS) *figterm* && \
		cd $(BUILT_PRODUCTS_DIR) && \
		install -p $(PROGS) $(HOME)/.fig/bin && \
		install -p $(HOME)/.fig/bin/figterm $(HOME)/.fig/bin/zsh\ \(figterm\) && \
		install -p $(HOME)/.fig/bin/figterm $(HOME)/.fig/bin/fish\ \(figterm\) && \
		install -p $(HOME)/.fig/bin/figterm $(HOME)/.fig/bin/bash\ \(figterm\); \
	# Add fake fig binary on linux or if fig not installed that just logs
	# commands to fig.
	command -v ~/.fig/bin/fig > /dev/null 2>&1 || ( \
		printf "#!/bin/bash\necho \"\$$@\" >> ~/.fig/fig.log" > $(HOME)/.fig/bin/fig && \
		chmod +x $(HOME)/.fig/bin/fig)

clean:
	command -v cargo &>/dev/null && cargo clean
	rm -rf $(TARGET_DIR)
	rm -rf $(ARTIFACTS)
	rm -rf $(BUILT_PRODUCTS_DIR)