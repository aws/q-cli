#include "fig.h"
#include <ctype.h>
#include <errno.h>
#include <fcntl.h>
#include <math.h>
#include <stdio.h>
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

static volatile sig_atomic_t sigcaught;

// Called when child sends us SIGTERM
static void sig_term(int signo) { sigcaught = 1; }

char *update_row(char *row, int *row_len, VTermRect rect, VTermScreen *vts) {
  char *new_row = row;
  if (rect.end_col == 0)
    return row;

  if (rect.end_col > *row_len) {
    // TODO(sean) assert that start_col is less than or equal to row_len
    new_row = realloc(row, sizeof(char) * rect.end_col);
    if (new_row == NULL)
      err_sys("Error in realloc");

    memset(new_row + *row_len, ' ', sizeof(char) * (rect.end_col - *row_len));
  }
  // TODO(sean) segfaults on resize when new_cols > old_cols and new_rows <
  // old_rows
  size_t outpos = vterm_screen_get_text(vts, new_row + rect.start_col,
                                        rect.end_col - rect.start_col, rect);

  *row_len = rect.end_col > *row_len ? rect.start_col + outpos : *row_len;
  return new_row;
}

VTermRect full_screen(VTerm *vt) {
  int nrow, ncol;
  vterm_get_size(vt, &nrow, &ncol);

  VTermRect rect = {
      .start_row = 0,
      .end_row = nrow,
      .start_col = 0,
      .end_col = ncol,
  };
  return rect;
}

void term_state_update(TermState *ts, VTerm *vt, VTermRect rect, bool reset) {
  if (rect.end_row > ts->nrows || reset) {
    log_info("Term state update reset.");
    term_state_free_rows(ts);

    int nrow, ncol;
    vterm_get_size(vt, &nrow, &ncol);
    term_state_init_rows(ts, nrow);
    rect = full_screen(vt);
  }

  VTermScreen *vts = vterm_obtain_screen(vt);
  int end_row = rect.end_row;
  for (int i = rect.start_row; i < end_row; i++) {
    rect.start_row = i;
    rect.end_row = i + 1;
    ts->rows[i] = update_row(ts->rows[i], ts->row_lens + i, rect, vts);
  }
}

