ROOT=.
include $(ROOT)/Makefile.shared
LIBFIG=$(ROOT)/lib/libfig.a
LIBVTERM=$(ROOT)/libvterm/libvterm.la

PROGS =	fig_pty

all:	$(PROGS)

fig_pty:	main.o loop.o term_state.o figterm.o $(LIBVTERM) $(LIBFIG)
	$(CC) $(CFLAGS) -o fig_pty main.o loop.o term_state.o figterm.o $(LDFLAGS) $(LDLIBS)

clean:
	rm -f $(PROGS) $(TEMPFILES) *.o; \
	rm out.log; \
  (cd $(ROOT)/lib && $(MAKE) clean); \
  (cd $(ROOT)/libvterm && $(MAKE) clean); \

$(LIBVTERM):
	(cd $(ROOT)/libvterm && $(MAKE) libvterm.la);

$(LIBFIG):
	(cd $(ROOT)/lib && $(MAKE))
