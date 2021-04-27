ROOT=..
include $(ROOT)/Makefile.shared

# Common temp files to delete from each directory.
TEMPFILES=core core.* *.o temp.* *.out
DIRS = lib 

LIBFIG	= libfig.a
OBJS   = string.o log.o pty.o tty.o

all:	$(LIBFIG)

$(LIBFIG):	$(OBJS)
	$(AR) rv $(LIBFIG) $?
	$(RANLIB) $(LIBFIG)


clean:
	rm -f *.o a.out core temp.* $(LIBFIG)
