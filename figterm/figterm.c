#include "fig.h"
#include <sys/fcntl.h>
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

  // Turn off figterm if there's an error.
  bool disable_figterm;

  // Whether or not we've seen the first prompt.
  bool has_seen_prompt;

  // Used for resizing.
  int ptyp_fd;
  int shell_pid;

  HistoryEntry* last_command;
};

static void handle_osc(FigTerm* ft) {
  // Handle osc after we received the final fragment.
  if (strcmp(ft->osc, "NewCmd") == 0) {
    char* context = printf_alloc(
      "{\"sessionId\":\"%s\",\"pid\":\"%s\",\"hostname\":\"%s\",\"ttys\":\"%s\"}",
      ft->shell_state.session_id,
      ft->shell_state.pid,
      ft->shell_state.hostname,
      ft->shell_state.tty
    );

    publish_json(
      "{\"hook\":{\"prompt\":{\"context\": %s}}}",
      context
    );
    free(context);

    figterm_screen_get_cursorpos(ft->screen, ft->cmd_cursor);
    log_info("Prompt at position: (%d, %d)", ft->cmd_cursor->row, ft->cmd_cursor->col);
    ft->shell_state.preexec = false;
    if (ft->last_command != NULL) {
      // TODO(sean) this won't work super well for first/last commands in a new
      // shell, ssh, docker etc.
      write_history_entry(ft->last_command);
      history_entry_free(ft->last_command);
      ft->last_command = NULL;
    }
  } else if (strcmp(ft->osc, "StartPrompt") == 0) {
    ft->shell_state.in_prompt = true;
    figterm_screen_set_attr(ft->screen, FIGTERM_ATTR_IN_PROMPT, &ft->shell_state.in_prompt);
    ft->has_seen_prompt = true;
  } else if (strcmp(ft->osc, "EndPrompt") == 0) {
    ft->shell_state.in_prompt = false;
    figterm_screen_set_attr(ft->screen, FIGTERM_ATTR_IN_PROMPT, &ft->shell_state.in_prompt);
  } else if (strcmp(ft->osc, "PreExec") == 0) {
    publish_message("fig bg:exec %d %s\n", ft->shell_state.pid, ft->shell_state.tty);
    char* context = printf_alloc(
      "{\"sessionId\":\"%s\",\"pid\":\"%s\",\"hostname\":\"%s\",\"ttys\":\"%s\"}",
      ft->shell_state.session_id,
      ft->shell_state.pid,
      ft->shell_state.hostname,
      ft->shell_state.tty
    );

    publish_json(
      "{\"hook\":{\"preExec\":{\"context\": %s}}}",
      context
    );
    figterm_preexec_hook(ft);
    ft->shell_state.preexec = true;
  } else if (strneq(ft->osc, "Dir=", 4)) {
    log_info("In dir %s", ft->osc + 4);
    if (!ft->shell_state.in_ssh) {
      // change figterm cwd to match shell.
      chdir(ft->osc + 4);
    }
  } else if (strneq(ft->osc, "ExitCode=", 9)) {
    if (ft->last_command != NULL) {
      int exit_code;
      sscanf(ft->osc + 9, "%d", &exit_code);
      history_entry_set_exit_code(ft->last_command, exit_code);
    }
  } else if (strneq(ft->osc, "Shell=", 6)) {
    strcpy(ft->shell_state.shell, ft->osc + 6);
  } else if (strneq(ft->osc, "FishSuggestionColor=", 20)) {
    figterm_update_fish_suggestion_color(ft, ft->osc + 20);
  } else if (strneq(ft->osc, "TTY=", 4)) {
    strcpy(ft->shell_state.tty, ft->osc + 4);
  } else if (strneq(ft->osc, "PID=", 4)) {
    strcpy(ft->shell_state.pid, ft->osc + 4);
  } else if (strneq(ft->osc, "SessionId=", 10)) {
    strncpy(ft->shell_state.session_id, ft->osc + 10, SESSION_ID_MAX_LEN);
    ft->shell_state.session_id[SESSION_ID_MAX_LEN] = '\0';
  } else if (strneq(ft->osc, "Docker=", 7)) {
    ft->shell_state.in_docker = ft->osc[7] == '1';
  } else if (strneq(ft->osc, "Hostname=", 7)) {
    free(ft->shell_state.hostname);
    ft->shell_state.hostname = strdup(ft->osc + 7);
  } else if (strneq(ft->osc, "Log=", 4)) {
    set_logging_level_from_string(ft->osc + 4);
  } else if (strneq(ft->osc, "SSH=", 4)) {
    ft->shell_state.in_ssh = ft->osc[4] == '1';
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
      log_info("OSC CB: %s", ft->osc);
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
    chdir(cwd);
    free(cwd);
  }

  return 0;
}

