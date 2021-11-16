//
//  pty.c
//  fig
//
//  Created by Matt Schrage on 11/15/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

#include "pty.h"

#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <signal.h>

#include <sys/types.h>
#include <sys/termios.h>
#include <stdarg.h>
#include <stdbool.h>
#include <errno.h>
#include <fcntl.h>
#include <termios.h>

#define _POSIX_C_SOURCE 200809L
#define _DEFAULT_SOURCE

#if defined(SOLARIS)
#define _XOPEN_SOURCE 600
#else
#define _XOPEN_SOURCE 700
#endif

#if defined(__APPLE__) || !defined(TIOCGWINSZ)
#include <sys/ioctl.h>
#endif

#ifdef LINUX
#define OPTSTR "+d:einv"
#else
#define OPTSTR "d:einv"
#endif

#define BUFFSIZE (1024 * 100)

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

void error(int log, const char* fmt, ...) {
  va_list ap;
  va_start(ap, fmt);
  vdprintf(log, fmt, ap);
  va_end(ap);
  exit(1);
}

int pty_send(Pty* p, const char* buf, int count) {
    int nwrite;
    while ((nwrite = write(p->fd, buf, count)) < 0 && errno == EINTR) {};
    return nwrite;
}

Pty* pty_init(const char* logfile) {
  int fdp;
  pid_t process_pid;
  pid_t pty_pid;
  char ptc_name[30];
  int log = open(logfile, O_APPEND | O_CREAT | O_WRONLY);

  // Open parent/child ends of pty.
  if ((fdp = ptyp_open(ptc_name)) < 0)
    error(log, "failed to open pty parent");

  if ((process_pid = fork()) < 0) {
    error(log, "failed to fork pty child");
  } else if (process_pid == 0) {
    close(log);
    ptyc_open(fdp, ptc_name, NULL, NULL);
    char* argv[] = { "/bin/bash", "--noprofile", "--norc", "--noediting", NULL };
    execvp(argv[0], argv);
    exit(1);
  }

//  if ((pty_pid = fork()) < 0) {
//    error(log, "failed to fork pty parent");
//  } else if (pty_pid == 0) {
//    char buf[BUFFSIZE + 1];
//    for (;;) {
//      int nread = read(fdp, buf, BUFFSIZE - 1);
//      if (nread <= 0)
//        break;
//      if (write(log, buf, nread) != nread)
//        error(log, "failed to write to log file");
//    }
//    close(log);
//    exit(0);
//  }
  close(log);

  Pty* pty = malloc(sizeof(Pty));
  pty->process_pid = process_pid;
  pty->fd = fdp;

  return pty;
}

void pty_free(Pty* pty) {
  kill(pty->process_pid, SIGKILL);
  free(pty);
}
