#include "fig.h"

static int should_resize;

void get_winsize(struct winsize *ws) {
  const char *term = ctermid(NULL);
  log_debug("term %s", term);
  if (!term[0]) {
    err_sys("can't get name of controlling terminal");
  }
  int fd = open(term, O_RDONLY);
  if (fd == -1) {
    err_sys("can't open terminal at %s", term);
  }
  if (ioctl(fd, TIOCGWINSZ, ws) == -1) {
    err_sys("can't get the window size of %s", term);
  }
  close(fd);
}

FigTerm *figterm_new(bool utf8, VTermScreenCallbacks *screen_callbacks) {
  FigTerm *ft = malloc(sizeof(FigTerm));

  // Initialize vterm
  struct winsize w;
  get_winsize(&w);
  // ioctl(STDOUT_FILENO, TIOCGWINSZ, &w);

  VTerm *vt = vterm_new(w.ws_row, w.ws_col);

  ft->screen_rows = w.ws_row;
  ft->screen_cols = w.ws_col;

  vterm_set_utf8(vt, utf8);
  ft->state = term_state_new(vt);
  ft->prompt_state = term_state_new(vt);

  ft->update_prompt = false;
  ft->is_resizing = false;
  ft->vt = vt;

  VTermScreen *vts = vterm_obtain_screen(vt);
  vterm_screen_set_callbacks(vts, screen_callbacks, ft);
  vterm_screen_set_damage_merge(vts, VTERM_DAMAGE_ROW);

  vterm_screen_reset(vts, 1);

  return ft;
}

void figterm_free(FigTerm *ft) {
  vterm_free(ft->vt);
  term_state_free(ft->state);
  term_state_free(ft->prompt_state);
  free(ft);
}

void figterm_resize(FigTerm *ft) {
  struct winsize window_size;
  int nrow, ncol;

  get_winsize(&window_size);
  // ioctl(STDIN_FILENO, TIOCGWINSZ, &window_size);
  ioctl(ft->ptyp, TIOCSWINSZ, &window_size);
  vterm_get_size(ft->vt, &nrow, &ncol);
  log_debug("RESIZING %d, %d -> %d, %d", nrow, ncol, window_size.ws_row,
            window_size.ws_col);
  ft->is_resizing = true;
  ft->update_prompt = true;
  vterm_set_size(ft->vt, window_size.ws_row, window_size.ws_col);
  ft->update_prompt = false;
  ft->is_resizing = false;

  // VTermRect rect = { .start_row = 0, .end_row = window_size.ws_row,
  // .start_col = 0, .end_col = window_size.ws_col };
  // term_state_update(ft->state, ft->vt, rect, true);
  // term_state_update(ft->prompt_state, ft->vt, rect, true);

  should_resize = 0;
}

void figterm_handle_winch(int sig) { should_resize = 1; }

int figterm_should_resize() { return should_resize; }
