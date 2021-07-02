#include "fig.h"
#include <vterm.h>

#define UNICODE_SPACE 0x20

static void handle_osc(FigTerm* ft) {
  // Handle osc after we received the final fragment.
  if (strcmp(ft->osc, "NewCmd") == 0) {
    figterm_screen_get_cursorpos(ft->screen, ft->cmd_cursor);
    log_info("Prompt at position: (%d, %d)", ft->cmd_cursor->row, ft->cmd_cursor->col);
    ft->preexec = false;
  } else if (strcmp(ft->osc, "StartPrompt") == 0) {
    bool in_prompt = true;
    figterm_screen_set_attr(ft->screen, FIGTERM_ATTR_IN_PROMPT, &in_prompt);
  } else if (strcmp(ft->osc, "EndPrompt") == 0) {
    bool in_prompt = false;
    figterm_screen_set_attr(ft->screen, FIGTERM_ATTR_IN_PROMPT, &in_prompt);
  } else if (strcmp(ft->osc, "PreExec") == 0) {
    ft->preexec = true;
  } else if (!strncmp(ft->osc, "Dir=", 4)) {
    log_info("In dir %s", ft->osc + 4);
  } else if (!strncmp(ft->osc, "Shell=", 6)) {
    // Only enable in bash for now.
    ft->shell_enabled = strcmp(ft->osc + 6, "bash") == 0;
  } else if (!strncmp(ft->osc, "TTY=", 4)) {
    strcpy(ft->tty, ft->osc + 4);
  } else if (!strncmp(ft->osc, "PID=", 4)) {
    strcpy(ft->pid, ft->osc + 4);
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
  ft->cmd_cursor->row += scroll_delta;
}

static VTermStateFallbacks state_fallbacks = {
    .osc = osc_cb,
};

static FigTermScreenCallbacks screen_callbacks = {
    .scroll = scroll_cb,
};

FigTerm *figterm_new(int ptyc_pid, int ptyp) {
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

  ft->pid[0] = '\0';
  ft->tty[0] = '\0';

  ft->osc = NULL;
  ft->parsing_osc = false;

  // Default to disabled until we see a shell prompt with shell info we
  // recognize.
  ft->shell_enabled = false;
  ft->preexec = true;
  ft->disable_figterm = false;

  // Used for resize.
  ft->ptyp = ptyp;
  ft->ptyc_pid = ptyc_pid;

  ft->vt = vt;
  FigTermScreen* screen = figterm_screen_new(vt);
  figterm_screen_set_callbacks(screen, &screen_callbacks, ft);
  figterm_screen_set_unrecognised_fallbacks(screen, &state_fallbacks, ft);
  figterm_screen_reset(screen, true);
  ft->screen = screen;

  ft->insertion_lock_path = fig_path("insertion-lock");

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
      ft->preexec || access(ft->insertion_lock_path, F_OK) == 0 ||
      i < 0)
    return NULL;

  int rows, cols;
  vterm_get_size(ft->vt, &rows, &cols);

  int len = (rows + 1 - i) * (cols + 1);
  char* buf = malloc(sizeof(char) * len);

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

  figterm_screen_get_text(ft->screen, buf + row_len, len - row_len, rect, UNICODE_SPACE, index_ptr);

  if (index_ptr != NULL)
    *index += row_len;

  return rtrim(buf, *index);
}

void figterm_resize(FigTerm* ft) {
  if (ft->ptyc_pid > 0)
    kill(ft->ptyc_pid, SIGWINCH);

  struct winsize ws;
  if (get_winsize(&ws) == -1 || ioctl(ft->ptyp, TIOCSWINSZ, &ws))
    err_sys("failed to set window size");

  if (ft->disable_figterm) return;
  vterm_set_size(ft->vt, ws.ws_row, ws.ws_col);
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

  log_info("\ntext:\n%.*s\ncursor pos: %d %d", outpos, buf, cursor.row, cursor.col);
  free(buf);
}

