/*
 * FigtermScreen library.
 * Exposes similar bindings to VTermScreen, while giving us access to the lower
 * level state API. Using a custom screen class allows us to 
 * 1) Ignore unnecessary attributes in VTermScreen, e.g. a cell's color
 * 2) Add our own state to individual cells, e.g. whether a cell is part of a
 *    prompt.
 *
 * A majority of this library is static. Exposed methods should be used
 * primarily in figterm.c
 */
#include "fig.h"
#include "utf8.h"
#include "vterm.h"

#define UNICODE_SPACE 0x20
#define UNICODE_LINEFEED 0x0a
#define MAX_CHARS_PER_CELL 6
#define BUFIDX_PRIMARY   0
#define BUFIDX_ALTSCREEN 1

typedef struct {
  bool in_prompt;
  bool in_suggestion;
  VTermColor fg;
  VTermColor bg;
} ScreenAttrs;

typedef struct {
  uint32_t chars[MAX_CHARS_PER_CELL];
  ScreenAttrs attrs;
} ScreenCell;

struct FigTermScreen {
  VTermState *state;
  VTermPos *prompt_cursor;

  int rows;
  int cols;

  // Primary and altscreen.
  ScreenCell *buffers[2];

  // Current buffer, either primary or altscreen.
  ScreenCell *buffer;

  // Buffer for a single row used in scrollback storage.
  ScreenCell *sb_buffer;

  const FigTermScreenCallbacks* callbacks;
  void* cbdata;

  // Current attributes to write cells with.
  ScreenAttrs attrs;
};

static inline void setcell(const FigTermScreen *screen, ScreenCell *cell, uint32_t val) {
  // Set the value of a cell to a single value, also setting fig attributes.
  cell->chars[0] = val;
  cell->attrs = screen->attrs;
}

static inline void clearcell(const FigTermScreen *screen, ScreenCell *cell) {
  setcell(screen, cell, 0);
}

static inline ScreenCell *getcell(const FigTermScreen *screen, int row, int col) {
  if (row < 0 || row >= screen->rows)
    return NULL;
  if (col < 0 || col >= screen->cols)
    return NULL;
  return screen->buffer + (screen->cols * row) + col;
}

static ScreenCell *buffer_new(FigTermScreen *screen, int rows, int cols) {
  ScreenCell *new_buffer = malloc(sizeof(ScreenCell) * rows * cols);

  for (int row = 0; row < rows; row++) {
    for (int col = 0; col < cols; col++) {
      clearcell(screen, &new_buffer[row * cols + col]);
    }
  }

  return new_buffer;
}

static int putglyph(VTermGlyphInfo *info, VTermPos pos, void *user) {
  FigTermScreen *screen = user;
  ScreenCell *cell = getcell(screen, pos.row, pos.col);

  if (!cell)
    return 0;

  int i;
  for (i = 0; i < MAX_CHARS_PER_CELL && info->chars[i]; i++)
    cell->chars[i] = info->chars[i];

  if (i < MAX_CHARS_PER_CELL)
    cell->chars[i] = 0;

  cell->attrs = screen->attrs;

  // Gap behind double width char.
  for (int col = 1; col < info->width; col++)
    setcell(screen, getcell(screen, pos.row, pos.col + col), (uint32_t) -1);

  return 1;
}

static void sb_pushline(FigTermScreen *screen, int row) {
  for (int col = 0; col < screen->cols; col++)
    memcpy(screen->sb_buffer + col, getcell(screen, row, col), sizeof(ScreenCell));

  // TODO(sean) copy sb_buffer to sb line store.
}

static int sb_popline(FigTermScreen* screen) {
  // TODO(sean) copy from sb line store to sb buffer, up to screen->cols cols.
  return 0;
}

