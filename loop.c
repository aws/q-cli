#include "fig.h"
#include <errno.h>
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/wait.h>
#include <unistd.h>
#include <vterm.h>
#include <vterm_keycodes.h>

#define strneq(a,b,n) (strncmp(a,b,n)==0)
#define BUFFSIZE (1024 * 100)

static volatile sig_atomic_t sigcaught;

// Called when child sends us SIGTERM
static void sig_term(int signo) { sigcaught = 1; }

static int movecursor_cb(VTermPos pos, VTermPos oldpos, int visible,
                         void *user) {
  FigTerm *ft = user;
  log_debug("Move cursor: (%d, %d)->(%d, %d)", oldpos.row, oldpos.col, pos.row,
            pos.col);
  ft->cursor->row = pos.row;
  ft->cursor->col = pos.col;
  term_state_update_cursor(ft->state, pos);
  return 0;
}

static int sb_pushline_cb(int cols, const VTermScreenCell *cells, void *user) {
  FigTerm *ft = user;
  log_debug("Scroll down");
  ft->prompt_state->scroll += 1;
  return 0;
}

static int sb_popline_cb(int cols, VTermScreenCell *cells, void *user) {
  FigTerm *ft = user;
  log_debug("Scroll up");
  ft->prompt_state->scroll -= 1;
  return 0;
}

static int damage_cb(VTermRect rect, void *user) {
  FigTerm *ft = user;
  char *prompt_str = ft->in_prompt ? " (+prompt)" : "";
  log_debug("Damage screen%s: (%d-%d, %d-%d)", prompt_str, rect.start_row,
            rect.end_row, rect.start_col, rect.end_col);
  if (term_state_update(ft->state, ft->vt, rect, false) == -1) 
    ft->disable_figterm = true;
  if (ft->in_prompt) {
    if (term_state_update(ft->prompt_state, ft->vt, rect, false) == -1)
      ft->disable_figterm = true;
  }

  print_term_state(ft->state, false);
  print_term_state(ft->prompt_state, true);
  return 0;
}

int settermprop_cb(VTermProp prop, VTermValue *val, void *user) {
  FigTerm *ft = user;
  log_debug("Termprop: %d, %d", prop);
  if (prop == VTERM_PROP_ALTSCREEN) {
    log_debug("Altscreen: %s", val->boolean ? "on" : "off");
    ft->altscreen = val->boolean;
  }
  return 0;
}

int osc_cb(int command, VTermStringFragment frag, void *user) {
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

      if (strcmp(ft->osc, "NewCmd") == 0) {
        VTermRect rect = {};
        if (term_state_update(ft->prompt_state, ft->vt, rect, true) == -1)
          ft->disable_figterm = true;
        term_state_update_cursor(ft->prompt_state, *ft->cursor);
        log_info("Prompt at position: (%d, %d)", ft->cursor->row,
                ft->cursor->col);
        ft->preexec = false;
      } else if (strcmp(ft->osc, "StartPrompt") == 0) {
        VTermScreen *vts = vterm_obtain_screen(ft->vt);
        vterm_screen_set_damage_merge(vts, VTERM_DAMAGE_CELL);
        ft->in_prompt = true;
      } else if (strcmp(ft->osc, "EndPrompt") == 0) {
        VTermScreen *vts = vterm_obtain_screen(ft->vt);
        vterm_screen_flush_damage(vts);
        vterm_screen_set_damage_merge(vts, VTERM_DAMAGE_ROW);
        ft->in_prompt = false;
      } else if (strcmp(ft->osc, "PreExec") == 0) {
        ft->preexec = true;
      } else if (strneq(ft->osc, "Dir=", 4)) {
        log_info("In dir %s", ft->osc + 4);
      } else if (strneq(ft->osc, "Shell=", 6)) {
        // Only enable in bash for now.
        ft->shell_enabled = strcmp(ft->osc + 6, "bash") == 0;
      } else if (strneq(ft->osc, "TTY=", 4)) {
        strcpy(ft->tty, ft->osc + 4);
      } else if (strneq(ft->osc, "PID=", 4)) {
        strcpy(ft->pid, ft->osc + 4);
      }
      free(ft->osc);
      ft->osc = NULL;
    }

  }
  return 0;
}