char *extract_buffer(TermState *state, TermState *prompt_state, int *index) {
  int i = prompt_state->cursor->row - prompt_state->scroll;
  int j = prompt_state->cursor->col;

  if (i < 0)
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

void write_output_to_vterm(VTerm *vt, char *buf, int len) {
  log_info("Received: %.*s", len, buf);
  vterm_input_write(vt, buf, len);
  VTermScreen *vts = vterm_obtain_screen(vt);
  vterm_screen_flush_damage(vts);
}

static void print_term_state(TermState *ts) {
  log_debug("text: ");
  for (int i = 0; i < ts->nrows; i++) {
    log_debug("%.*s", ts->row_lens[i], ts->rows[i]);
  }
  log_debug("cursor pos: %d %d", ts->cursor->row, ts->cursor->col);
}

static void print_guess(FigTerm *u) {
  if (u->prompt_state->cursor->row - u->prompt_state->scroll < 0) {
    log_debug("NULL GUESS");
    return;
  }
  print_term_state(u->state);
  print_term_state(u->prompt_state);
  log_info("last_prompt_pos: %d %d", u->prompt_state->cursor->row,
           u->prompt_state->cursor->col);
  log_info("prompt_sb: %d", u->prompt_state->scroll);
  int index;
  char *guess = extract_buffer(u->state, u->prompt_state, &index);
  if (guess != NULL) {
    log_info("guess: %s\nindex: %d", guess, index);
    free(guess);
  } else {
    log_debug("NULL GUESS");
  }
}

static int movecursor_cb(VTermPos pos, VTermPos oldpos, int visible,
                         void *user) {
  FigTerm *u = user;
  term_state_update_cursor(u->state, pos);
  log_info("Move cursor, %d", u->update_prompt);
  if (u->update_prompt)
    term_state_update_cursor(u->prompt_state, pos);
  return 0;
}

static int sb_pushline_cb(int cols, const VTermScreenCell *cells, void *user) {
  FigTerm *u = user;
  u->prompt_state->scroll += 1;
  return 0;
}

static int sb_popline_cb(int cols, VTermScreenCell *cells, void *user) {
  FigTerm *u = user;
  u->prompt_state->scroll -= 1;
  return 0;
}

static int damage_cb(VTermRect rect, void *user) {
  FigTerm *ft = user;
  // log_debug("DAMAGE %d, %d: %d, %d", rect.start_row, rect.end_row,
  // rect.start_col, rect.end_col);
  term_state_update(ft->state, ft->vt, rect, false);
  if (ft->update_prompt) {
    // log_debug("DAMAGE PROMPT");
    term_state_update(ft->prompt_state, ft->vt, rect, !ft->is_resizing);
  }

  int prompt_row = ft->prompt_state->cursor->row;
  int prompt_col = ft->prompt_state->cursor->col;
  if (!ft->update_prompt &&
      (rect.end_row > prompt_row ||
       (rect.end_row == prompt_row && rect.end_col > prompt_col))) {
    print_guess(ft);
  }
  return 0;
}

static VTermScreenCallbacks vterm_screen_callbacks = {
    .damage = damage_cb,
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

int set_non_blocking(int fd) {
  int flags;
  // If they have O_NONBLOCK, use the Posix way to do it
#if defined(O_NONBLOCK)
  // TODO(sean): O_NONBLOCK is defined but broken on SunOS 4.1.x and AIX 3.2.5.
  if (-1 == (flags = fcntl(fd, F_GETFL, 0)))
    flags = 0;
  return fcntl(fd, F_SETFL, flags | O_NONBLOCK);
#else
  // Otherwise, use the old way of doing it */
  flags = 1;
  return ioctl(fd, FIOBIO, &flags);
#endif
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
      err_sys("write error to master pty");
  }

  // notify parent process on exit.
  kill(getppid(), SIGTERM);
  exit(0);
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

  if (set_non_blocking(ptyp) < 0)
    err_sys("Unable to set nonblocking");

  size_t new_cmd_len, start_skip_len, end_skip_len;
  char *new_cmd_secret = make_secret_text(FIG_NEW_CMD, &new_cmd_len);
  char *start_skip_secret = make_secret_text(FIG_START_SKIP, &start_skip_len);
  char *end_skip_secret = make_secret_text(FIG_END_SKIP, &end_skip_len);

  int status;
  FigTerm *ft = figterm_new(true, &vterm_screen_callbacks);
  ft->ptyp = ptyp;

  if (set_sigaction(SIGWINCH, figterm_handle_winch) == SIG_ERR)
    err_sys("signal_intr error for SIGWINCH");

  if (set_sigaction(SIGTERM, sig_term) == SIG_ERR)
    err_sys("signal_intr error for SIGTERM");

  for (;;) {
    // Check if pty child is finished.
    pid_t return_pid = waitpid(ptyc_pid, &status, WNOHANG);
    if (return_pid == -1 || return_pid == ptyc_pid)
      break;

    // Check for resize.
    if (figterm_should_resize()) {
      kill(ptyc_pid, SIGWINCH);
      figterm_resize(ft);
    }

    // Read from pty parent.
    nread = read(ptyp, buf, BUFFSIZE - 1);
    if (nread == -1 && errno == EAGAIN) {
      // Empty pipe, continue to other checks.
      continue;
    } else if (nread <= 0) {
      // EOF or read error, exit.
      break;
    }

    buf[nread] = '\0';

    char *end_prompt, *start_prompt;
    char *p = buf;
    int n = nread;
    log_debug("Read from pty: %.*s", n, p);
    if ((end_prompt = strrstr(p, new_cmd_secret, n, new_cmd_len)) != NULL) {
      log_debug("New prompt");
      // Found new_cmd prompt (PS1/PS3). First print prompt and store screen,
      // then print remaining buffer.
      int prompt_len = end_prompt - p;
      ft->update_prompt = true;
      write_output_to_vterm(ft->vt, p, prompt_len);
      ft->update_prompt = false;

      p = end_prompt + new_cmd_len;
      n = nread - (p - buf);
    }

    log_debug("Post prompt %.*s", n, p);
    if (((start_prompt = strstr(p, start_skip_secret)) != NULL) &&
        ((end_prompt = strrstr(p, end_skip_secret, n, end_skip_len)) != NULL)) {
      log_debug("Skip");
      // Skip
      // TODO(sean) assumes prompt is sent atomically
      /*
      int pre_prompt_len = start_prompt - p;
      log_info("Pre prompt");
      write_output_to_vterm(ft->vt, p, pre_prompt_len);

      // Found prompt continuation (PS2). Treat it as empty string.
      p = end_prompt + end_skip_len;
      n = nread - (p - buf);
      */
    } else {
      // Print remaining buffer to stdout normally.
      write_output_to_vterm(ft->vt, p, n);
    }
    log_debug("Post skip: %.*s", n, p);

    if (write(STDOUT_FILENO, buf, nread) != nread)
      err_sys("write error to stdout");
  }

  // Kill child if we read EOF on pty parent
  if (sigcaught == 0)
    kill(child, SIGTERM);

  // clean up
  figterm_free(ft);
  free(new_cmd_secret);
  free(end_skip_secret);
  free(start_skip_secret);
}