static int moverect(VTermRect dest, VTermRect src, void *user) {
  FigTermScreen *screen = user;

  if (dest.start_row == 0 && dest.start_col == 0 &&        // starts top-left corner
      dest.end_col == screen->cols &&                      // full width
      screen->buffer == screen->buffers[BUFIDX_PRIMARY]) { // not altscreen
    // Push to scroll back 
    for (int row = 0; row < src.start_row; row++)
      sb_pushline(screen, row);
  }

  int cols = src.end_col - src.start_col;
  int downward = src.start_row - dest.start_row;

  int first_row, last_row, inc_row;
  if (downward < 0) {
    first_row = dest.end_row - 1;
    last_row = dest.start_row - 1;
    inc_row  = -1;
  } else {
    first_row = dest.start_row;
    last_row = dest.end_row;
    inc_row  = +1;
  }

  for(int row = first_row; row != last_row; row += inc_row)
    memmove(getcell(screen, row, dest.start_col),
            getcell(screen, row + downward, src.start_col),
            cols * sizeof(ScreenCell));

  return 1;
}

static int erase(VTermRect rect, int selective, void *user) {
  FigTermScreen *screen = user;
  for (int row = rect.start_row; row < screen->rows && row < rect.end_row; row++) {
    for (int col = rect.start_col; col < rect.end_col; col++) {
      clearcell(screen, getcell(screen, row, col));
    }
  }

  return 1;
}

static int scrollrect(VTermRect rect, int downward, int rightward, void *user) {
  FigTermScreen *screen = user;
  if (rect.start_row == 0 && rect.end_row == screen->rows &&
      rect.start_col == 0 && rect.end_col == screen->cols &&
      screen->callbacks && screen->callbacks->scroll) {
    // If whole screen is scrolled then call scroll callback.
    screen->callbacks->scroll(-downward, screen->cbdata);
  }
  vterm_scroll_rect(rect, downward, rightward, moverect, erase, user);
  return 1;
}

static int settermprop(VTermProp prop, VTermValue *val, void *user) {
  FigTermScreen *screen = user;

  switch(prop) {
    case VTERM_PROP_ALTSCREEN:
      screen->buffer = val->boolean ? screen->buffers[BUFIDX_ALTSCREEN] : screen->buffers[BUFIDX_PRIMARY];
      break;
    default:
      ;
  }

  return 1;
}

static void resize_buffer(FigTermScreen *screen, int bufidx, int new_rows, int new_cols, bool active, VTermStateFields *statefields)
{
  int old_rows = screen->rows;
  int old_cols = screen->cols;
  int scroll_delta = 0;

  ScreenCell *old_buffer = screen->buffers[bufidx];
  ScreenCell *new_buffer = malloc(sizeof(ScreenCell) * new_rows * new_cols);

  int old_row = old_rows - 1;
  int new_row = new_rows - 1;

  // Starting at bottom copy over old rows until we run out of either.
  while (new_row >= 0 && old_row >= 0) {
    int col;
    for (col = 0; col < old_cols && col < new_cols; col++)
      new_buffer[new_row * new_cols + col] = old_buffer[old_row * old_cols + col];

    // Clear extra new columns if number of columns is increasing.
    for ( ; col < new_cols; col++)
      clearcell(screen, &new_buffer[new_row * new_cols + col]);

    old_row--;
    new_row--;

    // TODO(sean) understand this part.
    if (new_row < 0 && old_row >= 0 &&
        new_buffer[(new_rows - 1) * new_cols].chars[0] == 0 &&
        (!active || statefields->pos.row < (new_rows - 1))) {
      int moverows = new_rows - 1;
      memmove(&new_buffer[1 * new_cols], &new_buffer[0], moverows * new_cols * sizeof(ScreenCell));

      new_row++;
    }
  }

  // Extra lines from old row. Try pushing to scrollback.
  if (old_row >= 0 && bufidx == BUFIDX_PRIMARY) {
    for (int row = 0; row <= old_row; row++)
      sb_pushline(screen, row);
    if (active)
      scroll_delta -= (old_row + 1);
  }
  if (new_row >= 0 && bufidx == BUFIDX_PRIMARY) {
    /* Try to backfill rows by popping scrollback buffer */
    while (new_row >= 0) {
      if (!sb_popline(screen))
        break;

      int col = 0;
      for (col = 0; col < old_cols && col < new_cols; col++) {
        memcpy(&new_buffer[new_row * new_cols + col], &screen->sb_buffer[col], sizeof(ScreenCell));
      }
      for ( ; col < new_cols; col++)
        clearcell(screen, &new_buffer[new_row * new_cols + col]);
      new_row--;

      if (active)
        scroll_delta++;
    }
  }

  if (new_row >= 0) {
    int moverows = new_rows - new_row - 1;
    memmove(&new_buffer[0], &new_buffer[(new_row + 1) * new_cols], moverows * new_cols * sizeof(ScreenCell));

    for (new_row = moverows; new_row < new_rows; new_row++)
      for (int col = 0; col < new_cols; col++)
        clearcell(screen, &new_buffer[new_row * new_cols + col]);
  }

  free(old_buffer);
  screen->buffers[bufidx] = new_buffer;

  statefields->pos.row += scroll_delta;
  if (screen->callbacks && screen->callbacks->scroll)
    screen->callbacks->scroll(scroll_delta, screen->cbdata);

  return;
  // TODO(sean) handle reflow.
}