static VTermStateFallbacks parser_callbacks = {
    .osc = osc_cb,
};

static VTermScreenCallbacks screen_callbacks = {
    .damage = damage_cb,
    .settermprop = settermprop_cb,
    .movecursor = movecursor_cb,
    .sb_pushline = sb_pushline_cb,
    .sb_popline = sb_popline_cb,
};


void publish_guess(int index, char *buffer, FigTerm* ft) {
  FigInfo *fig_info = get_fig_info();
  size_t buflen = strlen(buffer) +
    strlen(fig_info->term_session_id) +
    strlen(fig_info->fig_integration_version) +
    strlen(ft->tty) +
    strlen(ft->pid);

  char *tmpbuf = malloc(buflen + sizeof(char) * 50);
  sprintf(
    tmpbuf,
    "fig bg:bash-keybuffer %s %s %s %s 0 %d \"%s\"",
    fig_info->term_session_id,
    fig_info->fig_integration_version,
    ft->tty,
    ft->pid,
    index,
    buffer
  );

  fig_socket_send(tmpbuf);
  free(tmpbuf);
}

void loop(int ptyp, pid_t child, pid_t ptyc_pid) {
  int nread;
  char buf[BUFFSIZE + 1];

  if (set_sigaction(SIGWINCH, figterm_handle_winch) == SIG_ERR)
    err_sys("signal_intr error for SIGWINCH");

  if (set_sigaction(SIGTERM, sig_term) == SIG_ERR)
    err_sys("signal_intr error for SIGTERM");

  // Initialize screen buffer copy "FigTerm".
  FigTerm *ft = figterm_new(true, &screen_callbacks, &parser_callbacks, ptyc_pid, ptyp);

  char* insertion_lock = fig_path("insertion-lock");
  for (;;) {
    // Read from pty parent.
    nread = read(ptyp, buf, BUFFSIZE - 1);
    log_debug("read %d chars on ptyp (%d)", nread, errno);
    if (nread < 0 && errno == EINTR)
      continue;
    else if (nread <= 0)
      break;

    if (write(STDOUT_FILENO, buf, nread) != nread)
      err_sys("write error to stdout");

    if (ft == NULL || ft->disable_figterm)
      continue;

    // Make buf a proper str to use str operations.
    buf[nread] = '\0';

    log_debug("Writing %d chars %.*s", nread, nread, buf);
    vterm_input_write(ft->vt, buf, nread);
    VTermScreen *vts = vterm_obtain_screen(ft->vt);
    vterm_screen_flush_damage(vts);

    if (!ft->preexec && ft->shell_enabled && access(insertion_lock, F_OK) != 0) {
      int index;
      char *guess = extract_buffer(ft->state, ft->prompt_state, &index);

      if (guess != NULL) {
        log_info("guess: %s|\nindex: %d", guess, index);
        if (index >= 0)
          publish_guess(index, guess, ft);
      } else {
        ft->preexec = true;
        log_info("Null guess, waiting for new prompt...");
        ft->prompt_state->cursor->row = -1;
        ft->prompt_state->cursor->col = -1;
      }
      free(guess);
    }
  }

  // clean up
  figterm_free(ft);
  free(insertion_lock);

  if (sigcaught == 1) {
    // child exited, check status code.
    int status;
    if ((waitpid(child, &status, 0) != child)
        || (WIFEXITED(status) && WEXITSTATUS(status) != 0))
      err_sys("child did not exit cleanly");
  } else {
    // Kill child if we read EOF on pty parent
    kill(child, SIGTERM);
  }
}
