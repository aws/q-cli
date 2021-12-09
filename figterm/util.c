#include "fig.h"
#include <string.h>
#include <stdlib.h>
#include <sys/types.h>
#include <sys/un.h>
#include <sys/socket.h>

static FigInfo *_fig_info;
static int fig_sock = -1;
static int ipc_sock = -1;

int get_winsize(struct winsize *ws) {
  // Get window size of current terminal.
  const char *term = ctermid(NULL);
  CHECK(term[0], -1, "Can't get name of controlling terminal");

  int fd = open(term, O_RDONLY);
  CHECK_SYS(fd, "Can't open terminal at %s", term);
  CHECK_SYS(ioctl(fd, TIOCGWINSZ, ws), "Can't get the window size of %s", term);
  CHECK_SYS(close(fd), "Failed to close fd %d", fd);

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
  CHECK_NONNULL(fi, "Failed to malloc figinfo");
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
  CHECK_NONNULL(file, "Failed to malloc file name");
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

int unix_socket_listen(char *path) {
  // Connect to a unix socket at path.
  int sock = socket(AF_UNIX, SOCK_STREAM, 0);
  CHECK_SYS(sock, "Failed to create socket object");

  struct sockaddr_un remote;
  memset(&remote, 0, sizeof(struct sockaddr_un));
  remote.sun_family = AF_UNIX;
  strcpy(remote.sun_path, path);

  size_t len = SUN_LEN(&remote);

  CHECK_SYS(bind(sock, (struct sockaddr *) &remote, len), "Failed to bind socket");
  
  // Set backlog max of 5 queued messages
  listen(sock, 5);
  return sock;
}

static int unix_socket_connect(char *path) {
  // Connect to a unix socket at path.
  int sock = socket(AF_UNIX, SOCK_STREAM, 0);
  CHECK_SYS(sock, "Failed to create socket object");

  struct sockaddr_un remote;
  memset(&remote, 0, sizeof(struct sockaddr_un));
  remote.sun_family = AF_UNIX;
  strcpy(remote.sun_path, path);

  // https://rigtorp.se/sockets/
  int opt = 1;
  if (setsockopt(sock, SOL_SOCKET, SO_NOSIGPIPE, &opt, sizeof(opt)) == -1) {
      log_err("Failed to set SO_NOSIGPIPE");
      return -1;
  }

  size_t len = SUN_LEN(&remote);
  if (connect(sock, (struct sockaddr *)&remote, len) == -1) {
    log_err("Failed to connect to socket");
    CHECK_SYS(close(sock), "Failed to close socket");
    return -1;
  }
  return sock;
}

char* _incoming_socket_path = NULL;
int _incoming_socket_fd = -1;

int set_blocking(int fd, bool blocking) {
  int flags = fcntl(fd, F_GETFL);

  if (flags != -1)
    flags = fcntl(fd, F_SETFL, blocking ? flags ^ O_NONBLOCK : flags | O_NONBLOCK);

  if (flags == -1)
    log_warn("Failed to set fd blocking");

  return flags;
}

int fig_socket_listen() {
  FigInfo *fig_info = get_fig_info();
  _incoming_socket_path = malloc(sizeof("char") * (
    strlen("/tmp/figterm-.socket") + SESSION_ID_MAX_LEN + 1
  ));

  sprintf(_incoming_socket_path, "/tmp/figterm-%s.socket", fig_info->term_session_id);
  _incoming_socket_fd = unix_socket_listen(_incoming_socket_path);
  set_blocking(_incoming_socket_fd, false);
  return _incoming_socket_fd;
}

void fig_socket_cleanup() {
  if (_incoming_socket_fd != -1) {
    close(_incoming_socket_fd);
  }
  if (_incoming_socket_path != NULL) {
    unlink(_incoming_socket_path);
    free(_incoming_socket_path);
  }
}

void fig_sigpipe_handler(int sig) {
  if (fig_sock > -1) {
    if (close(fig_sock) < 0) {
      log_err("Failed to close fig socket");
    }
  }
  fig_sock = -1;
}

void ipc_sigpipe_handler(int sig) {
  if (ipc_sock > -1) {
    if (close(ipc_sock) < 0) {
      log_err("Failed to close ipc socket");
    }
  }
  ipc_sock = -1;
}

int fig_socket_send(char* buf) {
  // Base64 encode buf and send to fig socket.
  int st;
  size_t out_len;

  unsigned char *encoded =
      base64_encode((unsigned char *) buf, strlen(buf), &out_len);

  if (fig_sock < 0) {
    fig_sock = unix_socket_connect("/tmp/fig.socket");
    CHECK_SYS(fig_sock, "Can't connect to fig socket");
    CHECK_SYS(set_blocking(fig_sock, false), "Couldn't set fig sock to nonblocking");
  }
  
  st = send(fig_sock, encoded, out_len, 0);

  if (st < 0 && errno == EPIPE) {
    fig_sigpipe_handler(SIGPIPE);
    log_err("Error sending buffer to socket");
  }

  return st;
}

int ipc_socket_send(char* buf, int len) {
  // send to ipc socket. No base64 encoding.
  int st;

  if (ipc_sock < 0) {
    char* path = printf_alloc("%sfig.socket", getenv("TMPDIR"));
    ipc_sock = unix_socket_connect(path);
    CHECK_SYS(ipc_sock, "Can't connect to ipc socket");
    CHECK_SYS(set_blocking(ipc_sock, false), "Couldn't set ipc sock to nonblocking");
    free(path);
  }

  st = send(ipc_sock, buf, len, 0);
  if (st < 0 && errno == EPIPE) {
    ipc_sigpipe_handler(SIGPIPE);
    log_err("Error sending buffer to socket");
  }

  return st;
}

char* vprintf_alloc(const char* fmt, va_list va) {
  va_list arg_copy;
  va_copy(arg_copy, va);
  const int len = vsnprintf(NULL, 0, fmt, arg_copy);
  va_end(arg_copy);
  char *tmpbuf = malloc((len + 1) * sizeof(char));
  if (tmpbuf == NULL)
    return NULL;
  vsprintf(tmpbuf, fmt, va);
  return tmpbuf;
}

char* printf_alloc(const char* fmt, ...) {
  va_list va;
  va_start(va, fmt);
  char* tmpbuf = vprintf_alloc(fmt, va);
  va_end(va);
  return tmpbuf;
}

#define HEADER_PREFIX_LEN 10
#define HEADER_INT64_LEN 8
#define HEADER_LEN HEADER_PREFIX_LEN + HEADER_INT64_LEN

void publish_json(const char* fmt, ...) {
  va_list va;

  va_start(va, fmt);
  char* tmpbuf = vprintf_alloc(fmt, va);
  va_end(va);

  // Convert to int64 big endian
  unsigned int buf_len = strlen(tmpbuf);
  unsigned char len[8] = {
    0,
    0,
    0,
    0,
    (buf_len >> 24) & 0xFF,
    (buf_len >> 16) & 0xFF,
    (buf_len >> 8) & 0xFF,
    buf_len & 0xFF,
  };

  char* msg = printf_alloc("\x1b@fig-json%c%c%c%c%c%c%c%c%s", len[0],
                                                              len[1],
                                                              len[2], 
                                                              len[3], 
                                                              len[4], 
                                                              len[5], 
                                                              len[6], 
                                                              len[7],
                                                              tmpbuf);

  if (msg == NULL) {
    log_info("Null message, not sending");
  } else {
    if (ipc_socket_send(msg, HEADER_LEN + strlen(tmpbuf)) > -1) {
      log_info("done sending %s", tmpbuf);
    } else {
      log_info("failed sending");
    }
  }
  free(msg);
  free(tmpbuf);
}

// https://stackoverflow.com/a/33988826
char *escaped_str(const char *src) {
  int i, j;

  for (i = j = 0; src[i] != '\0'; i++) {
    if (src[i] == '\n' || src[i] == '\t' ||
        src[i] == '\\' || src[i] == '\"' ||
        src[i] == '/' || src[i] == '\b' ||
        src[i] == '\r' || src[i] == '\f') {
      j++;
    }
  }
  char* pw = malloc(sizeof(char) * (i + j + 1));

  for (i = j = 0; src[i] != '\0'; i++) {
    switch (src[i]) {
      case '\n': pw[i+j] = '\\'; pw[i+j+1] = 'n'; j++; break;
      case '\t': pw[i+j] = '\\'; pw[i+j+1] = 't'; j++; break;
      case '\\': pw[i+j] = '\\'; pw[i+j+1] = '\\'; j++; break;
      case '\"': pw[i+j] = '\\'; pw[i+j+1] = '\"'; j++; break;
      case '/': pw[i+j] = '\\'; pw[i+j+1] = '/'; j++; break;
      case '\b': pw[i+j] = '\\'; pw[i+j+1] = 'b'; j++; break;
      case '\r': pw[i+j] = '\\'; pw[i+j+1] = 'r'; j++; break;
      case '\f': pw[i+j] = '\\'; pw[i+j+1] = 'f'; j++; break;
      default:   pw[i+j] = src[i]; break;
    }
  }
  pw[i+j] = '\0';
  return pw;
}

char *get_term_bundle() {
  char *term_program = getenv("TERM_PROGRAM");
  if (term_program == NULL) {
    return "unknown";
  }

  if (strcmp(term_program, "iTerm.app") == 0) {
    return "com.googlecode.iterm2";
  }

  if (strcmp(term_program, "Apple_Terminal") == 0) {
    return "com.apple.Terminal";
  }

  if (strcmp(term_program, "Hyper") == 0) {
    return "co.zeit.hyper";
  }

  if (strcmp(term_program, "vscode") == 0) {
    char *term_program_version = getenv("TERM_PROGRAM_VERSION");
    
    if (term_program_version == NULL) {
      return "com.microsoft.vscode";
    }

    if (strstr(term_program_version, "insiders") != NULL) {
      return "com.microsoft.vscode-insiders";
    } else {
      return "com.microsoft.vscode";
    }
  }

  if (strcmp(term_program, "Hyper") == 0) {
    return "co.zeit.hyper";
  }

  char *term_bundle = getenv("TERM_BUNDLE_IDENTIFIER");

  if (term_bundle == NULL) {
    return "unknown";
  }

  return term_bundle;

}
