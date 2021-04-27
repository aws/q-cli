#include "fig.h"
#include <errno.h>
#include <fcntl.h>
#include <termios.h>

#if defined(SOLARIS)
#include <stropts.h>
#endif

pid_t pty_fork(int *ptrfdm, char *child_name, int child_name_size,
               const struct termios *term, const struct winsize *ws) {
  int fdp, fdc;
  pid_t pid;
  char ptc_name[20];

  if ((fdp = ptyp_open(ptc_name, sizeof(ptc_name))) < 0)
    err_sys("can't open master pty: %s, error %d", ptc_name, fdp);

  if (child_name != NULL) {
    // Return child's name
    strncpy(child_name, ptc_name, child_name_size);
    child_name[child_name_size - 1] = '\0';
  }

  if ((pid = fork()) < 0) {
    return -1;
  } else if (pid == 0) {
    if (setsid() < 0)
      err_sys("setsid error");

    // System V acquires controlling terminal on open().
    if ((fdc = ptyc_open(ptc_name)) < 0)
      err_sys("can't open slave pty");
    close(fdp); /* all done with master in child */

#if defined(BSD)
    // BSD acquires controlling terminal with TIOCSCTTY.
    if (ioctl(fdc, TIOCSCTTY, (char *)0) < 0)
      err_sys("TIOCSCTTY error");
#endif
    // Set child's termios and window size.
    if (term != NULL) {
      if (tcsetattr(fdc, TCSANOW, term) < 0)
        err_sys("tcsetattr error on slave pty");
    }
    if (ws != NULL) {
      if (ioctl(fdc, TIOCSWINSZ, ws) < 0)
        err_sys("TIOCSWINSZ error on slave pty");
    }

    // Slave becomes stdin/stdout/stderr of child.
    if (dup2(fdc, STDIN_FILENO) != STDIN_FILENO)
      err_sys("dup2 error to stdin");
    if (dup2(fdc, STDOUT_FILENO) != STDOUT_FILENO)
      err_sys("dup2 error to stdout");
    if (dup2(fdc, STDERR_FILENO) != STDERR_FILENO)
      err_sys("dup2 error to stderr");
    if (fdc != STDIN_FILENO && fdc != STDOUT_FILENO && fdc != STDERR_FILENO)
      close(fdc);
    return 0;
  } else {
    *ptrfdm = fdp;
    return pid;
  }
}

int ptyp_open(char *ptc_name, int ptc_name_size) {
  char *ptr;
  int fdp, err;

  if ((fdp = posix_openpt(O_RDWR)) < 0)
    return -1;
  // grant access to child, clear lock flag on child, & get child's name
  if (grantpt(fdp) < 0 || unlockpt(fdp) < 0 || (ptr = ptsname(fdp)) == NULL) {
    err = errno;
    close(fdp);
    errno = err;
    return -1;
  }

  // return name of child and fd of parent
  strncpy(ptc_name, ptr, ptc_name_size);
  ptc_name[ptc_name_size - 1] = '\0';
  return fdp;
}

int ptyc_open(char *ptc_name) {
  int fdc = open(ptc_name, O_RDWR);

  if (fdc < 0)
    return -1;

#if defined(SOLARIS)
  int err, setup;
  // Check if stream is already set up by autopush facility.
  if ((setup = ioctl(fdc, I_FIND, "ldterm")) < 0)
    goto errout;

  if (setup == 0) {
    if (ioctl(fdc, I_PUSH, "ptem") < 0 || ioctl(fdc, I_PUSH, "ldterm") < 0 ||
        ioctl(fdc, I_PUSH, "ttcompat") < 0) {
    errout:
      err = errno;
      close(fdc);
      errno = err;
      return -1;
    }
  }
#endif
  return fdc;
}