static int resize(int new_rows, int new_cols, VTermStateFields *fields, void *user) {
  FigTermScreen *screen = user;

  int altscreen_active = screen->buffer == screen->buffers[BUFIDX_ALTSCREEN];
  int old_cols = screen->cols;

  // Ensure sb_buffer can hold new or old rows.
  if (new_cols > old_cols) {
    if (screen->sb_buffer)
      free(screen->sb_buffer);
    screen->sb_buffer = malloc(sizeof(ScreenCell) * new_cols);
  }

  resize_buffer(screen, 0, new_rows, new_cols, !altscreen_active, fields);
  resize_buffer(screen, 1, new_rows, new_cols, altscreen_active, fields);

  screen->buffer = altscreen_active ? screen->buffers[BUFIDX_ALTSCREEN] : screen->buffers[BUFIDX_PRIMARY];

  screen->rows = new_rows;
  screen->cols = new_cols;

  // Ensure sb_buffer can hold new rows.
  if (new_cols <= old_cols) {
    if (screen->sb_buffer)
      free(screen->sb_buffer);
    screen->sb_buffer = malloc(sizeof(ScreenCell) * new_cols);
  }

  return 1;
}

static int movecursor(VTermPos pos, VTermPos oldpos, int visible, void *user) {
  FigTermScreen *screen = user;

  if (screen->callbacks && screen->callbacks->movecursor)
    return (*screen->callbacks->movecursor)(pos, oldpos, visible, screen->cbdata);

  return 0;
}

int setpenattr(VTermAttr attr, VTermValue *val, void *user) {
  FigTermScreen *screen = user;

  if (screen->callbacks && screen->callbacks->setpenattr)
    return (*screen->callbacks->setpenattr)(attr, val, screen->cbdata);

  return 0;
}

static VTermStateCallbacks state_cbs = {
  .putglyph    = &putglyph,
  .scrollrect  = &scrollrect,
  .erase       = &erase,
  .settermprop = &settermprop,
  .setpenattr =  &setpenattr,
  .resize      = &resize,
  .movecursor  = &movecursor
};

FigTermScreen *figterm_screen_new(VTerm *vt) {
  VTermState *state = vterm_obtain_state(vt);
  if (!state)
    return NULL;

  FigTermScreen *screen = malloc(sizeof(FigTermScreen));

  int rows, cols;
  vterm_get_size(vt, &rows, &cols);

  screen->state = state;

  screen->rows = rows;
  screen->cols = cols;

  screen->buffers[BUFIDX_PRIMARY] = buffer_new(screen, rows, cols);
  screen->buffers[BUFIDX_ALTSCREEN] = buffer_new(screen, rows, cols);

  screen->buffer = screen->buffers[BUFIDX_PRIMARY];

  screen->sb_buffer = malloc(sizeof(ScreenCell) * cols);

  screen->attrs.in_prompt = false;
  screen->attrs.in_suggestion = false;
  vterm_color_indexed(&screen->attrs.fg, 7);
  vterm_color_indexed(&screen->attrs.bg, 0);

  screen->callbacks = NULL;
  screen->cbdata = NULL;

  vterm_state_set_callbacks(screen->state, &state_cbs, screen);
  vterm_set_utf8(vt, true);

  return screen;
}

void figterm_screen_free(FigTermScreen *screen) {
  free(screen->buffers[BUFIDX_PRIMARY]);
  if (screen->buffers[BUFIDX_ALTSCREEN])
    free(screen->buffers[BUFIDX_ALTSCREEN]);

  free(screen->sb_buffer);
  free(screen);
}

