#include "fig.h"
#include <ctype.h>
#include <vterm.h>

char *update_row(char *row, int *row_len, VTermRect rect, VTermScreen *vts) {
  char *new_row = row;
  if (rect.end_col == 0)
    return row;

  if (rect.end_col > *row_len) {
    new_row = realloc(row, sizeof(char) * rect.end_col);
    if (new_row == NULL)
      err_sys("Error in realloc");

    memset(new_row + *row_len, ' ', sizeof(char) * (rect.end_col - *row_len));
  }
  size_t outpos = vterm_screen_get_text(vts, new_row + rect.start_col,
                                        rect.end_col - rect.start_col, rect);

  *row_len = rect.end_col > *row_len ? rect.start_col + outpos : *row_len;
  return new_row;
}

VTermRect full_screen(VTerm *vt) {
  VTermRect rect = {.start_row = 0, .start_col = 0};
  vterm_get_size(vt, &rect.end_row, &rect.end_col);
  return rect;
}

void term_state_update(TermState *ts, VTerm *vt, VTermRect rect, bool reset) {
  int nrow, ncol;
  vterm_get_size(vt, &nrow, &ncol);
  if (reset || rect.end_row > ts->nrows) {
    log_debug("Term state update reset.");
    term_state_free_rows(ts);

    term_state_init_rows(ts, nrow);
    rect = full_screen(vt);
  }

  VTermScreen *vts = vterm_obtain_screen(vt);
  int end_row = rect.end_row;
  rect.end_col = ncol;
  for (int i = rect.start_row; i < end_row; i++) {
    rect.start_row = i;
    rect.end_row = i + 1;
    ts->rows[i] = update_row(ts->rows[i], ts->row_lens + i, rect, vts);
  }
}

void term_state_free_rows(TermState *ts) {
  for (int i = 0; i < ts->nrows; i++) {
    free(ts->rows[i]);
  }
  free(ts->row_lens);
  free(ts->rows);
}

void term_state_init_rows(TermState *ts, int nrow) {
  ts->rows = malloc(sizeof(char *) * nrow);

  if (ts->rows == NULL)
    err_sys("malloc error");

  ts->row_lens = malloc(sizeof(int) * nrow);
  if (ts->row_lens == NULL)
    err_sys("malloc error");

  ts->nrows = nrow;

  for (int i = 0; i < nrow; i++) {
    ts->rows[i] = NULL;
    ts->row_lens[i] = 0;
  }

  ts->scroll = 0;
}

TermState *term_state_new(VTerm *vt) {
  TermState *ts = malloc(sizeof(TermState));
  ts->cursor = malloc(sizeof(VTermPos));
  ts->cursor->row = -1;
  ts->cursor->col = -1;

  int nrow, ncol;
  vterm_get_size(vt, &nrow, &ncol);
  term_state_init_rows(ts, nrow);
  return ts;
}

void term_state_free(TermState *ts) {
  free(ts->cursor);
  term_state_free_rows(ts);
  free(ts);
}

void term_state_update_cursor(TermState *ts, const VTermPos pos) {
  ts->cursor->row = pos.row;
  ts->cursor->col = pos.col;
}

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

  if (state->cursor->row == i && state->cursor->col == j) {
    *index = 0;
  }

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
        *index = pos + 1;
      }
      text[pos++] = c;
    }
    text[pos++] = '\n';
    j = 0;
  }
  text[pos] = '\0';
  return rtrim(text, *index);
}

void print_term_state(TermState *ts, bool is_prompt) {
  log_debug("text:");
  for (int i = 0; i < ts->nrows; i++) {
    log_debug("%.*s", ts->row_lens[i], ts->rows[i]);
  }
  log_debug("cursor pos: %d %d", ts->cursor->row, ts->cursor->col);
  log_debug("scrollback: %d", ts->scroll);
  log_debug("is_prompt: %s", is_prompt ? "true" : "false");
}
