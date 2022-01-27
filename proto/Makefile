ROOT:=$(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))

export BUILT_PRODUCTS_DIR ?= $(ROOT)


# Add brew executables to PATH
export PATH := /opt/homebrew/bin:/opt/homebrew/sbin:$(PATH)

# Add go executables to PATH
export PATH := $(shell go env GOPATH)/bin:$(PATH)

SHELL := env PATH=$(PATH) /bin/bash

# Flags
FLAGS       = --experimental_allow_proto3_optional 
TS_FLAGS    = --ts_proto_opt=esModuleInterop=true --ts_proto_opt=oneof=unions --ts_proto_opt=fileSuffix=.pb
SWIFT_FLAGS = --swift_opt=Visibility=Public

# Proto files
LOCAL = "./local.proto"
API   = "./fig.proto" "./local.proto"
FIGTERM = "./figterm.proto"

# Destinations
ARTIFACTS          = "./bin"
TS_API_BINDINGS    = $(ROOT)/../typescript-api-bindings/src
SWIFT_API_BINDINGS = $(ROOT)/../swift-api-bindings/Sources/FigAPIBindings
GO_CLI             = $(ROOT)/../figcli/fig-proto
DOTENV             ?= $(ROOT)/../dotenv/src

DESTINATIONS       = $(ARTIFACTS) \
					 $(TS_API_BINDINGS) \
					 $(SWIFT_API_BINDINGS) \
					 $(GO_CLI) \
					 $(DOTENV)

# Ensure $ARTIFACTS directory exists
$(shell mkdir -p $(ARTIFACTS))

all: api local figterm

check-protoc:
	command -v protoc 2> /dev/null || echo "protoc is not in PATH"

check-swift: check-protoc
	command -v protoc-gen-swift 2> /dev/null || echo "protoc-gen-swift is not in PATH"

check-go: check-protoc
	command -v protoc-gen-go 2> /dev/null || echo "protoc-gen-go is not in PATH"

check-ts: check-protoc
	command -v protoc-gen-ts_proto 2> /dev/null || echo "protoc-gen-ts_proto is not in PATH"

api-swift: check-swift
	protoc $(FLAGS) $(SWIFT_FLAGS) --swift_out=$(ARTIFACTS) $(API)
	mkdir -p $(SWIFT_API_BINDINGS)
	cp $(ARTIFACTS)/*.pb.swift $(SWIFT_API_BINDINGS)

api-ts: check-ts
	protoc $(FLAGS) $(TS_FLAGS) --ts_proto_out=$(ARTIFACTS) $(API)
	mkdir -p $(TS_API_BINDINGS)
	cp $(ARTIFACTS)/*.pb.ts $(TS_API_BINDINGS)

api: api-swift api-ts

local-go: check-go
	protoc $(FLAGS) \
		--go_opt=paths=source_relative \
	   	--go_opt=Mlocal.proto="." \
	   	--go_out=$(ARTIFACTS) $(LOCAL)
	mkdir -p $(GO_CLI)
	cp $(ARTIFACTS)/local.pb.go $(GO_CLI)

local-ts: check-ts
	protoc $(FLAGS) $(TS_FLAGS) --ts_proto_out=$(ARTIFACTS) $(LOCAL)
	mkdir -p $(DOTENV)
	cp $(ARTIFACTS)/local.pb.ts $(DOTENV)

local: local-go local-ts

figterm-swift:
	protoc $(FLAGS) $(SWIFT_FLAGS) --swift_out=$(ARTIFACTS) $(FIGTERM)
	mkdir -p $(SWIFT_API_BINDINGS)
	cp $(ARTIFACTS)/*.pb.swift $(SWIFT_API_BINDINGS)

figterm: figterm-swift

clean: 
	# Remove compiled protos from all destinations, including $ARTIFACTS
	$(foreach destination,$(DESTINATIONS),rm $(destination)/*.pb.* ;)
