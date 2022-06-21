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

ifneq "$(PLATFORM)" "Linux"
	exit 1
endif

# Add cargo executables to PATH
export PATH := $(HOME)/.cargo/bin:$(PATH)

# Check if cargo exists
CARGO := $(shell command -v cargo 2> /dev/null)

ifndef CARGO
    $(error "cargo is not in PATH $(PATH)")
endif

MAKE_DIR    ?= $(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))
TARGET_DIR  ?= $(shell cargo metadata --format-version 1 | jq -r .target_directory)

BUILT_PRODUCTS_DIR ?= $(MAKE_DIR)/build
OUT_DIR            := $(BUILT_PRODUCTS_DIR)
$(shell mkdir -p $(OUT_DIR))

SHELL := env PATH=$(PATH) /bin/bash

CARGO_OPTIONS ?= --locked --release

BINARY_NAME ?= fig_ibus_engine
CRATE_NAME  ?= fig_ibus_engine

### Build

linux: $(BINARY_NAME)-x86_64-unknown-linux-gnu
	install $(OUT_DIR)/$< $(OUT_DIR)/$(BINARY_NAME)

$(BINARY_NAME)-x86_64-unknown-linux-gnu:
	cargo build --target=x86_64-unknown-linux-gnu $(CARGO_OPTIONS)
	install $(TARGET_DIR)/x86_64-unknown-linux-gnu/release/$(CRATE_NAME) $(OUT_DIR)/$@

$(BINARY_NAME)-aarch64-unknown-linux-gnu:
	cargo build --target=aarch64-unknown-linux-gnu $(CARGO_OPTIONS)
	install $(TARGET_DIR)/aarch64-unknown-linux-gnu/release/$(CRATE_NAME) $(OUT_DIR)/$@

### Clean the project

clean:
	command -v cargo &>/dev/null && cargo clean
	rm -rf $(BUILT_PRODUCTS_DIR)

