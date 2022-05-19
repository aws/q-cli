MAKE_DIR    ?= $(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))

BUILD_DIR = $(MAKE_DIR)/build

export BUILT_PRODUCTS_DIR = $(BUILD_DIR)/usr/bin
$(shell mkdir -p $(BUILT_PRODUCTS_DIR))

VERSION = $(shell jq -r .FIG_VERSION $(MAKE_DIR)/bundle/bundle_info.json)

all: archive deb

archive: bin icons bundle 
	cd $(BUILD_DIR) && tar -cjf fig-x86_64-linux.tar.gz usr install.sh

deb: bin icons bundle
	mkdir -p $(BUILD_DIR)/fig-x86_64-linux
	cp -r $(BUILD_DIR)/usr $(BUILD_DIR)/fig-x86_64-linux
	cp -r $(MAKE_DIR)/bundle/deb/. $(BUILD_DIR)/fig-x86_64-linux
	sed -i "s/^Version:.*/Version: $(VERSION)/" $(BUILD_DIR)/fig-x86_64-linux/DEBIAN/control
	cd $(BUILD_DIR) && dpkg-deb --build --root-owner-group fig-x86_64-linux
	dpkg-deb --info $(BUILD_DIR)/fig-x86_64-linux.deb

bin: fig_ibus_engine fig_desktop fig figterm
	rm -f $(BUILT_PRODUCTS_DIR)/*-x86_64-unknown-linux-gnu

fig_desktop:
	$(MAKE) -C $(MAKE_DIR)/fig_tauri/src-tauri

fig: fig_cli

fig_cli:
	$(MAKE) -C $(MAKE_DIR)/$@

figterm:
	$(MAKE) -C $(MAKE_DIR)/$@

fig_ibus_engine: 
	$(MAKE) -C $(MAKE_DIR)/$@

icons:
	install -Dm644 fig_tauri/src-tauri/icons/32x32.png      $(BUILD_DIR)/usr/share/icons/hicolor/32x32/apps/fig.png
	install -Dm644 fig_tauri/src-tauri/icons/128x128.png    $(BUILD_DIR)/usr/share/icons/hicolor/128x128/apps/fig.png
	install -Dm644 fig_tauri/src-tauri/icons/128x128@2x.png $(BUILD_DIR)/usr/share/icons/hicolor/256x256/apps/fig.png
	install -Dm644 fig_tauri/src-tauri/icons/icon.png       $(BUILD_DIR)/usr/share/icons/hicolor/512x512/apps/fig.png
	install -Dm644 fig_tauri/src-tauri/icons/icon.png       $(BUILD_DIR)/usr/share/pixmaps/fig.png

bundle:
	cp -r $(MAKE_DIR)/bundle/linux/. $(BUILD_DIR)

# Actions

preview: archive
	tar -tvf $(BUILD_DIR)/fig-x86_64-linux.tar.gz

.PHONY: all archive deb bin fig_desktop figterm fig fig_cli fig_ibus_engine icons preview bundle

