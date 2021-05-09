#include "fig.h"
#include <ctype.h>
#include <errno.h>
#include <math.h>
#include <stdio.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <sys/un.h>
#include <sys/wait.h>
#include <unistd.h>
#include <vterm.h>
#include <vterm_keycodes.h>

#define BUFFSIZE (1024 * 100)
#define max(a, b)                                                              \
  ({                                                                           \
    __typeof__(a) _a = (a);                                                    \
    __typeof__(b) _b = (b);                                                    \
    _a > _b ? _a : _b;                                                         \
  })
#define min(a, b)                                                              \
  ({                                                                           \
    __typeof__(a) _a = (a);                                                    \
    __typeof__(b) _b = (b);                                                    \
    _a < _b ? _a : _b;                                                         \
  })

static volatile sig_atomic_t sigcaught;

// Called when child sends us SIGTERM
static void sig_term(int signo) { sigcaught = 1; }

char *extract_buffer(TermState *state, TermState *prompt_state, int *index) {
  int i = prompt_state->cursor->row - prompt_state->scroll;
  int j = prompt_state->cursor->col;

  // Invalid prompt cursor position, return null.
  if (i < 0 || state->row_lens[i] < j)
    return NULL;

  size_t total_len = 0;
  for (int k = i; k < state->nrows; k++) {
    total_len += state->row_lens[k] + 1;
  }
  total_len -= j;

  log_debug("Alloc text: %d", (int)total_len);
  char *text = malloc(sizeof(char) * (total_len + 1));
  int pos = 0;

  *index = -1;
  for (; i < state->nrows; i++) {
    char *row = state->rows[i];

    char *prow = NULL;
    int prow_len = 0;

    if (i + prompt_state->scroll < prompt_state->nrows) {
      prow = prompt_state->rows[i + prompt_state->scroll];
      prow_len = prompt_state->row_lens[i + prompt_state->scroll];
    }

    for (; j < state->row_lens[i]; j++) {
      char c = row[j];
      if (prow != NULL && j < prow_len && !isspace(c) && c == prow[j]) {
        c = ' ';
      }
      if (state->cursor->row == i && state->cursor->col - 1 == j) {
        *index = pos;
      }
      text[pos++] = c;
    }
    text[pos++] = '\n';
    j = 0;
  }
  text[pos] = '\0';
  return rtrim(text);
}

static void print_term_state(TermState *ts, bool is_prompt) {
  log_debug("text:");
  for (int i = 0; i < ts->nrows; i++) {
    log_debug("%.*s", ts->row_lens[i], ts->rows[i]);
  }
  log_debug("cursor pos: %d %d", ts->cursor->row, ts->cursor->col);
  log_debug("scrollback: %d", ts->scroll);
  log_debug("is_prompt: %s", is_prompt ? "true" : "false");
}

static int movecursor_cb(VTermPos pos, VTermPos oldpos, int visible,
                         void *user) {
  FigTerm *ft = user;
  log_debug("Move cursor: (%d, %d)->(%d, %d)", oldpos.row, oldpos.col, pos.row,
            pos.col);
  ft->cursor->row = pos.row;
  ft->cursor->col = pos.col;
  term_state_update_cursor(ft->state, pos);
  return 0;
}

static int sb_pushline_cb(int cols, const VTermScreenCell *cells, void *user) {
  FigTerm *ft = user;
  log_debug("Scroll down");
  ft->prompt_state->scroll += 1;
  return 0;
}

static int sb_popline_cb(int cols, VTermScreenCell *cells, void *user) {
  FigTerm *ft = user;
  log_debug("Scroll up");
  ft->prompt_state->scroll -= 1;
  return 0;
}

static int damage_cb(VTermRect rect, void *user) {
  FigTerm *ft = user;
  char *prompt_str = ft->damage_prompt ? " (+prompt)" : "";
  log_debug("Damage screen%s: (%d-%d, %d-%d)", prompt_str, rect.start_row,
            rect.end_row, rect.start_col, rect.end_col);
  term_state_update(ft->state, ft->vt, rect, false);
  if (ft->damage_prompt)
    term_state_update(ft->prompt_state, ft->vt, rect, false);

  print_term_state(ft->state, false);
  print_term_state(ft->prompt_state, true);
  return 0;
}

int settermprop_cb(VTermProp prop, VTermValue *val, void *user) {
  FigTerm *ft = user;
  log_info("Termprop: %d, %d", prop);
  if (prop == VTERM_PROP_ALTSCREEN) {
    log_debug("Altscreen: %s", val->boolean ? "on" : "off");
    ft->altscreen = val->boolean;
  } else if (prop == VTERM_PROP_TITLE) {
    log_info("Title: %s", val->string);
    ft->wait_for_prompt = true;
  }
  return 0;
}

int osc_cb(const char *command, size_t cmdlen, void *user) {
  log_debug("OSC CB: %d, %.*s", cmdlen, cmdlen, command);
  // FIG OSC
  int prefix_len = 4;
  if (strncmp(command, "697;", prefix_len) == 0) {
    command += prefix_len;
    cmdlen -= prefix_len;

    FigTerm *ft = user;
    if (strncmp(command, "NEW_CMD", cmdlen) == 0) {
      VTermRect rect = {};
      term_state_update(ft->prompt_state, ft->vt, rect, true);
      term_state_update_cursor(ft->prompt_state, *ft->cursor);
      log_info("Got prompt");
      ft->wait_for_prompt = false;
    }
    if (strncmp(command, "START_PROMPT", cmdlen) == 0) {
      VTermScreen *vts = vterm_obtain_screen(ft->vt);
      vterm_screen_set_damage_merge(vts, VTERM_DAMAGE_CELL);
      ft->damage_prompt = true;
    }
    if (strncmp(command, "END_PROMPT", cmdlen) == 0) {
      VTermScreen *vts = vterm_obtain_screen(ft->vt);
      vterm_screen_flush_damage(vts);
      vterm_screen_set_damage_merge(vts, VTERM_DAMAGE_ROW);
      ft->damage_prompt = false;
    }
  }
  return 0;
}

