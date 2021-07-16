ROOT=$(shell ./realpath.sh)
include $(ROOT)/Makefile.shared

LIBFIG=$(ROOT)/lib/libfig.a
LIBVTERM=$(ROOT)/lib/libvterm.a

PROGS =	figterm fig_get_shell

all:	$(PROGS)

figterm-arm: main.o figterm.o screen.o util.o $(LIBVTERM) $(LIBFIG)
	$(CC) $(CFLAGS) -o figterm-arm main.o figterm.o screen.o util.o $(LDFLAGS) $(LDLIBS) -target arm64-apple-macos11

figterm-x86: main.o figterm.o screen.o util.o $(LIBVTERM) $(LIBFIG)
	$(CC) $(CFLAGS) -o figterm-x86 main.o figterm.o screen.o util.o $(LDFLAGS) $(LDLIBS) -target x86_64-apple-macos10.12

figterm: figterm-x86 figterm-arm
	lipo -create -output figterm figterm-x86 figterm-arm

fig_get_shell_arm: get_shell.o
	$(CC) $(CFLAGS) -o fig_get_shell get_shell.o -target arm64-apple-macos11

fig_get_shell_x86: get_shell.o
	$(CC) $(CFLAGS) -o fig_get_shell get_shell.o -target x86_64-apple-macos10.12

fig_get_shell: fig_get_shell_x86 fig_get_shell_arm
	lipo -create -output fig_get_shell fig_get_shell_x86 fig_get_shell_arm

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
  (cd $(ROOT)/lib && $(MAKE) clean && rm libvterm.*); \
  (cd $(ROOT)/libvterm && $(MAKE) PREFIX=$(ROOT)/libvterm clean && rm -rf lib); \

$(LIBVTERM):
	(cd $(ROOT)/libvterm && $(MAKE) PREFIX=$(ROOT)/libvterm install-lib);

$(LIBFIG):
	(cd $(ROOT)/lib && $(MAKE))
