#include "fig.h"
#include <errno.h>
#include <fcntl.h>
#include <termios.h>

#if defined(SOLARIS)
#include <stropts.h>
#endif

int ptyp_open(char* ptc_name) {
  // Open pseudoterminal, return pty parent fd and pty child name.
  char* name;
  int fdp;

  if ((fdp = posix_openpt(O_RDWR)) < 0)
    return -1;

  // grant access to child, clear lock flag, and then get child's name.
  if (grantpt(fdp) < 0 || unlockpt(fdp) < 0 || (name = ptsname(fdp)) == NULL) {
    close(fdp);
    return -1;
  }

  strcpy(ptc_name, name);
  return fdp;
}

int ptyc_open(int fdp, char* ptc_name, const struct termios *term, const struct winsize *ws) {
  // Open pty child and set calling process as stdin/stdout/stderr of pty.
  int fdc, err;

  // Create a new session.
  if (setsid() < 0)
    return -1;
  close(fdp);

  // Open child, System V acquires controlling terminal on open.
  if ((fdc = open(ptc_name, O_RDWR)) < 0)
    return -1;

#if defined(BSD)
  // acquire controlling terminal with TIOCSCTTY.
  if (ioctl(fdc, TIOCSCTTY, (char *)NULL) < 0)
    goto fail;
#endif

  // Set child's termios and window size.
  if (term != NULL && tcsetattr(fdc, TCSANOW, term) < 0)
    goto fail;

  if (ws != NULL && ioctl(fdc, TIOCSWINSZ, ws) < 0)
    goto fail;

#if defined(SOLARIS)
  int setup;
  // Check if stream is already set up by autopush facility.
  if ((setup = ioctl(fdc, I_FIND, "ldterm")) < 0)
    goto fail;

  if (setup == 0) {
    if (ioctl(fdc, I_PUSH, "ptem") < 0 ||
        ioctl(fdc, I_PUSH, "ldterm") < 0 ||
        ioctl(fdc, I_PUSH, "ttcompat") < 0) {
      goto fail;
    }
  }
#endif

  // PTY becomes stdin/stdout/stderr of process.
  if (dup2(fdc, STDIN_FILENO) != STDIN_FILENO ||
      dup2(fdc, STDOUT_FILENO) != STDOUT_FILENO ||
      dup2(fdc, STDERR_FILENO) != STDERR_FILENO)
    goto fail;
  if (fdc != STDIN_FILENO && fdc != STDOUT_FILENO && fdc != STDERR_FILENO)
    close(fdc);

  return 0;

fail:
  err = errno;
  if (fdc >= 0)
    close(fdc);
  errno = err;
  return -1;
}
