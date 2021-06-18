ROOT=..
include $(ROOT)/Makefile.shared
LIBVTERM=$(ROOT)/libvterm/libvterm.la

all:	libfig.a

libfig.a:	base64.o string.o log.o pty.o tty.o exit.o
	$(AR) rv libfig.a $?

clean:
	rm -f *.o libfig.a
