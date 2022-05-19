MAKE_DIR    ?= $(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))

export BUILT_PRODUCTS_DIR = "$(MAKE_DIR)/build"
$(shell mkdir -p $(BUILT_PRODUCTS_DIR))

all: fig_cli figterm

figterm:
	$(MAKE) -C $@

fig_cli:
	$(MAKE) -C $@

.PHONY: all figterm fig_cli
