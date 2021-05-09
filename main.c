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

void loop(int, int); /* in the file loop.c */

int validshell(const char *shell) {
  if (shell == NULL || *shell != '/')
    return (0);
  if (access(shell, X_OK) != 0)
    return (0);
  return (1);
}

static const char *getshell(void) {
  struct passwd *pw;
  const char *shell;

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
  log_info("ABORTT %d: %d", getpid(), sig);
  tty_reset(STDIN_FILENO);
  quick_exit(0);
}

int main(int argc, char *argv[]) {
  int fdm;
  pid_t pid;
  char child_name[20];

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
    // TODO(sean) change to getshell
    char *shell = "/bin/bash";
    char *const args[] = {shell, NULL};
    setenv("FIG_TERM", "1", 1);
    if (tmux != NULL) {
      log_info("IN TMUX");
    }
    setenv("FIG_TERM_TMUX", "1", 1);
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
