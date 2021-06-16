ROOT=$(shell ./realpath.sh)
include $(ROOT)/Makefile.shared
LIBFIG=$(ROOT)/lib/libfig.a
LIBVTERM=$(ROOT)/lib/libvterm.a

PROGS =	fig_pty

all:	$(PROGS)

fig_pty:	main.o loop.o term_state.o figterm.o util.o $(LIBVTERM) $(LIBFIG)
	$(CC) $(CFLAGS) -o fig_pty main.o loop.o term_state.o figterm.o util.o $(LDFLAGS) $(LDLIBS)

clean:
	rm -f $(PROGS) $(TEMPFILES) *.o *.log; \
  (cd $(ROOT)/lib && $(MAKE) clean && rm libvterm.*); \
  (cd $(ROOT)/libvterm && $(MAKE) PREFIX=$(ROOT)/libvterm clean); \

$(LIBVTERM):
	(cd $(ROOT)/libvterm && $(MAKE) PREFIX=$(ROOT)/libvterm install-lib);

$(LIBFIG):
	(cd $(ROOT)/lib && $(MAKE))
