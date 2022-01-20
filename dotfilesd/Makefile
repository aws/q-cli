ROOT:=$(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))

export BUILT_PRODUCTS_DIR ?= $(ROOT)/build

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

BINARY_NAME=dotfilesd

$(shell mkdir -p $(BUILT_PRODUCTS_DIR))

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

macos: build-darwin-x86_64 build-darwin-aarch64 build-darwin-universal

build-darwin-x86_64: 
	cargo build --target=x86_64-apple-darwin --release
	cp $(ROOT)/target/x86_64-apple-darwin/release/$(BINARY_NAME) $(BUILT_PRODUCTS_DIR)/$(BINARY_NAME)-darwin-x86_64

build-darwin-aarch64: 
	cargo build --target=aarch64-apple-darwin --release
	cp $(ROOT)/target/aarch64-apple-darwin/release/$(BINARY_NAME) $(BUILT_PRODUCTS_DIR)/$(BINARY_NAME)-darwin-aarch64

build-darwin-universal: build-darwin-x86_64 build-darwin-aarch64
	lipo -create -output $(BUILT_PRODUCTS_DIR)/$(BINARY_NAME)-darwin-universal $(BUILT_PRODUCTS_DIR)/$(BINARY_NAME)-darwin-x86_64 $(BUILT_PRODUCTS_DIR)/$(BINARY_NAME)-darwin-aarch64

linux: build-linux-x86_64 build-linux-aarch64

build-linux-x86_64: 
	cargo build --target=x86_64-unknown-linux-gnu --release
	cp $(ROOT)/target/x86_64-unknown-linux-gnu/release/$(BINARY_NAME) $(BUILT_PRODUCTS_DIR)/$(BINARY_NAME)-linux-x86_64

build-linux-aarch64: 
	cargo build --target=aarch64-unknown-linux-gnu --release
	cp $(ROOT)/target/aarch64-unknown-linux-gnu/release/$(BINARY_NAME) $(BUILT_PRODUCTS_DIR)/$(BINARY_NAME)-linux-aarch64

windows: build-windows-x86_64

build-windows-x86_64:
	cargo build --target x86_64-pc-windows-gnu --release
	cp $(ROOT)/target/x86_64-pc-windows-gnu/release/$(BINARY_NAME).exe $(BUILT_PRODUCTS_DIR)/$(BINARY_NAME)-windows-x86_64

build-native:
	cargo build --release
	cp $(ROOT)/target/release/$(BINARY_NAME) $(BUILT_PRODUCTS_DIR)/$(BINARY_NAME)

install: build-native
	sudo rm -f /usr/local/bin/$(BINARY_NAME)
	sudo cp $(BUILT_PRODUCTS_DIR)/$(BINARY_NAME) /usr/local/bin/$(BINARY_NAME)

clean:
	cargo clean
	rm -rf build