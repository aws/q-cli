ROOT:=$(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))

export BUILT_PRODUCTS_DIR ?= $(ROOT)

# Add brew executables to PATH
export PATH := /opt/homebrew/bin:/opt/homebrew/sbin:$(PATH)
SHELL := env PATH=$(PATH) /bin/bash

# check if go is installed
GO := $(shell command -v go 2> /dev/null)

all: build-figcli build-callback
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

build-callback-arm:
	GOOS=darwin GOARCH=arm64 go build -o bin/fig_callback-arm ./callback/main.go

build-callback-x86:
	GOOS=darwin GOARCH=amd64 go build -o bin/fig_callback-x86 ./callback/main.go

build-callback: build-callback-arm build-callback-x86
	lipo -create -output $(BUILT_PRODUCTS_DIR)/fig_callback bin/fig_callback-x86 bin/fig_callback-arm
	codesign -s - -v $(BUILT_PRODUCTS_DIR)/fig_callback
else
build-figcli:
	go build -o $(BUILT_PRODUCTS_DIR)/figcli .

build-callback:
	go build -o $(BUILT_PRODUCTS_DIR)/fig_callback ./callback/main.go
endif

clean:
	rm -f $(ROOT)/bin/*
	rm -f $(BUILT_PRODUCTS_DIR)/figcli
	rm -f $(BUILT_PRODUCTS_DIR)/fig_callback

install: all
	mkdir -p ~/.fig/bin
	rm ~/.fig/bin/fig || echo "Could not remove ~/.fig/bin/fig"
	rm ~/.fig/bin/fig_callback || echo "Could not remove ~/.fig/bin/fig_callback"
	cp $(BUILT_PRODUCTS_DIR)/figcli ~/.fig/bin/fig
	cp $(BUILT_PRODUCTS_DIR)/fig_callback ~/.fig/bin/fig_callback

