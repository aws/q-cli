#include "fig.h"
#include <vterm.h>

static FigTerm *_ft = NULL;

FigTerm *figterm_new(bool utf8, VTermScreenCallbacks *screen_callbacks,
                     VTermStateFallbacks *parser_callbacks, int ptyc_pid, int ptyp) {
  VTerm* vt;
  FigTerm *ft;
  struct winsize w;

  if (_ft != NULL)
    return _ft;

  if ((ft = malloc(sizeof(FigTerm))) == NULL)
    goto error;

  // Initialize vterm
  if (get_winsize(&w) == -1)
    goto error;

  if ((vt = vterm_new(w.ws_row, w.ws_col)) == NULL)
    goto error;

  if ((ft->state = term_state_new(vt)) == NULL)
    goto error;

  if ((ft->prompt_state = term_state_new(vt)) == NULL)
    goto error;

  if ((ft->cursor = malloc(sizeof(VTermPos))) == NULL)
    goto error;

  ft->cursor->row = -1;
  ft->cursor->col = -1;

  // Default to disabled until we see a shell prompt with shell info we
  // recognize.
  ft->shell_enabled = false;

  ft->pid[0] = '\0';
  ft->tty[0] = '\0';
  ft->osc = NULL;
  ft->altscreen = false;
  ft->parsing_osc = false;
  ft->in_prompt = false;
  ft->preexec = true;
  ft->vt = vt;
  ft->disable_figterm = false;

  // Used for resize.
  ft->ptyp = ptyp;
  ft->ptyc_pid = ptyc_pid;

  vterm_set_utf8(vt, utf8);

  VTermScreen *vts = vterm_obtain_screen(vt);
  vterm_screen_set_callbacks(vts, screen_callbacks, ft);
  vterm_screen_set_unrecognised_fallbacks(vts, parser_callbacks, ft);
  vterm_screen_set_damage_merge(vts, VTERM_DAMAGE_ROW);

  vterm_screen_reset(vts, 1);

  _ft = ft;

  return ft;

error:
  figterm_free(ft);
  return NULL;
}

void figterm_free(FigTerm *ft) {
  if (ft != NULL) {
    vterm_free(ft->vt);
    term_state_free(ft->state);
    term_state_free(ft->prompt_state);
    free(ft->cursor);
    free(ft->osc);
  }
  free(ft);
}

void figterm_resize(FigTerm *ft) {
  struct winsize ws;
  int nrow, ncol;

  if (get_winsize(&ws) == -1 || ioctl(ft->ptyp, TIOCSWINSZ, &ws))
    err_sys("failed to set window size");

  if (ft->ptyc_pid > 0) {
    kill(ft->ptyc_pid, SIGWINCH);
  }

  if (ft->disable_figterm) return;

  vterm_get_size(ft->vt, &nrow, &ncol);

  ft->in_prompt = true;
  log_info("Resizing (%d, %d) -> (%d, %d)", nrow, ncol, ws.ws_row, ws.ws_col);
  vterm_set_size(ft->vt, ws.ws_row, ws.ws_col);
  ft->in_prompt = false;

  VTermRect rect = {.start_row = 0,
                    .end_row = ws.ws_row,
                    .start_col = 0,
                    .end_col = ws.ws_col};
  if (term_state_update(ft->state, ft->vt, rect, true) == -1)
    ft->disable_figterm = true;
  if (term_state_update(ft->prompt_state, ft->vt, rect, true) == -1)
    ft->disable_figterm = true;
}

// TODO(sean) This should probably not be done inside a signal handler,
// consider non-blocking i/o and setting a flag instead.
void figterm_handle_winch(int sig) { figterm_resize(_ft); }
