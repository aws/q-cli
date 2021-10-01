#include "fig.h"
#include <termios.h>
#include <errno.h>

static struct termios	save_termios;
static int ttysavefd = -1;
static enum { RESET, RAW } ttystate = RESET;

// Put terminal into raw, passthrough mode.
int tty_raw(int fd){
	int	err;
	struct termios buf;

	if (ttystate != RESET) {
		errno = EINVAL;
		return -1;
	}
	if (tcgetattr(fd, &buf) < 0)
		return(-1);

	// Copy structure.
	save_termios = buf;

	// Turn off echo, canonical mode extended input processing & signal chars.
	buf.c_lflag &= ~(ECHO | ICANON | IEXTEN | ISIG);

	// Turn off SIGINT on BREAK, CR-to-NL, input parity check, strip 8th bit on input, 
	// and output control flow.
	buf.c_iflag &= ~(BRKINT | ICRNL | INPCK | ISTRIP | IXON);

	// Clear size bits, parity checking off.
	buf.c_cflag &= ~(CSIZE | PARENB);

	// 8 bits/char
	buf.c_cflag |= CS8;

	// Output processing off.
	buf.c_oflag &= ~(OPOST);

	// Set case b, 1 byte at a time, no timer.
	buf.c_cc[VMIN] = 1;
	buf.c_cc[VTIME] = 0;

	if (tcsetattr(fd, TCSAFLUSH, &buf) < 0)
		return -1;

	// Confirm all changes persisted.
	if (tcgetattr(fd, &buf) < 0) {
		err = errno;
		tcsetattr(fd, TCSAFLUSH, &save_termios);
		errno = err;
		return -1;
	}
	if ((buf.c_lflag & (ECHO | ICANON | IEXTEN | ISIG)) ||
	  (buf.c_iflag & (BRKINT | ICRNL | INPCK | ISTRIP | IXON)) ||
	  (buf.c_cflag & (CSIZE | PARENB | CS8)) != CS8 ||
	  (buf.c_oflag & OPOST) || buf.c_cc[VMIN] != 1 ||
	  buf.c_cc[VTIME] != 0) {
		// Only partial success, restore original settings.
		tcsetattr(fd, TCSAFLUSH, &save_termios);
		errno = EINVAL;
		return -1;
	}

	ttystate = RAW;
	ttysavefd = fd;
	return 0;
}

int tty_reset(int fd) {
	if (ttystate == RESET)
		return 0;
	if (tcsetattr(fd, TCSAFLUSH, &save_termios) < 0)
		return -1;
	ttystate = RESET;
	return 0;
}

void tty_atexit(void) {
	if (ttysavefd >= 0)
		tty_reset(ttysavefd);
}
