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

BINARY_NAME=dotfiles

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

macos: build-darwin-universal

build-x86_64-apple-darwin: 
	cargo build --target=x86_64-apple-darwin --release
	cp $(ROOT)/target/x86_64-apple-darwin/release/$(BINARY_NAME) $(BUILT_PRODUCTS_DIR)/$(BINARY_NAME)-x86_64-apple-darwin

build-aarch64-apple-darwin: 
	cargo build --target=aarch64-apple-darwin --release
	cp $(ROOT)/target/aarch64-apple-darwin/release/$(BINARY_NAME) $(BUILT_PRODUCTS_DIR)/$(BINARY_NAME)-aarch64-apple-darwin

build-darwin-universal: build-x86_64-apple-darwin build-aarch64-apple-darwin
	lipo -create -output $(BUILT_PRODUCTS_DIR)/$(BINARY_NAME)-darwin-universal $(BUILT_PRODUCTS_DIR)/$(BINARY_NAME)-x86_64-apple-darwin $(BUILT_PRODUCTS_DIR)/$(BINARY_NAME)-aarch64-apple-darwin

linux: build-x86_64-unknown-linux-gnu build-aarch64-unknown-linux-gnu

build-x86_64-unknown-linux-gnu: 
	cargo build --target=x86_64-unknown-linux-gnu --release
	cp $(ROOT)/target/x86_64-unknown-linux-gnu/release/$(BINARY_NAME) $(BUILT_PRODUCTS_DIR)/$(BINARY_NAME)-x86_64-unknown-linux-gnu

build-aarch64-unknown-linux-gnu: 
	cargo build --target=aarch64-unknown-linux-gnu --release
	cp $(ROOT)/target/aarch64-unknown-linux-gnu/release/$(BINARY_NAME) $(BUILT_PRODUCTS_DIR)/$(BINARY_NAME)-aarch64-unknown-linux-gnu

windows: build-x86_64-pc-windows-gnu

build-x86_64-pc-windows-gnu:
	cargo build --target x86_64-pc-windows-gnu --release
	cp $(ROOT)/target/x86_64-pc-windows-gnu/release/$(BINARY_NAME).exe $(BUILT_PRODUCTS_DIR)/$(BINARY_NAME)-x86_64-pc-windows-gnu

build-native:
	cargo build --release
	cp $(ROOT)/target/release/$(BINARY_NAME) $(BUILT_PRODUCTS_DIR)/$(BINARY_NAME)

install: build-native
	sudo rm -f /usr/local/bin/$(BINARY_NAME)
	sudo cp $(BUILT_PRODUCTS_DIR)/$(BINARY_NAME) /usr/local/bin/$(BINARY_NAME)

clean:
	cargo clean
	rm -rf build