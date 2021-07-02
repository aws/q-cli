#include "fig.h"
#include <pwd.h>
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

#define BUFFSIZE (1024 * 100)

void loop(int, pid_t);

void abort_handler(int sig) {
  log_warn("Aborting %d: %d", getpid(), sig);
  EXIT(1);
}

static char* _parent_shell;

void launch_shell() {
  char *const args[] = {_parent_shell, NULL};
  setenv("FIG_TERM", "1", 1);
  if (getenv("TMUX") != NULL)
    setenv("FIG_TERM_TMUX", "1", 1);
  if (execvp(args[0], args) < 0) {
    printf("FAILL");
    err_sys("execvp error");
  }
}

void on_pty_exit() {
  int status = get_exit_status();
  log_info("Exiting (%d).", status);
  free_fig_info();
  close_log_file();
  tty_reset(STDIN_FILENO);
  if (status != 0) {
    // Unexpected exit, fallback to exec parent shell.
    launch_shell();
  }
}

int main(int argc, char *argv[]) {
  int fdm, fdp;
  pid_t pid;
  char ptc_name[30];
  char log_name[100];

  char shell[30];
  int len = strlen(argv[0]) - strlen(" (figterm)");
  strncpy(shell, argv[0], len);
  shell[len] = '\0';

  _parent_shell = shell;

  FigInfo* fig_info = init_fig_info();

  // TODO(sean) breaks if these are NULL.
  if (!isatty(STDIN_FILENO) || fig_info->term_session_id == NULL ||
      fig_info->fig_integration_version == NULL) {
    execvp(argv[0], argv);
    EXIT(1);
  }

  if ((fdp = ptyp_open(ptc_name, sizeof(ptc_name))) < 0)
    goto fallback;

  set_pty_name(ptc_name);
  // Get log file name from ptc_name, e.g. logs/figterm_dev_pts_4.log.
  sprintf(log_name, "figterm%s.log", ptc_name);
  replace(log_name, '/', '_');

  // Initialize logging.
  set_logging_level(LOG_INFO);
  char* log_file = log_path(log_name);
  init_log_file(log_file);
  free(log_file);

  struct termios orig_termios;
  struct winsize size;

  if (tcgetattr(STDIN_FILENO, &orig_termios) < 0) {
    log_error("tcgetattr error on stdin");
    goto fallback;
  }

  if (ioctl(STDIN_FILENO, TIOCGWINSZ, (char *)&size) < 0) {
    log_error("get window size error");
    goto fallback;
  }

  if ((pid = pty_fork(&fdm, fdp, ptc_name, &orig_termios, &size)) < 0) {
    log_error("fork error");
  } else if (pid != 0) {
    log_info("Shell: %d", pid);
    log_info("Parent: %d", getpid());

    // On exit fallback to launching same shell as parent if unexpected exit.
    if (atexit(on_pty_exit) < 0) {
      log_error("error setting atexit");
      kill(pid, SIGKILL);
      launch_shell();
    }

    // Set parent tty to raw, passthrough mode.
    if (tty_raw(STDIN_FILENO) < 0)
      err_sys("tty_raw error");

    if (set_sigaction(SIGABRT, abort_handler) == SIG_ERR)
      err_sys("sigabrt error");

    if (set_sigaction(SIGSEGV, abort_handler) == SIG_ERR)
      err_sys("sigsegv error");

    // copy stdin -> ptyp, ptyp -> stdout
    loop(fdm, pid);
    EXIT(0);
  }

fallback:
  log_info("launching shell exe: %s", _parent_shell);
  launch_shell();
}
