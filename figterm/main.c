#include "fig.h"
#include <pwd.h>
#include <execinfo.h>
#include <termios.h>
#include <unistd.h>
#include <vterm.h>
#include <errno.h>

#ifdef LINUX
#define OPTSTR "+d:einv"
#else
#define OPTSTR "d:einv"
#endif

#ifndef _PATH_BSHELL
#define _PATH_BSHELL "/bin/sh"
#endif

#define BUFFSIZE (1024 * 100)
#define FIGTERM_VERSION 3
#define FIG_MAX(x,y) (((x) >= (y)) ? (x) : (y))

void abort_handler(int sig) {
  log_error("Aborting %d: %d", getpid(), sig);
  void *array[10];
  size_t size = backtrace(array, 10);
  int i;

  // print out all the frames to stderr
  char** symbols = backtrace_symbols(array, size);

  int total_len = 0;
  for (i = 0; i < size; i += 1) {
    total_len += strlen(symbols[i]);
  }

  char* tmp = malloc(sizeof(char) * (total_len + size));
  tmp[0] = '\0';
  for (i = 0; i < size; i += 1) {
    strcat(tmp, symbols[i]);
    if (i != size - 1) {
      strcat(tmp, "\n");
    }
  }
  log_warn("Error:\n%s", tmp);
  free(tmp);
  free(symbols);

  EXIT(1);
}

char* _parent_shell = NULL;
char* _parent_shell_is_login = NULL;
char* _parent_shell_extra_args = NULL;

void launch_shell(bool fatal_crash) {
  if (_parent_shell == NULL) {
    if ((_parent_shell = getenv("FIG_SHELL")) == NULL)
      EXIT(1);
  }

  if (_parent_shell_is_login == NULL)
    _parent_shell_is_login = getenv("FIG_IS_LOGIN_SHELL");

  if (_parent_shell_extra_args == NULL)
    _parent_shell_extra_args = getenv("FIG_SHELL_EXTRA_ARGS");

  int nargs = 2;
  char** args = malloc(sizeof(char*) * nargs);
  args[nargs - 2] = _parent_shell;
  args[nargs - 1] = NULL;

  bool is_login = _parent_shell_is_login != NULL && *_parent_shell_is_login == '1';
  if (is_login) {
    nargs += 1;
    args = realloc(args, sizeof(char*) * nargs);
    args[nargs - 2] = "--login";
    args[nargs - 1] = NULL;
  }

  if (_parent_shell_extra_args != NULL) {
    char* tmp = strdup(_parent_shell_extra_args);
    char* arg = strtok(tmp, " ");

    while (arg) {
      if (strcmp(arg, "--login") != 0) {
        nargs += 1;
        args = realloc(args, sizeof(char*) * nargs);
        args[nargs - 2] = strdup(arg);
        args[nargs - 1] = NULL;
      }
      arg = strtok(NULL, " ");
    }
    free(tmp);
  }

  // Expose shell variables for version and to prevent nested fig term launches.
  char version[3];
  sprintf(version, "%d", FIGTERM_VERSION);
  setenv("FIG_TERM", "1", 1);
  setenv("FIG_TERM_VERSION", version, 1);
  if (getenv("TMUX") != NULL)
    setenv("FIG_TERM_TMUX", "1", 1);

  // Clean up environment and launch shell.
  unsetenv("FIG_SHELL");
  unsetenv("FIG_IS_LOGIN_SHELL");
  unsetenv("FIG_START_TEXT");
  unsetenv("FIG_SHELL_EXTRA_ARGS");

  if (fatal_crash) {
    setenv("FIG_TERM_CRASHED", "1", 1);
  }

  if (execvp(args[0], args) < 0) {
    EXIT(1);
  }
  free(args);
  EXIT(1);
}

void on_figterm_exit() {
  // On exit launch another instance of shell in figterm. Because this is the
  // child process some things may no longer work properly, e.g. 
  // iTerm's "Reuse Previous Sessions' option will still use the directory from
  // the pty child shell.
  int status = get_exit_status();
  log_info("Exiting (%d).", status);
  free_fig_info();
  close_log_file();
  history_file_close();
  fig_socket_cleanup();
  tty_reset(STDIN_FILENO);
  if (status != 0) {
    // Unexpected exit, fallback to exec parent shell.
    launch_shell(true);
  }
}

