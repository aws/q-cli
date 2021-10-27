ROOT:=$(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))

export BUILT_PRODUCTS_DIR ?= $(ROOT)

# Add brew executables to PATH
export PATH := /opt/homebrew/bin:/opt/homebrew/sbin:$(PATH)
SHELL := env PATH=$(PATH) /bin/bash

# check if go is installed
GO := $(shell command -v go 2> /dev/null)

all: figcli
ifndef GO
    $(error "go is not in PATH")
endif

figcli-arm:
	GOOS=darwin GOARCH=arm64 go build -o bin/figcli-arm .

figcli-x86:
	GOOS=darwin GOARCH=amd64 go build -o bin/figcli-x86 .

figcli: figcli-arm figcli-x86
	lipo -create -output $(BUILT_PRODUCTS_DIR)/figcli bin/figcli-x86 bin/figcli-arm

clean:
	rm $(ROOT)/bin/*
	rm $(BUILT_PRODUCTS_DIR)/figcli