static VTermParserCallbacks vterm_parser_callbacks = {
    .osc = osc_cb,
};

static VTermScreenCallbacks vterm_screen_callbacks = {
    .damage = damage_cb,
    .settermprop = settermprop_cb,
    .movecursor = movecursor_cb,
    .sb_pushline = sb_pushline_cb,
    .sb_popline = sb_popline_cb,
};

char *make_secret_text(char *str, size_t *len) {
  *len = strlen(str) + 4;
  char *p = malloc(sizeof(char) * (*len + 1));
  sprintf(p, "%c]%s%c\\", 27, str, 27);
  p[*len] = '\0';
  return p;
}

void child_loop(int ptyp) {
  int nread;
  char buf[BUFFSIZE + 1];

  for (;;) {
    // Copy stdin to pty parent.
    if ((nread = read(STDIN_FILENO, buf, BUFFSIZE)) < 0)
      err_sys("read error from stdin");
    else if (nread == 0)
      break;
    if (write(ptyp, buf, nread) != nread)
      err_sys("write error to parent pty");
  }

  // notify parent process on exit.
  kill(getppid(), SIGTERM);
  exit(0);
}

static int fig_sock = -1;

int unix_socket_connect(char *path) {
  int sock;
  if ((sock = socket(AF_UNIX, SOCK_STREAM, 0)) < 0)
    return -1;

  struct sockaddr_un remote;
  remote.sun_family = AF_UNIX;
  strcpy(remote.sun_path, path);

  size_t len = strlen(remote.sun_path) + sizeof(remote.sun_family);
  if (connect(sock, (struct sockaddr *)&remote, len) == -1)
    return -1;
  return sock;
}

void publish_guess(int index, char *buffer) {
  FigInfo *fig_info = get_fig_info();
  size_t buflen = strlen(buffer) + strlen(fig_info->term_session_id) +
                  strlen(fig_info->fig_integration_version);

  char *tmpbuf = malloc(buflen + sizeof(char) * 50);
  sprintf(tmpbuf, "fig bg:bash-keybuffer %s %s 0 %d \"%s\"",
          fig_info->term_session_id, fig_info->fig_integration_version, index,
          buffer);
  size_t out_len;
  unsigned char *encoded =
      base64_encode((unsigned char *)tmpbuf, strlen(tmpbuf), &out_len);

  if (fig_sock < 0)
    fig_sock = unix_socket_connect("/tmp/fig.socket");

  if (fig_sock < 0)
    log_warn("Can't connect to fig socket");
  else
    send(fig_sock, encoded, out_len, 0);

  free(tmpbuf);
}

void loop(int ptyp, int ptyc_pid) {
  pid_t child;
  int nread;
  char buf[BUFFSIZE + 1];

  if ((child = fork()) < 0) {
    err_sys("fork error");
  } else if (child == 0) {
    child_loop(ptyp);
  }

  FigTerm *ft =
      figterm_new(true, &vterm_screen_callbacks, &vterm_parser_callbacks);
  ft->ptyp = ptyp;
  ft->ptyc_pid = ptyc_pid;
  log_info("Shell: %d", ptyc_pid);
  log_info("Child: %d", child);
  log_info("Parent: %d", getpid());

  if (set_sigaction(SIGWINCH, figterm_handle_winch) == SIG_ERR)
    err_sys("signal_intr error for SIGWINCH");

  if (set_sigaction(SIGTERM, sig_term) == SIG_ERR)
    err_sys("signal_intr error for SIGTERM");

  for (;;) {
    // Read from pty parent.
    nread = read(ptyp, buf, BUFFSIZE - 1);
    if (nread < 0 && errno == EINTR) {
      continue;
    } else if (nread < 0) {
      err_sys("read error from ptyp");
    }

    buf[nread] = '\0';

    log_info("Writing %.*s", nread, buf);
    int nrow, ncol;
    vterm_get_size(ft->vt, &nrow, &ncol);
    log_debug("%d, %d", nrow, ncol);

    vterm_input_write(ft->vt, buf, nread);
    VTermScreen *vts = vterm_obtain_screen(ft->vt);
    vterm_screen_flush_damage(vts);

    if (write(STDOUT_FILENO, buf, nread) != nread)
      err_sys("write error to stdout");

    if (!ft->wait_for_prompt) {
      int index;
      char *guess = extract_buffer(ft->state, ft->prompt_state, &index);

      if (guess != NULL) {
        if (index > 0) {
          publish_guess(index, guess);
          log_info("guess: %s\nindex: %d", guess, index);
        }
      } else {
        ft->wait_for_prompt = true;
        log_debug("Null guess, waiting for new prompt...");
        ft->prompt_state->cursor->row = -1;
        ft->prompt_state->cursor->col = -1;
      }
      free(guess);
    }
  }

  // Kill child if we read EOF on pty parent
  if (sigcaught == 0)
    kill(child, SIGTERM);

  // clean up
  figterm_free(ft);
}