void initialize_logging(char* ptc_name) {
  char log_name[100];

  // Get log file name from ptc_name, e.g. logs/figterm_dev_pts_4.log.
  sprintf(log_name, "figterm%s.log", ptc_name);
  replace(log_name, '/', '_');

  // Initialize logging.
  char* log_file = log_path(log_name);

  char* log_level = getenv("FIG_LOG_LEVEL");
  if (log_level != NULL) {
    set_logging_level_from_string(log_level);
  }

  init_log_file(log_file);
  free(log_file);
}

static FigTerm* _ft; 

// TODO(sean) This should probably not be done inside a signal handler,
// consider non-blocking i/o and setting a flag instead.
void handle_winch(int sig) { figterm_resize(_ft); }

void publish_buffer(FigTerm* ft) {
  int index;
  char* buffer = figterm_get_buffer(ft, &index);
  if (buffer != NULL) {
    log_info("guess: %s|\nindex: %d", buffer, index);
  }

  if (buffer == NULL || index < 0) {
    log_info("Buffer is null or invalid index, not publishing...");
    return;
  }

  if (get_logging_level() == LOG_DEBUG) {
    figterm_log(ft, '.');
  }

  char* context = figterm_get_shell_context(ft);
  char* buffer_escaped = escaped_str(buffer);

  publish_json("{\"hook\":{\"editBuffer\":{\"text\":\"%s\",\"cursor\":\"%i\",\"context\": %s}}}", buffer_escaped, index, context);
  
  free(context);
  free(buffer_escaped);
}

// Main figterm loop.
void figterm_loop(int ptyp_fd, pid_t shell_pid, char* initial_command) {
  int nread;
  char buf[BUFFSIZE + 1];
  FigTerm* ft;

  if (set_sigaction(SIGWINCH, handle_winch) == SIG_ERR)
    err_sys("signal_intr error for SIGWINCH");

  ft = _ft = figterm_new(shell_pid, ptyp_fd);
  int incoming_listener = fig_socket_listen();
  int incoming_socket = -1;
  if (incoming_listener == -1)
    log_error("Failed to open incoming socket, insertions will fail.");

  fd_set rfd;

  bool is_first_time = true;

  for (;;) {
    FD_ZERO(&rfd);
    FD_SET(STDIN_FILENO, &rfd);
    FD_SET(ptyp_fd, &rfd);
    if (incoming_listener >= 0)
      FD_SET(incoming_listener, &rfd);

    if (incoming_socket >= 0)
      FD_SET(incoming_socket, &rfd);

    if (figterm_has_seen_prompt(ft) && is_first_time) {
      if (initial_command != NULL && strlen(initial_command) > 0) {
        int cmdlen = strlen(initial_command);
        char* tmpbuf = malloc((cmdlen + 2) * sizeof(char));
        sprintf(tmpbuf, "%s\n", initial_command);
        if (write(ptyp_fd, tmpbuf, cmdlen + 1) != cmdlen + 1) {
          free(tmpbuf);
          err_sys("write error to parent pty");
        }
        free(tmpbuf);
      }
      is_first_time = false;
    }

    int max_fd = FIG_MAX(FIG_MAX(ptyp_fd, incoming_listener), incoming_socket);
    int n = select(max_fd + 1, &rfd, 0, 0, NULL);
    if (n < 0 && errno != EINTR) {
      err_sys("select error");
    }
    if (n > 0 && FD_ISSET(STDIN_FILENO, &rfd)) {
      // Copy stdin to pty parent.
      if ((nread = read(STDIN_FILENO, buf, BUFFSIZE)) < 0)
        err_sys("read error from stdin");
      log_info("Read %d chars on stdin", nread);
      if (write(ptyp_fd, buf, nread) != nread)
        err_sys("write error to parent pty");
      if (nread == 0)
        break;
    }
    if (n > 0 && FD_ISSET(ptyp_fd, &rfd)) {
      nread = read(ptyp_fd, buf, BUFFSIZE - 1);
      log_info("read %d chars on ptyp_fd (%d)", nread, errno);
      if (nread < 0 && errno == EINTR)
        continue;
      else if (nread <= 0)
        break;

      // Write to figterm first so we can e.g. chdir before terminal emulator.
      if (!figterm_is_disabled(ft))
        figterm_write(ft, buf, nread);

      if (write(STDOUT_FILENO, buf, nread) != nread)
        err_sys("write error to stdout");

      if (!figterm_is_disabled(ft) && figterm_can_send_buffer(ft)) {
        publish_buffer(ft);
      }
    }
    if (n > 0 && FD_ISSET(incoming_listener, &rfd)) {
      log_info("Got message on socket");
      incoming_socket = accept(incoming_listener, NULL, NULL);
      if (incoming_socket < 0) {
        log_warn("Failed to accept message on socket");
      }
    }
    if (n > 0 && FD_ISSET(incoming_socket, &rfd)) {
      nread = read(incoming_socket, buf, BUFFSIZE - 1);
      if (nread == -1) {
        log_warn("Failed to read on socket %d (%d): %s", incoming_socket, errno, strerror(errno));
      } else {
        log_warn("Message (%d): %.*s", nread, nread, buf);
        if (write(ptyp_fd, buf, nread) != nread)
          err_sys("write error to parent pty");
      }
      close(incoming_socket);
      incoming_socket = -1;
    }
  }

  // clean up
  figterm_free(ft);
}

