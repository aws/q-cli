#include "fig.h"
#include <pwd.h>
#include <term.h>
#include <termios.h>
#include <unistd.h>
#include <vterm.h>

#ifdef LINUX
#define OPTSTR "+d:einv"
#else
#define OPTSTR "d:einv"
#endif

#ifndef _PATH_BSHELL
#define _PATH_BSHELL "/bin/sh"
#endif

void loop(int, int);

static char *get_exe(pid_t pid) {
  char procfile[50];
  ssize_t ret;
  unsigned int bufsize = 1024;

  sprintf(procfile, "/proc/%d/exe", pid);
  char* tmp = calloc(bufsize, sizeof(char));
  while (true) {
    ret = readlink(procfile, tmp, bufsize - 1);
    if (ret == -1) {
      free(tmp);
      return NULL;
    } else if ((size_t) ret != bufsize - 1) {
      tmp[ret] = '\0';
      return tmp;
    }
    bufsize *= 2;
    tmp = (char *) realloc(tmp, bufsize);
  }
}

int validshell(const char *shell) {
  return shell != NULL && *shell == '/' && access(shell, X_OK) == 0;
}

static char *getshell(void) {
  struct passwd *pw;
  char *shell;
  shell = getenv("SHELL");
  if (validshell(shell))
    return shell;

  pw = getpwuid(getuid());
  if (pw != NULL && validshell(pw->pw_shell))
    return (pw->pw_shell);

  return (_PATH_BSHELL);
}

void kill_parent() {
  // Use SIGKILL to kill parent shell, shells trap SIGTERM.
  kill(getppid(), SIGKILL);
}

void abort_handler(int sig) {
  log_warn("Aborting %d: %d", getpid(), sig);
  tty_reset(STDIN_FILENO);

  // Use quick exit to avoid killing parent shell.
  quick_exit(0);
}

int main(int argc, char *argv[]) {
  int fdm;
  pid_t pid;
  char child_name[20];

  // TODO(sean) breaks if these are NULL.
  char *term_session_id = getenv("TERM_SESSION_ID");
  char *fig_integration_version = getenv("FIG_INTEGRATION_VERSION");
  char *tmux = getenv("TMUX");
  FigInfo *fi = malloc(sizeof(FigInfo));
  fi->term_session_id = term_session_id;
  fi->fig_integration_version = fig_integration_version;
  set_fig_info(fi);

  if (!isatty(STDIN_FILENO) || term_session_id == NULL ||
      fig_integration_version == NULL) {
    execvp(argv[0], argv + 1);
    err_sys("Not in tty");
  }

  struct termios orig_termios;
  struct winsize size;

  if (tcgetattr(STDIN_FILENO, &orig_termios) < 0)
    err_sys("tcgetattr error on stdin");
  if (ioctl(STDIN_FILENO, TIOCGWINSZ, (char *)&size) < 0)
    err_sys("TIOCGWINSZ error");

  pid = pty_fork(&fdm, child_name, sizeof(child_name), &orig_termios, &size);

  if (pid < 0) {
    err_sys("fork error");
  } else if (pid == 0) {
    // char *shell = get_exe(getppid());
    char* shell = getshell();
    char *const args[] = {shell, NULL};
    setenv("FIG_TERM", "1", 1);
    if (tmux != NULL) {
      setenv("FIG_TERM_TMUX", "1", 1);
    }
    if (execvp(args[0], args) < 0)
      err_sys("execvp error");
  }

  // Set parent tty to raw, passthrough mode.
  if (tty_raw(STDIN_FILENO) < 0)
    err_sys("tty_raw error");
  // Reset parent tty on exit.
  if (atexit(kill_parent) < 0)
    err_sys("atexit error");
  if (set_sigaction(SIGABRT, abort_handler) < 0)
    err_sys("sigabrt error");
  if (set_sigaction(SIGSEGV, abort_handler) < 0)
    err_sys("sigsegv error");
  fclose(stderr);

  // copy stdin -> ptyp, ptyp -> stdout
  loop(fdm, pid);

  free(fi);
  free(term_session_id);
  free(fig_integration_version);
  free(tmux);
  exit(0);
}
