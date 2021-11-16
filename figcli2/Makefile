ROOT:=$(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))

export BUILT_PRODUCTS_DIR ?= $(ROOT)

# Add brew executables to PATH
export PATH := /opt/homebrew/bin:/opt/homebrew/sbin:$(PATH)
SHELL := env PATH=$(PATH) /bin/bash

# check if go is installed
GO := $(shell command -v go 2> /dev/null)

all: build-figcli
ifndef GO
    $(error "go is not in PATH")
endif

ifeq "$(shell uname -s)" "Darwin"
build-figcli-arm:
	GOOS=darwin GOARCH=arm64 go build -o bin/figcli-arm .

build-figcli-x86:
	GOOS=darwin GOARCH=amd64 go build -o bin/figcli-x86 .

build-figcli: build-figcli-arm build-figcli-x86
	lipo -create -output $(BUILT_PRODUCTS_DIR)/figcli bin/figcli-x86 bin/figcli-arm
	codesign -s - -v $(BUILT_PRODUCTS_DIR)/figcli
else
build-figcli:
	go build -o $(BUILT_PRODUCTS_DIR)/figcli .
endif

clean:
	rm -f $(ROOT)/bin/*
	rm -f $(BUILT_PRODUCTS_DIR)/figcli

install: all
	mkdir -p ~/.fig/bin
	rm ~/.fig/bin/fig || echo "Could not remove ~/.fig/bin/fig"
	cp $(BUILT_PRODUCTS_DIR)/figcli ~/.fig/bin/fig
