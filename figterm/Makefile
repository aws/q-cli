ROOT=$(shell ./realpath.sh)
include $(ROOT)/Makefile.shared

LIBFIG=$(ROOT)/lib/libfig.a
LIBVTERM=$(ROOT)/lib/libvterm.a

PROGS =	figterm fig_get_shell fig_callback

# Add brew executables to PATH
export PATH := /opt/homebrew/bin:/opt/homebrew/sbin:$(PATH)
SHELL := env PATH=$(PATH) /bin/bash

# check if glibtool exists
GLIBTOOL := $(shell command -v glibtool 2> /dev/null)

export BUILT_PRODUCTS_DIR ?= $(ROOT)
all:	$(PROGS)
ifndef GLIBTOOL
    $(error "glibtool is not in PATH")
endif

figterm-arm: main.c figterm.c screen.c util.c color.c history.c $(LIBVTERM) $(LIBFIG)
	$(CC) main.c figterm.c screen.c util.c color.c history.c $(CFLAGS) -o figterm-arm $(LDFLAGS) $(LDLIBS) -target arm64-apple-macos11

figterm-x86: main.c figterm.c screen.c util.c color.c history.c $(LIBVTERM) $(LIBFIG)
	$(CC) main.c figterm.c screen.c util.c color.c history.c $(CFLAGS) -o figterm-x86 $(LDFLAGS) $(LDLIBS) -target x86_64-apple-macos10.12

figterm: figterm-x86 figterm-arm
	lipo -create -output $(BUILT_PRODUCTS_DIR)/figterm figterm-x86 figterm-arm

fig_get_shell-arm: get_shell.c
	$(CC) get_shell.c $(CFLAGS) -o fig_get_shell-arm -target arm64-apple-macos11

fig_get_shell-x86: get_shell.c
	$(CC) get_shell.c $(CFLAGS) -o fig_get_shell-x86 -target x86_64-apple-macos10.12

fig_get_shell: fig_get_shell-x86 fig_get_shell-arm
	lipo -create -output $(BUILT_PRODUCTS_DIR)/fig_get_shell fig_get_shell-x86 fig_get_shell-arm

fig_callback-arm: callback.c util.c $(LIBFIG)
	$(CC) callback.c util.c $(CFLAGS) -o fig_callback-arm $(LDFLAGS) $(LDLIBS) -target arm64-apple-macos11

fig_callback-x86: callback.c util.c $(LIBFIG)
	$(CC) callback.c util.c $(CFLAGS) -o fig_callback-x86 $(LDFLAGS) $(LDLIBS) -target x86_64-apple-macos10.12

fig_callback: fig_callback-x86 fig_callback-arm
	lipo -create -output $(BUILT_PRODUCTS_DIR)/fig_callback fig_callback-x86 fig_callback-arm

install: all
	mkdir -p $(HOME)/.fig/bin; \
	cd $(HOME)/.fig/bin && \
		rm -rf $(PROGS) *figterm* && \
		cd $(BUILT_PRODUCTS_DIR) && \
		cp -p $(PROGS) $(HOME)/.fig/bin && \
		cp -p $(HOME)/.fig/bin/figterm $(HOME)/.fig/bin/zsh\ \(figterm\) && \
		cp -p $(HOME)/.fig/bin/figterm $(HOME)/.fig/bin/fish\ \(figterm\) && \
		cp -p $(HOME)/.fig/bin/figterm $(HOME)/.fig/bin/bash\ \(figterm\); \
	# Add fake fig binary on linux or if fig not installed that just logs
	# commands to fig.
	command -v ~/.fig/bin/fig > /dev/null 2>&1 || ( \
		printf "#!/bin/bash\necho \"\$$@\" >> ~/.fig/fig.log" > $(HOME)/.fig/bin/fig && \
		chmod +x $(HOME)/.fig/bin/fig)

clean:
	rm -f $(PROGS) $(TEMPFILES) *.o *.log; \
	rm -f *-x86 *-arm; \
	rm -rf *.dSYM; \
	cd $(BUILT_PRODUCTS_DIR); \
	rm -f $(PROGS); \
  (cd $(ROOT)/lib && $(MAKE) clean && rm libvterm.*); \
  (cd $(ROOT)/libvterm && $(MAKE) PREFIX=$(ROOT)/libvterm clean && rm -rf lib); \

$(LIBVTERM):
	(cd $(ROOT)/libvterm && $(MAKE) clean && $(MAKE) LIBRARY="libvterm.arm.la" CFLAGS="-target arm64-apple-macos11" PREFIX=$(ROOT)/libvterm install-lib);
	(cd $(ROOT)/libvterm && $(MAKE) clean && $(MAKE) LIBRARY="libvterm.x86.la" CFLAGS="-target x86_64-apple-macos10.12" PREFIX=$(ROOT)/libvterm install-lib);
	lipo -create -output $(ROOT)/libvterm/lib/libvterm.a $(ROOT)/libvterm/lib/libvterm.x86.a $(ROOT)/libvterm/lib/libvterm.arm.a

$(LIBFIG):
	(cd $(ROOT)/lib && $(MAKE))
