#include "fig.h"
#include <vterm.h>

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
  VTermState *state = vterm_obtain_state(vt);
  vterm_state_get_cursorpos(state, ts->cursor);

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
