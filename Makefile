ROOT=$(shell ./realpath.sh)
include $(ROOT)/Makefile.shared
LIBFIG=$(ROOT)/lib/libfig.a
LIBVTERM=$(ROOT)/lib/libvterm.a

PROGS =	fig_pty fig_get_shell

all:	$(PROGS)

fig_pty:	main.o loop.o term_state.o figterm.o util.o $(LIBVTERM) $(LIBFIG)
	$(CC) $(CFLAGS) -o fig_pty main.o loop.o term_state.o figterm.o util.o $(LDFLAGS) $(LDLIBS)

fig_get_shell:	get_shell.o
	$(CC) $(CFLAGS) -o fig_get_shell get_shell.o

install: all
	mkdir -p $(HOME)/.fig/bin; \
	cd $(ROOT) && cp $(PROGS) $(HOME)/.fig/bin; \
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
