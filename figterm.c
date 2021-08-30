#include "fig.h"
#include <vterm.h>

#define UNICODE_SPACE 0x20
#define strneq(a,b,n) (strncmp(a,b,n)==0)

struct FigTerm {
  VTerm* vt;
  FigTermScreen* screen;
  VTermPos *cmd_cursor;
  char* insertion_lock_path;

  char* osc;
  bool parsing_osc;

  FigShellState shell_state;

  bool shell_enabled;

  // Turn off figterm if there's an error.
  bool disable_figterm;

  // Whether or not we've seen the first prompt.
  bool has_seen_prompt;

  // Used for resizing.
  int ptyp_fd;
  int shell_pid;
};

static void handle_osc(FigTerm* ft) {
  // Handle osc after we received the final fragment.
  if (strcmp(ft->osc, "NewCmd") == 0) {
    figterm_screen_get_cursorpos(ft->screen, ft->cmd_cursor);
    log_info("Prompt at position: (%d, %d)", ft->cmd_cursor->row, ft->cmd_cursor->col);
    ft->shell_state.preexec = false;
  } else if (strcmp(ft->osc, "StartPrompt") == 0) {
    ft->shell_state.in_prompt = true;
    figterm_screen_set_attr(ft->screen, FIGTERM_ATTR_IN_PROMPT, &ft->shell_state.in_prompt);
    ft->has_seen_prompt = true;
  } else if (strcmp(ft->osc, "EndPrompt") == 0) {
    ft->shell_state.in_prompt = false;
    figterm_screen_set_attr(ft->screen, FIGTERM_ATTR_IN_PROMPT, &ft->shell_state.in_prompt);
  } else if (strcmp(ft->osc, "PreExec") == 0) {
    ft->shell_state.preexec = true;
  } else if (strneq(ft->osc, "Dir=", 4)) {
    log_info("In dir %s", ft->osc + 4);
    if (!ft->shell_state.in_ssh) {
      // change figterm cwd to match shell.
      chdir(ft->osc + 4);
    }
  } else if (strneq(ft->osc, "Shell=", 6)) {
    // Only enable in bash for now.
    ft->shell_enabled = strcmp(ft->osc + 6, "bash") == 0;
  } else if (strneq(ft->osc, "TTY=", 4)) {
    strcpy(ft->shell_state.tty, ft->osc + 4);
  } else if (strneq(ft->osc, "PID=", 4)) {
    strcpy(ft->shell_state.pid, ft->osc + 4);
  } else if (strneq(ft->osc, "Log=", 4)) {
    if (strcmp(ft->osc + 4, "DEBUG") == 0) {
      set_logging_level(LOG_DEBUG);
    } else if (strcmp(ft->osc + 4, "INFO") == 0) {
      set_logging_level(LOG_INFO);
    } else if (strcmp(ft->osc + 4, "ERROR") == 0) {
      set_logging_level(LOG_ERROR);
    } else if (strcmp(ft->osc + 4, "FATAL") == 0) {
      set_logging_level(LOG_FATAL);
    } else {
      // Default to WARN.
      set_logging_level(LOG_WARN);
    }
  } else if (strneq(ft->osc, "SSH=", 4)) {
    ft->shell_state.in_ssh = ft->osc[5] == '1';
  }
}

static int osc_cb(int command, VTermStringFragment frag, void *user) {
  // Piece fragments of an osc together.
  if (command == 697) {
    FigTerm *ft = user;
    if (frag.initial) {
      ft->parsing_osc = true;
      free(ft->osc);
      ft->osc = malloc(sizeof(char) * (frag.len + 1));
      strncpy(ft->osc, frag.str, frag.len);
      ft->osc[frag.len] = '\0';
    } else if (ft->parsing_osc) {
      // TODO(sean) handle failure in realloc.
      ft->osc = realloc(ft->osc, strlen(ft->osc) + sizeof(char) * (frag.len + 1));
      strncat(ft->osc, frag.str, frag.len);
    } 

    if (frag.final) {
      log_debug("OSC CB: %s", ft->osc);
      ft->parsing_osc = false;

      handle_osc(ft);

      free(ft->osc);
      ft->osc = NULL;
    }

  }
  return 0;
}

static void scroll_cb(int scroll_delta, void* user) {
  FigTerm* ft = user;
  log_debug("Scroll cb %d+%d", ft->cmd_cursor->row, scroll_delta);
  ft->cmd_cursor->row += scroll_delta;
}

static int movecursor_cb(VTermPos pos, VTermPos oldpos, int visible, void *user) {
  FigTerm *ft = user;

  if (pos.col == 0 || oldpos.col == 0) {
    // On or after linefeed, update cwd to match shell's.
    char* cwd = get_cwd(ft->shell_pid);
    log_debug("cwd (%d, %d) -> (%d, %d): %s", oldpos.row, oldpos.col, pos.row, pos.col, cwd);
    chdir(cwd);
    free(cwd);
  }

  return 0;
}

static VTermStateFallbacks state_fallbacks = {
    .osc = osc_cb,
};

static FigTermScreenCallbacks screen_callbacks = {
    .scroll = scroll_cb,
    .movecursor = movecursor_cb,
};

