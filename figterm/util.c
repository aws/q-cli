#include "fig.h"
#include <stdlib.h>
#include <sys/types.h>
#include <sys/un.h>
#include <sys/socket.h>

static FigInfo *_fig_info;
static int fig_sock = -1;

int get_winsize(struct winsize *ws) {
  // Get window size of current terminal.
  const char *term = ctermid(NULL);
  if (!term[0]) {
    log_error("can't get name of controlling terminal");
    return -1;
  }
  int fd = open(term, O_RDONLY);
  if (fd == -1) {
    log_error("can't open terminal at %s", term);
    return -1;
  }
  if (ioctl(fd, TIOCGWINSZ, ws) == -1) {
    log_error("can't get the window size of %s", term);
    return -1;
  }
  close(fd);
  return 0;
}

void free_fig_info() {
  free(_fig_info);
}

FigInfo *init_fig_info() {
  // Store fig environment variables to send with a guess.
  char *term_session_id = getenv("TERM_SESSION_ID");
  char *fig_integration_version = getenv("FIG_INTEGRATION_VERSION");

  FigInfo *fi = malloc(sizeof(FigInfo));
  fi->term_session_id = term_session_id;
  fi->fig_integration_version = fig_integration_version;
  fi->pty_name = NULL;
  _fig_info = fi;
  return _fig_info;
}

void set_pty_name(char* name) {
  _fig_info->pty_name = name;
}
FigInfo *get_fig_info() { return _fig_info; }

char* fig_path(char* fname) {
  char* home_dir = getenv("HOME");
  int path_len = strlen(home_dir) + strlen("/.fig/") + strlen(fname) + 1;
  char* file = malloc(path_len * sizeof(char));
  return strcat(strcat(strcpy(file, home_dir), "/.fig/"), fname);
}

char* log_path(char* log_name) {
  char* dir = fig_path("logs/");

  struct stat st = {0};
  if (stat(dir, &st) == -1) {
    mkdir(dir, 0700);
  }

  int path_len = strlen(dir) + strlen(log_name) + 1;
  char* full_path = malloc(path_len * sizeof(char));
  strcat(strcpy(full_path, dir), log_name);
  free(dir);
  return full_path;
}

static int unix_socket_connect(char *path) {
  // Connect to a unix socket at path.
  int sock;
  if ((sock = socket(AF_UNIX, SOCK_STREAM, 0)) < 0)
    return -1;

  struct sockaddr_un remote;
  memset(&remote, 0, sizeof(struct sockaddr_un));
  remote.sun_family = AF_UNIX;
  strcpy(remote.sun_path, path);

  size_t len = SUN_LEN(&remote);
  if (connect(sock, (struct sockaddr *)&remote, len) == -1)
    return -1;
  return sock;
}

void sigpipe_handler(int sig) {
  fig_sock = -1;
}

int fig_socket_send(char* buf) {
  // Base64 encode buf and send to fig socket.
  int st;
  size_t out_len;
  SigHandler* old_handler;

  unsigned char *encoded =
      base64_encode((unsigned char *) buf, strlen(buf), &out_len);

  if (fig_sock < 0) {
    fig_sock = unix_socket_connect("/tmp/fig.socket");
  }

  if (fig_sock < 0) {
    log_warn("Can't connect to fig socket");
    return fig_sock;
  }
  
  // Handle sigpipe if socket is closed, reset afterwards.
  if ((old_handler = set_sigaction(SIGPIPE, sigpipe_handler)) == SIG_ERR)
    err_sys("sigpipe error");
  st = send(fig_sock, encoded, out_len, 0);
  if (set_sigaction(SIGPIPE, old_handler) == SIG_ERR)
    err_sys("sigpipe error");

  return st;
}
