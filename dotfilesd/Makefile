ROOT:=$(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))

export BUILT_PRODUCTS_DIR ?= $(ROOT)/build
export ARTIFACTS ?= $(ROOT)/build

TARGET_DIR ?= $(ROOT)/../target

# Add brew executables to PATH
export PATH := /opt/homebrew/bin:/opt/homebrew/sbin:$(PATH)
# Add cargo executables to PATH
export PATH := $(HOME)/.cargo/bin:$(PATH)

SHELL := env PATH=$(PATH) /bin/bash

# check if cargo exists
CARGO := $(shell command -v cargo 2> /dev/null)

ifndef CARGO
    $(error "cargo is not in PATH $(PATH)")
endif

CARGO_OPTIONS ?= --locked --release

BINARY_NAME=dotfilesd
SRC = Cargo.toml $(wildcard src/**/*)


$(shell mkdir -p $(BUILT_PRODUCTS_DIR))
$(shell mkdir -p $(ARTIFACTS))

PLATFORM := $(shell uname -s)

# Check if variable contains string https://stackoverflow.com/a/2741747
ifneq (,$(findstring CYGWIN,$(PLATFORM)))
	PLATFORM := "windows"
endif
ifneq (,$(findstring MINGW,$(PLATFORM)))
	PLATFORM := "windows"
endif
ifneq (,$(findstring MSYS,$(PLATFORM)))
	PLATFORM := "windows"
endif

$(warning PLATFORM is $(PLATFORM))

ifeq "$(PLATFORM)" "Darwin"
all: macos
endif
ifeq "$(PLATFORM)" "Linux"
all: linux
endif
ifeq "$(PLATFORM)" "windows"
all: windows
endif

macos: $(ARTIFACTS)/$(BINARY_NAME)-darwin-universal
	install $^ $(BUILT_PRODUCTS_DIR)

build-x86_64-apple-darwin: $(ARTIFACTS)/$(BINARY_NAME)-x86_64-apple-darwin
	install $^ $(BUILT_PRODUCTS_DIR)

build-aarch64-apple-darwin: $(ARTIFACTS)/$(BINARY_NAME)-aarch64-apple-darwin
	install $^ $(BUILT_PRODUCTS_DIR)

build-x86_64-unknown-linux-gnu: $(ARTIFACTS)/$(BINARY_NAME)-x86_64-unknown-linux-gnu
	install $^ $(BUILT_PRODUCTS_DIR)

$(ARTIFACTS)/$(BINARY_NAME)-x86_64-apple-darwin: $(SRC)
	cargo build --target=x86_64-apple-darwin $(CARGO_OPTIONS)
	install $(TARGET_DIR)/x86_64-apple-darwin/release/$(BINARY_NAME) $@

$(ARTIFACTS)/$(BINARY_NAME)-aarch64-apple-darwin: $(SRC)
	cargo build --target=aarch64-apple-darwin $(CARGO_OPTIONS)
	install $(TARGET_DIR)/aarch64-apple-darwin/release/$(BINARY_NAME) $@

$(ARTIFACTS)/$(BINARY_NAME)-darwin-universal: $(ARTIFACTS)/$(BINARY_NAME)-x86_64-apple-darwin $(ARTIFACTS)/$(BINARY_NAME)-aarch64-apple-darwin
	lipo -create -output $@ $^
linux: $(ARTIFACTS)/$(BINARY_NAME)-x86_64-unknown-linux-gnu $(ARTIFACTS)/$(BINARY_NAME)-aarch64-unknown-linux-gnu

$(ARTIFACTS)/$(BINARY_NAME)-x86_64-unknown-linux-gnu: $(SRC)
	cargo build --target=x86_64-unknown-linux-gnu $(CARGO_OPTIONS)
	install $(TARGET_DIR)/x86_64-unknown-linux-gnu/release/$(BINARY_NAME) $@

$(ARTIFACTS)/$(BINARY_NAME)-aarch64-unknown-linux-gnu: $(SRC)
	cargo build --target=aarch64-unknown-linux-gnu $(CARGO_OPTIONS)
	install $(TARGET_DIR)/aarch64-unknown-linux-gnu/release/$(BINARY_NAME) $@

windows: $(ARTIFACTS)/$(BINARY_NAME)-x86_64-pc-windows-gnu

$(ARTIFACTS)/$(BINARY_NAME)-x86_64-pc-windows-gnu: $(SRC)
	cargo build --target x86_64-pc-windows-gnu $(CARGO_OPTIONS)
	install $(TARGET_DIR)/x86_64-pc-windows-gnu/release/$(BINARY_NAME).exe $@

build-native:
	cargo build
	install $(TARGET_DIR)/debug/$(BINARY_NAME) $(ARTIFACTS)/$(BINARY_NAME)

install: all

install-native: build-native
	mkdir -p ~/.local/bin/
	install $(ARTIFACTS)/$(BINARY_NAME) ~/.local/bin/$(BINARY_NAME)

clean:
	command -v cargo &>/dev/null && cargo clean
	rm -rf $(TARGET_DIR)
	rm -rf $(ARTIFACTS)
	rm -rf $(BUILT_PRODUCTS_DIR)
