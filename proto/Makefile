ROOT:=$(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))

export BUILT_PRODUCTS_DIR ?= $(ROOT)

# Add brew executables to PATH
export PATH := /opt/homebrew/bin:/opt/homebrew/sbin:$(PATH)

SHELL := env "PATH=$(PATH)" bash

TS_PROTO = $(shell pnpm bin)/protoc-gen-ts_proto

# Flags
FLAGS       = --experimental_allow_proto3_optional 
TS_FLAGS    = --plugin=$(TS_PROTO) \
              --ts_proto_opt=esModuleInterop=true \
              --ts_proto_opt=oneof=unions \
              --ts_proto_opt=fileSuffix=.pb \
              --ts_proto_opt=removeEnumPrefix=true

# Proto files
LOCAL = "./local.proto" "./fig_common.proto"
API   = "./fig.proto" "./local.proto" "./fig_common.proto"
FIGTERM = "./figterm.proto"
remote = "./remote.proto"

# Destinations
ARTIFACTS           = bin
ARTIFACTS_TS        = bin/ts
TS_API_BINDINGS     = $(ROOT)/../typescript-api-bindings/src
DOTENV             ?= $(ROOT)/../tests/dotenv/src

DESTINATIONS = $(ARTIFACTS) \
					     $(TS_API_BINDINGS) \
					     $(DOTENV)

# Ensure $ARTIFACTS directory exists
$(shell mkdir -p $(ARTIFACTS))
$(shell mkdir -p $(ARTIFACTS_TS))

all: api local

check-protoc:
	command -v protoc 2> /dev/null || echo "protoc is not in PATH"

check-ts: check-protoc
	command -v $(TS_PROTO) 2> /dev/null || echo "protoc-gen-ts_proto is not in PATH"

api-ts: check-ts
	protoc $(FLAGS) $(TS_FLAGS) --ts_proto_out=$(ARTIFACTS_TS) $(API)
	mkdir -p $(TS_API_BINDINGS)
	cp -r $(ARTIFACTS_TS)/** $(TS_API_BINDINGS)

api: api-ts

local-ts: check-ts
	protoc $(FLAGS) $(TS_FLAGS) --ts_proto_out=$(ARTIFACTS) $(LOCAL)
	mkdir -p $(DOTENV)
	cp $(ARTIFACTS)/local.pb.ts $(DOTENV)

local: local-ts

clean: 
	# Remove compiled protos from all destinations, including $ARTIFACTS
	$(foreach destination,$(DESTINATIONS),rm $(destination)/*.pb.* ;)
