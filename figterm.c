#include "fig.h"
#include <vterm.h>

static FigTerm *_ft = NULL;

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

void preparser_reset(PreParserData* data) {
  data->has_internal_cmd = false;
  data->has_new_cmd = false;
}

FigTerm *figterm_new(bool utf8, VTermScreenCallbacks *screen_callbacks,
                     VTermParserCallbacks *parser_callbacks, int ptyc_pid, int ptyp) {
  if (_ft != NULL) {
    return _ft;
  }

  FigTerm *ft = malloc(sizeof(FigTerm));

  // Initialize vterm
  struct winsize w;
  get_winsize(&w);
  // ioctl(STDOUT_FILENO, TIOCGWINSZ, &w);

  VTerm *vt = vterm_new(w.ws_row, w.ws_col);

  ft->altscreen = false;

  ft->preparser_data = malloc(sizeof(PreParserData));
  preparser_reset(ft->preparser_data);

  ft->cursor = malloc(sizeof(VTermPos));
  ft->cursor->row = -1;
  ft->cursor->col = -1;

  vterm_set_utf8(vt, utf8);
  ft->state = term_state_new(vt);
  ft->prompt_state = term_state_new(vt);

  ft->in_prompt = false;
  ft->preexec = true;
  ft->in_internal = false;
  ft->vt = vt;

  // Used for resize.
  ft->ptyp = ptyp;
  ft->ptyc_pid = ptyc_pid;

  VTermScreen *vts = vterm_obtain_screen(vt);
  vterm_screen_set_callbacks(vts, screen_callbacks, ft);
  vterm_screen_set_unrecognised_fallbacks(vts, parser_callbacks, ft);
  vterm_screen_set_damage_merge(vts, VTERM_DAMAGE_ROW);

  vterm_screen_reset(vts, 1);

  _ft = ft;

  return ft;
}

void figterm_free(FigTerm *ft) {
  vterm_free(ft->vt);
  term_state_free(ft->state);
  term_state_free(ft->prompt_state);
  free(ft->preparser_data);
  free(ft->cursor);
  free(ft);
}

void figterm_resize(FigTerm *ft) {
  struct winsize ws;
  int nrow, ncol;

  get_winsize(&ws);
  if (ioctl(ft->ptyp, TIOCSWINSZ, &ws))
    err_sys("failed to set window size");

  vterm_get_size(ft->vt, &nrow, &ncol);

  ft->in_prompt = true;
  log_info("Resizing (%d, %d) -> (%d, %d)", nrow, ncol, ws.ws_row, ws.ws_col);
  vterm_set_size(ft->vt, ws.ws_row, ws.ws_col);
  ft->in_prompt = false;

  VTermRect rect = {.start_row = 0,
                    .end_row = ws.ws_row,
                    .start_col = 0,
                    .end_col = ws.ws_col};
  term_state_update(ft->state, ft->vt, rect, true);
  term_state_update(ft->prompt_state, ft->vt, rect, true);

  if (ft->ptyc_pid > 0) {
    kill(ft->ptyc_pid, SIGWINCH);
  }
}

// TODO(sean) This should probably not be done inside a signal handler,
// consider non-blocking i/o and setting a flag instead.
void figterm_handle_winch(int sig) { figterm_resize(_ft); }

static FigInfo *fig_info;

void set_fig_info(FigInfo *fi) { fig_info = fi; }
FigInfo *get_fig_info() { return fig_info; }
