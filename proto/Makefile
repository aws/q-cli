ROOT:=$(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))

export BUILT_PRODUCTS_DIR ?= $(ROOT)


# Add brew executables to PATH
export PATH := /opt/homebrew/bin:/opt/homebrew/sbin:$(PATH)

# Add go executables to PATH
export PATH := $(shell go env GOPATH)/bin:$(PATH)

SHELL := env PATH=$(PATH) /bin/bash


# check if protoc and language-specific plugins are installed
PROTOC := $(shell command -v protoc 2> /dev/null)
TS     := $(shell command -v protoc-gen-ts_proto 2> /dev/null)
SWIFT  := $(shell command -v protoc-gen-swift 2> /dev/null)
GO     := $(shell command -v protoc-gen-go 2> /dev/null)


# Flags
FLAGS       = --experimental_allow_proto3_optional 
TS_FLAGS    = --ts_proto_opt=esModuleInterop=true --ts_proto_opt=oneof=unions --ts_proto_opt=fileSuffix=.pb
SWIFT_FLAGS = --swift_opt=Visibility=Public

# Proto files
LOCAL = "./local.proto"
API   = "./fig.proto" "./local.proto"

# Destinations
ARTIFACTS          = "./bin"
TS_API_BINDINGS    = $(ROOT)/../typescript-api-bindings/src
SWIFT_API_BINDINGS = $(ROOT)/../swift-api-bindings/Sources/FigAPIBindings
GO_CLI             = $(ROOT)/../figcli/fig-proto
DOTENV             = $(ROOT)/../dotenv/src

DESTINATIONS       = $(ARTIFACTS) \
					 $(TS_API_BINDINGS) \
					 $(SWIFT_API_BINDINGS) \
					 $(GO_CLI) \
					 $(DOTENV)

# Ensure $ARTIFACTS directory exists
$(shell mkdir -p $(ARTIFACTS))

all: api local
ifndef PROTOC
    $(error "protoc is not in PATH")
endif

ifndef TS
    $(error "protoc-gen-ts_proto is not in PATH")
endif

ifndef SWIFT
    $(error "protoc-gen-swift is not in PATH")
endif

ifndef GO
    $(error "protoc-gen-go is not in PATH")
endif

api-swift:
	protoc $(FLAGS) $(SWIFT_FLAGS) --swift_out=$(ARTIFACTS) $(API)

api-ts:
	protoc $(FLAGS) $(TS_FLAGS) --ts_proto_out=$(ARTIFACTS) $(API)

local-go:
	protoc $(FLAGS) \
		--go_opt=paths=source_relative \
	   	--go_opt=Mlocal.proto="." \
	   	--go_out=$(ARTIFACTS) $(LOCAL)

local-ts:
	protoc $(FLAGS) $(TS_FLAGS) --ts_proto_out=$(ARTIFACTS) $(LOCAL)

api: api-swift api-ts
	cp $(ARTIFACTS)/*.pb.ts $(TS_API_BINDINGS)
	cp $(ARTIFACTS)/*.pb.swift $(SWIFT_API_BINDINGS)


local: local-go local-ts
	cp $(ARTIFACTS)/local.pb.ts $(DOTENV)
	cp $(ARTIFACTS)/local.pb.go $(GO_CLI)

clean: 
	# Remove compiled protos from all destinations, including $ARTIFACTS
	$(foreach destination,$(DESTINATIONS),rm $(destination)/*.pb.* ;)
