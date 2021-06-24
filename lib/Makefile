ROOT=$(shell ../realpath.sh)/..
include $(ROOT)/Makefile.shared

all: libfig

libfig: base64.o string.o log.o pty.o tty.o exit.o
	ar rv libfig.a $?

clean:
	rm -f *.o libfig.a
