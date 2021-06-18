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

#define BUFFSIZE (1024 * 100)

void loop(int, pid_t, pid_t);

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

void on_child_exit() {
  // notify parent process on exit.
  kill(getppid(), SIGTERM);
  log_info("child exiting");
}

void child_loop(int ptyp) {
  int nread;
  char buf[BUFFSIZE + 1];

  if (atexit(on_child_exit))
    err_sys("atexit error");

  for (;;) {
    // Copy stdin to pty parent.
    if ((nread = read(STDIN_FILENO, buf, BUFFSIZE)) < 0)
      err_sys("read error from stdin");
    log_debug("Read %d chars on child", nread);
    if (write(ptyp, buf, nread) != nread)
      err_sys("write error to parent pty");
    if (nread == 0)
      break;
  }

  EXIT(0);
}

int main(int argc, char *argv[]) {
  int fdm, fdp;
  pid_t pid;
  char ptc_name[30];
  char log_name[100];

  _parent_shell = argv[1];

  FigInfo* fig_info = init_fig_info();

  // TODO(sean) breaks if these are NULL.
  if (!isatty(STDIN_FILENO) || fig_info->term_session_id == NULL ||
      fig_info->fig_integration_version == NULL) {
    execvp(argv[0], argv);
    EXIT(1);
  }

  if ((fdp = ptyp_open(ptc_name, sizeof(ptc_name))) < 0)
    goto fallback;

  // Get log file name from ptc_name, e.g. pty_dev_pts_4.log.
  sprintf(log_name, "pty%s.log", ptc_name);
  replace(log_name, '/', '_');

  // Initialize logging.
  set_logging_level(LOG_INFO);
  char* log_path = fig_path(log_name);
  init_log_file(log_path);
  free(log_path);

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
    pid_t child;

    if ((child = fork()) < 0) {
      err_sys("fork error");
    } else if (child == 0) {
      child_loop(fdm);
    }

    log_info("Shell: %d", pid);
    log_info("Child: %d", child);
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

    if (set_sigaction(SIGABRT, abort_handler) < 0)
      err_sys("sigabrt error");

    if (set_sigaction(SIGSEGV, abort_handler) < 0)
      err_sys("sigsegv error");

    // copy stdin -> ptyp, ptyp -> stdout
    loop(fdm, child, pid);
    EXIT(0);
  }

fallback:
  log_info("launching shell exe: %s", _parent_shell);
  launch_shell();
}