FigTerm *figterm_new(int shell_pid, int ptyp_fd) {
  VTerm* vt;
  FigTerm *ft;
  struct winsize w;

  if ((ft = malloc(sizeof(FigTerm))) == NULL)
    goto error;

  // Initialize vterm
  if (get_winsize(&w) == -1)
    goto error;

  if ((vt = vterm_new(w.ws_row, w.ws_col)) == NULL)
    goto error;

  VTermPos* cmd_cursor = malloc(sizeof(VTermPos));
  cmd_cursor->row = -1;
  cmd_cursor->col = -1;
  ft->cmd_cursor = cmd_cursor;
  ft->insertion_lock_path = fig_path("insertion-lock");

  ft->osc = NULL;
  ft->parsing_osc = false;

  ft->shell_state.pid[0] = '\0';
  ft->shell_state.tty[0] = '\0';
  ft->shell_state.in_ssh = false;
  ft->shell_state.preexec = true;
  ft->shell_state.in_prompt = false;

  // Default to disabled until we see a shell prompt with shell info we
  // recognize.
  ft->shell_enabled = false;

  ft->disable_figterm = false;

  ft->has_seen_prompt = false;

  // Used for resize.
  ft->ptyp_fd = ptyp_fd;
  ft->shell_pid = shell_pid;

  ft->vt = vt;
  FigTermScreen* screen = figterm_screen_new(vt);
  figterm_screen_set_callbacks(screen, &screen_callbacks, ft);
  figterm_screen_set_unrecognised_fallbacks(screen, &state_fallbacks, ft);
  figterm_screen_reset(screen, true);
  ft->screen = screen;

  return ft;

error:
  figterm_free(ft);
  return NULL;
}

void figterm_free(FigTerm *ft) {
  if (ft != NULL) {
    vterm_free(ft->vt);
    figterm_screen_free(ft->screen);
    free(ft->cmd_cursor);
    free(ft->osc);
    free(ft->insertion_lock_path);
  }
  free(ft);
}

char* figterm_get_buffer(FigTerm* ft, int* index) {
  int i = ft->cmd_cursor->row;
  int j = ft->cmd_cursor->col;

  if (ft->disable_figterm || !ft->shell_enabled ||
      ft->shell_state.preexec || access(ft->insertion_lock_path, F_OK) == 0 ||
      i < 0)
    return NULL;

  int rows, cols;
  vterm_get_size(ft->vt, &rows, &cols);

  int len = (rows + 1 - i) * (cols + 1);
  char* buf = malloc(sizeof(char) * (len + 1));

  int* index_ptr = index;

  // Get prompt row text first.
  VTermRect rect = {
    .start_row = i, .end_row = i + 1, .start_col = j, .end_col = cols
  };
  size_t row_len = figterm_screen_get_text(ft->screen, buf, len, rect, UNICODE_SPACE, index_ptr);

  if (*index_ptr != -1)
    index_ptr = NULL;

  // Then the rest of the screen.
  rect.start_row += 1;
  rect.end_row = rows;
  rect.start_col = 0;
  rect.end_col = cols;

  size_t text_len = figterm_screen_get_text(ft->screen, buf + row_len, len - row_len, rect, UNICODE_SPACE, index_ptr);
  buf[row_len + text_len] = '\0';

  if (index_ptr != NULL)
    *index += row_len;

  return rtrim(buf, *index);
}

void figterm_resize(FigTerm* ft) {
  if (ft->shell_pid > 0)
    kill(ft->shell_pid, SIGWINCH);

  struct winsize ws;
  if (get_winsize(&ws) == -1 || ioctl(ft->ptyp_fd, TIOCSWINSZ, &ws))
    err_sys("failed to set window size");

  if (ft->disable_figterm) return;
  vterm_set_size(ft->vt, ws.ws_row, ws.ws_col);
}

void figterm_get_shell_state(FigTerm* ft, FigShellState* shell_state) {
  memcpy(shell_state, &ft->shell_state, sizeof(FigShellState));
}

void figterm_log(FigTerm *ft, char mask) {
  // Output text of figterm screen, optionally masking prompt cells with mask.
  // Passing mask=0 will display prompts normally.
  VTermRect rect = {.start_row = 0, .start_col = 0};
  vterm_get_size(ft->vt, &rect.end_row, &rect.end_col);
  int len = (rect.end_row + 1) * (rect.end_col + 1);
  char* buf = malloc(sizeof(char) * len);
  size_t outpos = figterm_screen_get_text(ft->screen, buf, len, rect, mask, NULL);

  VTermPos cursor;
  figterm_screen_get_cursorpos(ft->screen, &cursor);

  log_debug("\ntext:\n%.*s\ncursor pos: %d %d", outpos, buf, cursor.row, cursor.col);
  free(buf);
}

void figterm_write(FigTerm* ft, char* buf, int n) {
  log_debug("Writing %d chars %.*s", n, n, buf);
  vterm_input_write(ft->vt, buf, n);
}

bool figterm_is_disabled(FigTerm* ft) {
  return ft == NULL || ft->disable_figterm;
}

bool figterm_has_seen_prompt(FigTerm* ft) {
  return ft != NULL && ft->has_seen_prompt;
}
