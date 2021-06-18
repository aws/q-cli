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

void on_pty_exit() {
  // Use SIGKILL to kill parent shell, shells trap SIGTERM.
  tty_reset(STDIN_FILENO);
  log_info("Exiting.");
}

void abort_handler(int sig) {
  log_warn("Aborting %d: %d", getpid(), sig);
  exit(0);
}

int main(int argc, char *argv[]) {
  int fdm;
  pid_t pid;
  char child_name[20];

  pid_t shell_pid = getpid();

  char *shell = get_exe(shell_pid);
  printf("shell exe: %s\n", shell);

  set_logging_level(LOG_INFO);
  char* log_path = fig_path("pty.log");
  init_log_file(log_path);

  FigInfo* fig_info = init_fig_info();
  char *tmux = getenv("TMUX");

  // TODO(sean) breaks if these are NULL.
  if (!isatty(STDIN_FILENO) || fig_info->term_session_id == NULL ||
      fig_info->fig_integration_version == NULL) {
    execvp(argv[0], argv + 1);
    err_sys("Not in valid tty");
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
    log_info("shell exe: %s", shell);

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
  if (atexit(on_pty_exit) < 0)
    err_sys("atexit error");
  if (set_sigaction(SIGABRT, abort_handler) < 0)
    err_sys("sigabrt error");
  if (set_sigaction(SIGSEGV, abort_handler) < 0)
    err_sys("sigsegv error");
  fclose(stderr);

  // copy stdin -> ptyp, ptyp -> stdout
  loop(fdm, pid);

  free(fig_info);
  free(log_path);
  close_log_file();
  log_info("Cleaned up");
  int x= 0;
  int ret = waitpid(pid, &x, 0);
  log_info("%d, %d, %d, %d", x, ret, WIFEXITED(x), WIFSIGNALED(x));
  exit(0);
}
