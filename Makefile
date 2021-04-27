ROOT=.
include $(ROOT)/Makefile.shared
LIBFIG=$(ROOT)/lib/libfig.a

PROGS =	fig_pty

all:	$(PROGS)

fig_pty:	main.o loop.o term_state.o figterm.o $(LIBFIG)
	$(CC) $(CFLAGS) -o fig_pty main.o loop.o term_state.o figterm.o $(LDFLAGS) $(LDLIBS)

clean:
	rm -f $(PROGS) $(TEMPFILES) *.o; \
	rm out.log; \
  (cd $(ROOT)/lib && $(MAKE) clean)

$(LIBFIG):
	(cd $(ROOT)/lib && $(MAKE))
