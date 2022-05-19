MAKE_DIR    ?= $(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))

export BUILT_PRODUCTS_DIR = "$(MAKE_DIR)/build"
$(shell mkdir -p $(BUILT_PRODUCTS_DIR))

all: archive

archive: fig_desktop fig figterm fig_ibus_engine
	cd $(BUILT_PRODUCTS_DIR) &&	tar cjf fig-x86_64-linux.tar.gz $^

fig_desktop:
	$(MAKE) -C $(MAKE_DIR)/fig_tauri/src-tauri

fig:
	$(MAKE) -C $(MAKE_DIR)/$@

figterm:
	$(MAKE) -C $(MAKE_DIR)/$@

fig_ibus_engine:
	$(MAKE) -C $(MAKE_DIR)/$@

.PHONY: all figterm fig_cli fig_ibus_engine
