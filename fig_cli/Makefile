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
# Add brew executables to PATH
export PATH := /opt/homebrew/bin:/opt/homebrew/sbin:$(PATH)
endif

# Add cargo executables to PATH
export PATH := $(HOME)/.cargo/bin:$(PATH)

MAKE_DIR    ?= $(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))
TARGET_DIR  ?= $(shell cargo metadata --format-version 1 | jq -r .target_directory)

BUILT_PRODUCTS_DIR ?= $(MAKE_DIR)/build
OUT_DIR            := $(BUILT_PRODUCTS_DIR)
$(shell mkdir -p $(OUT_DIR))

SHELL := env PATH=$(PATH) /bin/bash

CARGO_OPTIONS ?= --locked --release

BINARY_NAME ?= fig
CRATE_NAME  ?= fig_cli

### Build for each platform

ifeq "$(PLATFORM)" "Darwin"
all: macos
endif
ifeq "$(PLATFORM)" "Linux"
all: linux
endif
ifeq "$(PLATFORM)" "windows"
all: windows
endif

macos: $(BINARY_NAME)-darwin-universal
	install $(OUT_DIR)/$< $(OUT_DIR)/$(BINARY_NAME)

$(BINARY_NAME)-x86_64-apple-darwin:
	cargo build --target=x86_64-apple-darwin $(CARGO_OPTIONS)
	install $(TARGET_DIR)/x86_64-apple-darwin/release/$(CRATE_NAME) $(OUT_DIR)/$@

$(BINARY_NAME)-aarch64-apple-darwin:
	cargo build --target=aarch64-apple-darwin $(CARGO_OPTIONS)
	install $(TARGET_DIR)/aarch64-apple-darwin/release/$(CRATE_NAME) $(OUT_DIR)/$@

$(BINARY_NAME)-darwin-universal: $(BINARY_NAME)-x86_64-apple-darwin $(BINARY_NAME)-aarch64-apple-darwin
	lipo -create -output $(OUT_DIR)/$@ $(OUT_DIR)/$(BINARY_NAME)-x86_64-apple-darwin $(OUT_DIR)/$(BINARY_NAME)-aarch64-apple-darwin

linux: $(BINARY_NAME)-x86_64-unknown-linux-gnu
	install $(OUT_DIR)/$< $(OUT_DIR)/$(BINARY_NAME)

$(BINARY_NAME)-x86_64-unknown-linux-gnu:
	cargo build --target=x86_64-unknown-linux-gnu $(CARGO_OPTIONS)
	install $(TARGET_DIR)/x86_64-unknown-linux-gnu/release/$(CRATE_NAME) $(OUT_DIR)/$@

$(BINARY_NAME)-aarch64-unknown-linux-gnu:
	cargo build --target=aarch64-unknown-linux-gnu $(CARGO_OPTIONS)
	install $(TARGET_DIR)/aarch64-unknown-linux-gnu/release/$(CRATE_NAME) $(OUT_DIR)/$@

windows: $(BINARY_NAME)-x86_64-pc-windows-gnu
	install $(OUT_DIR)/$< $(OUT_DIR)/$(BINARY_NAME)

$(BINARY_NAME)-x86_64-pc-windows-gnu:
	cargo build --target x86_64-pc-windows-gnu $(CARGO_OPTIONS)
	install $(TARGET_DIR)/x86_64-pc-windows-gnu/release/$(CRATE_NAME).exe $(OUT_DIR)/$@

### Build and Install for local dev

build-native:
	cargo build

build-native-release:
	cargo build --release

install-native: build-native
	mkdir -p $(HOME)/.local/bin/
	install $(TARGET_DIR)/debug/$(CRATE_NAME) $(HOME)/.local/bin/$(BINARY_NAME)

install-native-usr: build-native
	sudo install $(TARGET_DIR)/debug/$(CRATE_NAME) /opt/fig/bin/$(BINARY_NAME)

install-native-release: build-native-release
	mkdir -p $(HOME)/.local/bin/
	install $(TARGET_DIR)/release/$(CRATE_NAME) $(HOME)/.local/bin/$(BINARY_NAME)

### Generate Fig compleation spec

generate-spec: install-native
	$(HOME)/.local/bin/$(BINARY_NAME) completion fig > spec.ts

### Clean the project

clean:
	command -v cargo &>/dev/null && cargo clean
	rm -rf $(BUILT_PRODUCTS_DIR)

### Install for xcode
install: all

