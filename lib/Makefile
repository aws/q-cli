ROOT=$(shell ../realpath.sh)/..
include $(ROOT)/Makefile.shared

override CFLAGS +=-c

PROGS = base64.o string.o log.o pty.o tty.o exit.o proc.o

all: libfig

libfig: libfig_arm.a libfig_x86.a
	lipo -create -output libfig.a libfig_x86.a libfig_arm.a

libfig_arm.a: base64.c string.c log.c pty.c tty.c exit.c proc.c
	$(CC) $? $(CFLAGS) -target arm64-apple-macos11
	ar rv libfig_arm.a $(PROGS)

libfig_x86.a: base64.c string.c log.c pty.c tty.c exit.c proc.c
	$(CC) $? $(CFLAGS) -target x86_64-apple-macos10.12
	ar rv libfig_x86.a $(PROGS)

clean:
	rm -f *.o libfig_arm.a libfig_x86.a libfig.a
