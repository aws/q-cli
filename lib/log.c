#include "fig.h"
#include <errno.h>
#include <stdarg.h>
#include <stdio.h>

static const char *log_levels[] = {"DEBUG", "INFO", "WARN", "ERROR", "FATAL"};

int logging_level = LOG_INFO;
FILE *log_file;

void vlog_msg(int level, const char *file, int line, const char *fmt,
              va_list ap) {
  if (level >= logging_level) {
    time_t t = time(NULL);
    struct tm *time = localtime(&t);
    if (log_file == NULL) {
      char tmp[50];
      sprintf(tmp, "out.%d.log", getpid());
      log_file = fopen(tmp, "w");
    }

    char buf[64];
    buf[strftime(buf, sizeof(buf), "%Y-%m-%d %H:%M:%S", time)] = '\0';
    fprintf(log_file, "%s %-5s %s:%d: ", buf, log_levels[level], file, line);
    vfprintf(log_file, fmt, ap);
    fprintf(log_file, "\n");
    fflush(log_file);
  }
}

void log_msg(int level, const char *file, int line, const char *fmt, ...) {
  va_list ap;
  va_start(ap, fmt);
  vlog_msg(level, file, line, fmt, ap);
  va_end(ap);
}

// Fatal error related to a system call
void err_sys_msg(const char *file, int line, const char *fmt, ...) {
  char buf[MAXLINE];

  va_list ap;
  va_start(ap, fmt);

  vsnprintf(buf, MAXLINE - 1, fmt, ap);
  snprintf(buf + strlen(buf), MAXLINE - strlen(buf) - 1, ": %s",
           strerror(errno));
  strcat(buf, "\n");
  va_end(ap);

  log_msg(LOG_FATAL, __FILE__, __LINE__, "%s", buf);

  // TODO(sean) try to replicate current shell to fail fully silently: e.g.
  // if read/write fails within vim or something, the user will be popped out
  // of vim and into a parent shell no longer in vim. One solution is to stay
  // in the PTY and disable everything but the basic read/write calls.
  tty_reset(STDIN_FILENO);
  exit(1);
}

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
