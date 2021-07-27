ROOT=$(shell ./realpath.sh)
include $(ROOT)/Makefile.shared

LIBFIG=$(ROOT)/lib/libfig.a
LIBVTERM=$(ROOT)/lib/libvterm.a

PROGS =	figterm fig_get_shell fig_callback

all:	$(PROGS)

figterm-arm: main.c figterm.c screen.c util.c $(LIBVTERM) $(LIBFIG)
	$(CC) main.c figterm.c screen.c util.c $(CFLAGS) -o figterm-arm $(LDFLAGS) $(LDLIBS) -target arm64-apple-macos11

figterm-x86: main.c figterm.c screen.c util.c $(LIBVTERM) $(LIBFIG)
	$(CC) main.c figterm.c screen.c util.c $(CFLAGS) -o figterm-x86 $(LDFLAGS) $(LDLIBS) -target x86_64-apple-macos10.12

figterm: figterm-x86 figterm-arm
	lipo -create -output figterm figterm-x86 figterm-arm

fig_get_shell_arm: get_shell.c
	$(CC) get_shell.c $(CFLAGS) -o fig_get_shell_arm -target arm64-apple-macos11

fig_get_shell_x86: get_shell.c
	$(CC) get_shell.c $(CFLAGS) -o fig_get_shell_x86 -target x86_64-apple-macos10.12

fig_get_shell: fig_get_shell_x86 fig_get_shell_arm
	lipo -create -output fig_get_shell fig_get_shell_x86 fig_get_shell_arm

fig_callback-arm: callback.c util.c $(LIBFIG)
	$(CC) callback.c util.c $(CFLAGS) -o fig_callback-arm $(LDFLAGS) $(LDLIBS) -target arm64-apple-macos11

fig_callback-x86: callback.c util.c $(LIBFIG)
	$(CC) callback.c util.c $(CFLAGS) -o fig_callback-x86 $(LDFLAGS) $(LDLIBS) -target x86_64-apple-macos10.12

fig_callback: fig_callback-x86 fig_callback-arm
	lipo -create -output fig_callback fig_callback-x86 fig_callback-arm

install: all
	mkdir -p $(HOME)/.fig/bin; \
	cd $(HOME)/.fig/bin && rm -rf $(PROGS) *figterm* && cd $(ROOT) && cp $(PROGS) $(HOME)/.fig/bin; \
	# Add fake fig binary on linux or if fig not installed that just logs
	# commands to fig.
	command -v ~/.fig/bin/fig > /dev/null 2>&1 || ( \
		printf "#!/bin/bash\necho \"\$$@\" >> ~/.fig/fig.log" > $(HOME)/.fig/bin/fig && \
		chmod +x $(HOME)/.fig/bin/fig)

clean:
	rm -f $(PROGS) $(TEMPFILES) *.o *.log; \
	rm -f *-x86 *-arm; \
	rm -rf *.dSYM; \
  (cd $(ROOT)/lib && $(MAKE) clean && rm libvterm.*); \
  (cd $(ROOT)/libvterm && $(MAKE) PREFIX=$(ROOT)/libvterm clean && rm -rf lib); \

$(LIBVTERM):
	(cd $(ROOT)/libvterm && $(MAKE) clean && $(MAKE) LIBRARY="libvterm.arm.la" CFLAGS="-target arm64-apple-macos11" PREFIX=$(ROOT)/libvterm install-lib);
	(cd $(ROOT)/libvterm && $(MAKE) clean && $(MAKE) LIBRARY="libvterm.x86.la" CFLAGS="-target x86_64-apple-macos10.12" PREFIX=$(ROOT)/libvterm install-lib);
	lipo -create -output $(ROOT)/libvterm/lib/libvterm.a $(ROOT)/libvterm/lib/libvterm.x86.a $(ROOT)/libvterm/lib/libvterm.arm.a

$(LIBFIG):
	(cd $(ROOT)/lib && $(MAKE))