void figterm_screen_set_unrecognised_fallbacks(FigTermScreen* screen, const VTermStateFallbacks *state_fallbacks, void* user) {
  vterm_state_set_unrecognised_fallbacks(screen->state, state_fallbacks, user);
}

void figterm_screen_reset(FigTermScreen* screen, int hard) {
  vterm_state_reset(screen->state, hard);
}

void figterm_screen_set_callbacks(FigTermScreen *screen, const FigTermScreenCallbacks *callbacks, void *user)
{
  screen->callbacks = callbacks;
  screen->cbdata = user;
}

void figterm_screen_get_cursorpos(FigTermScreen* screen, VTermPos* cursor) {
  vterm_state_get_cursorpos(screen->state, cursor);
}

void figterm_screen_set_attr(FigTermScreen* screen, FigTermAttr attr, void* val) {
  if (attr == FIGTERM_ATTR_IN_PROMPT) {
    screen->attrs.in_prompt = *((bool*) val);
  } else if (attr == FIGTERM_ATTR_IN_SUGGESTION) {
    screen->attrs.in_suggestion = *((bool*) val);
  } else if (attr == FIGTERM_ATTR_FOREGROUND) {
    screen->attrs.fg = *((VTermColor*) val);
  } else if (attr == FIGTERM_ATTR_BACKGROUND) {
    screen->attrs.bg = *((VTermColor*) val);
  }
}

void figterm_screen_get_attr(FigTermScreen* screen, FigTermAttr attr, VTermValue* val) {
  if (attr == FIGTERM_ATTR_IN_PROMPT) {
    val->boolean = screen->attrs.in_prompt;
  } else if (attr == FIGTERM_ATTR_IN_SUGGESTION) {
    val->boolean = screen->attrs.in_suggestion;
  } else if (attr == FIGTERM_ATTR_FOREGROUND) {
    val->color = screen->attrs.fg;
  } else if (attr == FIGTERM_ATTR_BACKGROUND) {
    val->color = screen->attrs.bg;
  }
}

size_t figterm_screen_get_text(FigTermScreen *screen, char *buffer, size_t len, const VTermRect rect, int start_col_offset, char mask, bool wrap_lines, int* index) {
  size_t outpos = 0;
  int padding = 0;
  VTermPos cursor;
  figterm_screen_get_cursorpos(screen, &cursor);

  if (index != NULL)
    *index = -1;

#define PUT(c)                                           \
  size_t thislen = utf8_seqlen(c);                       \
  if (buffer && outpos + thislen <= len)                 \
    outpos += fill_utf8((c), buffer + outpos);           \
  else                                                   \
    outpos += thislen;

  for (int row = rect.start_row; row < rect.end_row; row++) {
    bool last_char_was_padding = true;
    int start_col = rect.start_col + (row == rect.start_row ? start_col_offset : 0);

    for (int col = start_col; col < rect.end_col; col++) {
      if (index != NULL && row == cursor.row && col == cursor.col) {
        while (padding) {
          PUT(UNICODE_SPACE);
          padding--;
        }
        *index = outpos;
      }

      ScreenCell *cell = getcell(screen, row, col);

      if (cell->chars[0] == 0 ||
          (mask == UNICODE_SPACE && (cell->attrs.in_prompt || cell->attrs.in_suggestion))) {
        // Erased prompt or autosuggestion cell, might need a space.
        padding++;
        last_char_was_padding = true;
      } else if (cell->chars[0] == (uint32_t) -1) {
        // Gap behind a double-width char, do nothing.
      } else {
        while (padding) {
          PUT(UNICODE_SPACE);
          padding--;
        }
        if (mask && (cell->attrs.in_prompt || cell->attrs.in_suggestion)) {
          PUT(mask);
        } else {
          for (int i = 0; i < MAX_CHARS_PER_CELL && cell->chars[i]; i++) {
            PUT(cell->chars[i]);
          }
          last_char_was_padding = false;
        }
      }
    }

    if (row < rect.end_row - 1) {
      // Reset padding, adding only a linefeed if EOL reached without char.
      if (last_char_was_padding || !wrap_lines) {
        // If last char was a non-whitespace character, don't add end of line,
        // terminal text is wrapped without explicit \n character.
        PUT(UNICODE_LINEFEED);
      }
      padding = 0;
    }
  }

  return outpos;
}
