ROOT=..
CC=gcc
CFLAGS=-I$(ROOT)/include -Wall -pthread $(EXTRA)

all:	libfig.a

libfig.a:	base64.o string.o log.o pty.o tty.o exit.o
	ar rv libfig.a $?

clean:
	rm -f *.o libfig.a