int main(int argc, char *argv[]) {
  int fdp;
  pid_t pid;
  char ptc_name[30];

  struct termios term;
  struct winsize ws;

  FigInfo* fig_info = init_fig_info();
  char* initial_command = getenv("FIG_START_TEXT");

  for (int i = 1; i < argc; i++) {
    if (strcmp(argv[i], "--version") == 0 || strcmp(argv[i], "-v") == 0) {
      printf("Figterm version: %d\n", FIGTERM_VERSION);
      exit(0);
    }
  }

  char* log_level = getenv("FIG_LOG_LEVEL");
  bool log_debug = log_level != NULL && strcmp(log_level, "DEBUG") == 0;
  if (log_debug) {
    printf("Checking stdin fd validity...\n");
  }

  if (!isatty(STDIN_FILENO) ||
      fig_info->term_session_id == NULL ||
      fig_info->fig_integration_version == NULL)
    goto fallback;

  if (tcgetattr(STDIN_FILENO, &term) < 0)
    goto fallback;

  if (ioctl(STDIN_FILENO, TIOCGWINSZ, (char *) &ws) < 0)
    goto fallback;

  // Open parent/child ends of pty.
  if ((fdp = ptyp_open(ptc_name)) < 0)
    goto fallback;

  set_pty_name(ptc_name);

  if (log_debug) {
    printf("Forking child shell process\n");
  }

  if ((pid = fork()) < 0) {
    log_error("fork error");
    goto fallback;
  } else if (pid != 0) {
    initialize_logging(ptc_name);
    // figterm process, parent of shell process.
    pid_t shell_pid = pid;
    log_info("Shell: %d", shell_pid);
    log_info("Figterm: %d", getpid());

    char* context = printf_alloc(
      "{\"sessionId\":\"%s\",\"pid\":\"%i\",\"ttys\":\"%s\",\"integrationVersion\":\"%s\"}",
      fig_info->term_session_id,
      shell_pid,
      ptc_name,
      fig_info->fig_integration_version
    );
    char* bundle = get_term_bundle();
    publish_json("{\"hook\":{\"init\":{\"context\": %s, \"bundle\": \"%s\"}}}", context, bundle);
    free(context);

    // On exit fallback to launching same shell as parent if unexpected exit.
    if (atexit(on_figterm_exit) < 0) {
      kill(shell_pid, SIGKILL);
      err_sys("error setting atexit");
    }

    // Set parent tty to raw, passthrough mode.
    if (tty_raw(STDIN_FILENO) < 0)
      err_sys("tty_raw error");

    if (set_sigaction(SIGABRT, abort_handler) == SIG_ERR)
      err_sys("sigabrt error");

    if (set_sigaction(SIGSEGV, abort_handler) == SIG_ERR)
      err_sys("sigsegv error");

    // copy stdin -> ptyp, ptyp -> stdout
    figterm_loop(fdp, shell_pid, initial_command);
    EXIT(0);
  }

  if (log_debug) {
    printf("About to launch shell\n");
  }

  // Child process becomes pty child and launches shell.
  ptyc_open(fdp, ptc_name, &term, &ws);
fallback:
  launch_shell(false);
}
