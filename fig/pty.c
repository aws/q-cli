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
#define _XOPEN_SOURCE 700

#if defined(__APPLE__) || !defined(TIOCGWINSZ)
#include <sys/ioctl.h>
#endif

#ifdef LINUX
#define OPTSTR "+d:einv"
#else
#define OPTSTR "d:einv"
#endif

#define BUFFSIZE (1024 * 100)

typedef void SigHandler(int);
SigHandler *set_sigaction(int sig, SigHandler *func) {
  struct sigaction action, old_action;

  action.sa_handler = func;
  sigemptyset(&action.sa_mask);
  action.sa_flags = 0;
#ifdef SA_INTERRUPT
  action.sa_flags |= SA_INTERRUPT;
#endif
  if (sigaction(sig, &action, &old_action) < 0)
    return SIG_ERR;

  return old_action.sa_handler;
}

ssize_t pty_send(const int fd, const char* buf, int count) {
  if (fd < 0) return -1;
  ssize_t nwrite;
  while ((nwrite = write(fd, buf, count)) < 0 && errno == EINTR) {};
  return nwrite;
}

int pty_init(const int fdp, const char* logfile) {

  int ppid = getpid();
  int log_pid = fork();

  if (log_pid == 0) {
    int log = open(logfile, O_APPEND | O_CREAT | O_WRONLY, 0666);

    char buf[BUFFSIZE + 1];
    
    fd_set set;
    
    for (;;) {
      FD_ZERO(&set); /* clear the set */
      FD_SET(fdp, &set); /* add our file descriptor to the set */
      
      struct timeval timeout;
      
      timeout.tv_sec = 5;
      timeout.tv_usec = 0;
      
      int n = select(fdp + 1, &set, NULL, NULL, &timeout);
      if (n < 0 || (getppid() != ppid)) {
        break;
      } else if (n > 0) {
        ssize_t nread = read(fdp, buf, BUFFSIZE - 1);
        if (nread < 0 && errno == EINTR)
          continue;
        if (nread <= 0)
          break;
        if (write(log, buf, nread) != nread)
          break;
      }
    }
    close(fdp);
    close(log);
    _exit(0);
  }
  return log_pid;
}

void pty_free(const int fdp, const int process_pid) {
  if (fdp > 0) {
    write(fdp, "\x04", 1);
  }
  if (process_pid > 0) {
    kill(process_pid, SIGKILL);
  }
}