static int setpenattr_cb(VTermAttr attr, VTermValue *val, void *user) {
  FigTerm *ft = user;
  bool in_suggestion = false;

  switch(attr) {
    case VTERM_ATTR_FOREGROUND:
      if (ft->shell_state.fish_suggestion_color != NULL &&
          vterm_color_is_equal(&val->color, ft->shell_state.fish_suggestion_color)) {
        in_suggestion = true;
      }
      figterm_screen_set_attr(ft->screen, FIGTERM_ATTR_IN_SUGGESTION, &in_suggestion);
      return 1;

    default:
      return 0;
  }

  return 0;
}

static VTermStateFallbacks state_fallbacks = {
  .osc = osc_cb,
};

static FigTermScreenCallbacks screen_callbacks = {
  .scroll = scroll_cb,
  .movecursor = movecursor_cb,
  .setpenattr = setpenattr_cb,
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

  FigInfo *fig_info = get_fig_info();

  ft->shell_state.pid[0] = '\0';
  if (fig_info->term_session_id != NULL) {
    strcpy(ft->shell_state.session_id, fig_info->term_session_id);
  } else {
    ft->shell_state.session_id[0] = '\0';
  }
  ft->shell_state.hostname = strdup("");
  ft->shell_state.tty[0] = '\0';
  ft->shell_state.shell[0] = '\0';

  // TODO(sean): should get color support on prompt passing relevent env variables through.
  ft->shell_state.color_support = get_color_support();
  ft->shell_state.fish_suggestion_color_text = NULL;
  ft->shell_state.fish_suggestion_color = NULL;
  figterm_update_fish_suggestion_color(ft, getenv("fish_color_autosuggestion"));

  ft->shell_state.in_ssh = false;
  ft->shell_state.in_docker = false;
  ft->shell_state.preexec = true;
  ft->shell_state.in_prompt = false;

  ft->disable_figterm = false;

  ft->has_seen_prompt = false;

  // Used for resize.
  ft->ptyp_fd = ptyp_fd;
  ft->shell_pid = shell_pid;

  ft->last_command = NULL;

  ft->vt = vt;
  FigTermScreen* screen = figterm_screen_new(vt);
  ft->screen = screen;

  figterm_screen_set_callbacks(screen, &screen_callbacks, ft);
  figterm_screen_set_unrecognised_fallbacks(screen, &state_fallbacks, ft);
  figterm_screen_reset(screen, true);

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
    free(ft->last_command);
  }
  free(ft);
}

bool figterm_can_send_buffer(FigTerm* ft) {
  bool in_ssh_or_docker = ft->shell_state.in_ssh || ft->shell_state.in_docker;
  bool shell_enabled = strcmp(ft->shell_state.shell, "bash") == 0 ||
    strcmp(ft->shell_state.shell, "fish") == 0 ||
    (in_ssh_or_docker && strcmp(ft->shell_state.shell, "zsh") == 0);
  bool insertion_locked = access(ft->insertion_lock_path, F_OK) == 0;
  return shell_enabled && !insertion_locked && !ft->shell_state.preexec;
}

char* figterm_get_buffer(FigTerm* ft, int* index) {
  int i = ft->cmd_cursor->row;
  int j = ft->cmd_cursor->col;

  if (i < 0)
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
  buf[row_len] = '\n';
  row_len += 1;

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
  if (!ft->disable_figterm && !ft->shell_state.preexec) {
    log_info("Writing %d chars to child shell: %.*s", n, n, buf);
  }
  vterm_input_write(ft->vt, buf, n);
}

bool figterm_is_disabled(FigTerm* ft) {
  return ft == NULL || ft->disable_figterm;
}

bool figterm_has_seen_prompt(FigTerm* ft) {
  return ft != NULL && ft->has_seen_prompt;
}

void figterm_update_fish_suggestion_color(FigTerm* ft, const char* new_color) {
  if (new_color == NULL) {
    return;
  }
  char* current_color = ft->shell_state.fish_suggestion_color_text;
  if (current_color == NULL || strcmp(current_color, new_color) != 0) {
    free(ft->shell_state.fish_suggestion_color);
    free(ft->shell_state.fish_suggestion_color_text);

    ft->shell_state.fish_suggestion_color_text = strdup(new_color);

    ft->shell_state.fish_suggestion_color = parse_vterm_color_from_string(
      new_color,
      ft->shell_state.color_support
    );
  }
}

void figterm_preexec_hook(FigTerm* ft) {
  int index;
  char* buffer = figterm_get_buffer(ft, &index);

  if (buffer == NULL)
    return;

  // Strip trailing \n before adding to history.
  int bufLen = strlen(buffer);
  if (index == bufLen && buffer[bufLen - 1] == '\n') {
    buffer[bufLen - 1] = '\0';
    index -= 1;
  }

  history_entry_free(ft->last_command);
  ft->last_command = history_entry_new(
    buffer,
    ft->shell_state.shell,
    ft->shell_state.pid,
    ft->shell_state.session_id,
    get_cwd(ft->shell_pid),
    time(NULL),
    ft->shell_state.in_ssh,
    ft->shell_state.in_docker,
    ft->shell_state.hostname,
    0
  );
}
