#include "fig.h"
#include <errno.h>
#include <stdarg.h>
#include <stdio.h>
#include <sys/mman.h>
#include <pthread.h>

typedef struct
{
  FILE* file;
  pthread_mutex_t mutex;
  pthread_mutexattr_t attr;
} shared_file_mutex;

static shared_file_mutex* _mutex_log_file = NULL;


static const char *log_levels[] = {"FATAL", "ERROR", "WARN", "INFO", "DEBUG"};
static int _logging_level = LOG_INFO;

void set_logging_level(int level) {
  level = level < LOG_FATAL ? LOG_FATAL : level;
  level = level > LOG_DEBUG ? LOG_DEBUG : level;
  _logging_level = level;
}

void init_log_file(char* path) {
  int prot = PROT_READ | PROT_WRITE;
  int flags = MAP_SHARED | MAP_ANONYMOUS;
  _mutex_log_file = mmap(NULL, sizeof(shared_file_mutex), prot, flags, -1, 0);

  pthread_mutexattr_init(&_mutex_log_file->attr);
  pthread_mutexattr_setpshared(&_mutex_log_file->attr, PTHREAD_PROCESS_SHARED);
  _mutex_log_file->file = fopen(path, "w");
  pthread_mutex_init(&_mutex_log_file->mutex, &_mutex_log_file->attr);
}

void close_log_file() {
  pthread_mutex_destroy(&_mutex_log_file->mutex);
  pthread_mutexattr_destroy(&_mutex_log_file->attr);
}

void vlog_msg(int level, const char *file, int line, const char *fmt,
              va_list ap) {
  if (level <= _logging_level) {
    time_t t = time(NULL);
    struct tm *time = localtime(&t);
    if (_mutex_log_file->file == NULL) {
      return;
    }

    char buf[64];
    buf[strftime(buf, sizeof(buf), "%Y-%m-%d %H:%M:%S", time)] = '\0';

    pthread_mutex_lock(&_mutex_log_file->mutex);

    fprintf(_mutex_log_file->file, "[%s %-5s %d %s:%d] ", buf, log_levels[level], getpid(), file, line);
    vfprintf(_mutex_log_file->file, fmt, ap);
    fprintf(_mutex_log_file->file, "\n");
    fflush(_mutex_log_file->file);

    pthread_mutex_unlock(&_mutex_log_file->mutex);
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
