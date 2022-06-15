MAKE_DIR    ?= $(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))

BUILD_DIR = $(MAKE_DIR)/build

export BUILT_PRODUCTS_DIR = $(BUILD_DIR)/usr/bin
$(shell mkdir -p $(BUILT_PRODUCTS_DIR))

VERSION = $(shell jq -r .FIG_VERSION $(MAKE_DIR)/bundle/bundle_info.json)
NUMERIC = $(shell echo ${VERSION} | cut -f1 -d-)
FLAVOR = $(shell echo ${VERSION} | cut -f2 -d-)

ifeq (${NUMERIC}, ${FLAVOR})
	FLAVOR = 1
endif

all: archive deb

archive: bin icons bundle 
	cd $(BUILD_DIR) && tar -cjf fig-x86_64-linux.tar.gz usr install.sh

deb: bin icons bundle
	mkdir -p $(BUILD_DIR)/fig-x86_64-linux
	cp -r $(BUILD_DIR)/usr $(BUILD_DIR)/fig-x86_64-linux
	cp $(BUILD_DIR)/fig-x86_64-linux/usr/bin/figterm $(BUILD_DIR)/fig-x86_64-linux/usr/bin/zsh\ \(figterm\)
	cp $(BUILD_DIR)/fig-x86_64-linux/usr/bin/figterm $(BUILD_DIR)/fig-x86_64-linux/usr/bin/bash\ \(figterm\)
	cp $(BUILD_DIR)/fig-x86_64-linux/usr/bin/figterm $(BUILD_DIR)/fig-x86_64-linux/usr/bin/fish\ \(figterm\)
	cp -r $(MAKE_DIR)/bundle/deb/. $(BUILD_DIR)/fig-x86_64-linux
	sed -i "s/^Version:.*/Version: $(VERSION)/" $(BUILD_DIR)/fig-x86_64-linux/DEBIAN/control
	cd $(BUILD_DIR) && dpkg-deb --build --root-owner-group fig-x86_64-linux
	dpkg-deb --info $(BUILD_DIR)/fig-x86_64-linux.deb

rpm: bin icons bundle
	rpmdev-setuptree
	cp $(MAKE_DIR)/bundle/rpm/fig.spec ~/rpmbuild/SPECS/
	sed -i "s/^Version:.*/Version: ${NUMERIC}/" ~/rpmbuild/SPECS/fig.spec
	sed -i "s/^Release:.*/Release: ${FLAVOR}/" ~/rpmbuild/SPECS/fig.spec
	mkdir -p ~/rpmbuild/BUILD/fig-${NUMERIC}-${FLAVOR}/
	cp -r $(BUILD_DIR)/usr ~/rpmbuild/BUILD/fig-${NUMERIC}-${FLAVOR}/
	rpmbuild -bb ~/rpmbuild/SPECS/fig.spec
	cp ~/rpmbuild/RPMS/x86_64/fig-${NUMERIC}-${FLAVOR}.x86_64.rpm $(BUILD_DIR)/fig-x86_64-linux.rpm

bin: fig_ibus_engine fig_desktop fig figterm
	rm -f $(BUILT_PRODUCTS_DIR)/*-x86_64-unknown-linux-gnu

fig_desktop:
	$(MAKE) -C $(MAKE_DIR)/$@

fig: fig_cli

fig_cli:
	$(MAKE) -C $(MAKE_DIR)/$@

figterm:
	$(MAKE) -C $(MAKE_DIR)/$@

fig_ibus_engine: 
	$(MAKE) -C $(MAKE_DIR)/$@

icons:
	for res in 16 22 24 32 48 64 128 256 512; do \
	  install -Dm644 "fig_desktop/icons/$${res}x$${res}.png" \
			"$(BUILD_DIR)/usr/share/icons/hicolor/$${res}x$${res}/apps/fig.png" ; \
	done
	install -Dm644 fig_desktop/icons/512x512.png $(BUILD_DIR)/usr/share/pixmaps/fig.png

bundle:
	cp -r $(MAKE_DIR)/bundle/linux/. $(BUILD_DIR)

# Actions

preview: archive
	tar -tvf $(BUILD_DIR)/fig-x86_64-linux.tar.gz

.PHONY: all archive deb bin fig_desktop figterm fig fig_cli fig_ibus_engine icons preview bundle

