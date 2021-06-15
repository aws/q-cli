#include "fig.h"
#include <errno.h>
#include <math.h>
#include <stdio.h>
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
  term_state_update(ft->state, ft->vt, rect, false);
  if (ft->in_prompt)
    term_state_update(ft->prompt_state, ft->vt, rect, false);

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
  log_debug("OSC CB: %d, %.*s", command, frag.len, frag.str);
  if (command == 697) {
    FigTerm *ft = user;
    if (strneq(frag.str, "NewCmd", frag.len)) {
      VTermRect rect = {};
      term_state_update(ft->prompt_state, ft->vt, rect, true);
      term_state_update_cursor(ft->prompt_state, *ft->cursor);
      log_info("Prompt at position: (%d, %d)", ft->cursor->row,
               ft->cursor->col);
      ft->preexec = false;
    } else if (strneq(frag.str, "StartPrompt", frag.len)) {
      VTermScreen *vts = vterm_obtain_screen(ft->vt);
      vterm_screen_set_damage_merge(vts, VTERM_DAMAGE_CELL);
      ft->in_prompt = true;
    } else if (strneq(frag.str, "EndPrompt", frag.len)) {
      VTermScreen *vts = vterm_obtain_screen(ft->vt);
      vterm_screen_flush_damage(vts);
      vterm_screen_set_damage_merge(vts, VTERM_DAMAGE_ROW);
      ft->in_prompt = false;
    } else if (strneq(frag.str, "PreExec", frag.len)) {
      ft->preexec = true;
    } else if (strneq(frag.str, "Dir=", 4)) {
      log_info("In dir %.*s", frag.len - 4, frag.str + 4);
    } else if (strneq(frag.str, "Shell=", 6)) {
      // Only enable in bash for now.
      ft->shell_enabled = strneq(frag.str + 6, "bash", frag.len - 6);
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

void child_loop(int ptyp) {
  int nread;
  char buf[BUFFSIZE + 1];

  for (;;) {
    // Copy stdin to pty parent.
    if ((nread = read(STDIN_FILENO, buf, BUFFSIZE)) < 0)
      err_sys("read error from stdin");
    else if (nread == 0)
      break;
    if (write(ptyp, buf, nread) != nread)
      err_sys("write error to parent pty");
  }

  // notify parent process on exit.
  kill(getppid(), SIGTERM);
  exit(0);
}

void publish_guess(int index, char *buffer) {
  FigInfo *fig_info = get_fig_info();
  size_t buflen = strlen(buffer) + strlen(fig_info->term_session_id) +
                  strlen(fig_info->fig_integration_version);
  char *tmpbuf = malloc(buflen + sizeof(char) * 50);
  sprintf(tmpbuf, "fig bg:bash-keybuffer %s %s 0 %d \"%s\"",
          fig_info->term_session_id, fig_info->fig_integration_version, index,
          buffer);
  fig_socket_send(tmpbuf);
  free(tmpbuf);
}

void loop(int ptyp, int ptyc_pid) {
  pid_t child;
  int nread;
  char buf[BUFFSIZE + 1];

  if ((child = fork()) < 0) {
    err_sys("fork error");
  } else if (child == 0) {
    child_loop(ptyp);
  }

  // Initialize screen buffer copy "FigTerm".
  FigTerm *ft = figterm_new(true, &screen_callbacks, &parser_callbacks, ptyc_pid, ptyp);
  char* insertion_lock = fig_path("insertion-lock");
  log_info("Shell: %d", ptyc_pid);
  log_info("Child: %d", child);
  log_info("Parent: %d", getpid());

  if (set_sigaction(SIGWINCH, figterm_handle_winch) == SIG_ERR)
    err_sys("signal_intr error for SIGWINCH");

  if (set_sigaction(SIGTERM, sig_term) == SIG_ERR)
    err_sys("signal_intr error for SIGTERM");

  for (;;) {
    // Read from pty parent.
    nread = read(ptyp, buf, BUFFSIZE - 1);
    if (nread < 0 && errno == EINTR) {
      continue;
    } else if (nread < 0) {
      err_sys("read error from ptyp");
    }

    // Make buf a proper str to use str operations.
    buf[nread] = '\0';

    log_debug("Writing %.*s", nread, buf);
    vterm_input_write(ft->vt, buf, nread);
    VTermScreen *vts = vterm_obtain_screen(ft->vt);
    vterm_screen_flush_damage(vts);

    if (write(STDOUT_FILENO, buf, nread) != nread)
      err_sys("write error to stdout");

    bool fig_is_inserting = access(insertion_lock, F_OK) == 0;

    if (!ft->preexec && ft->shell_enabled && !fig_is_inserting) {
      int index;
      char *guess = extract_buffer(ft->state, ft->prompt_state, &index);

      if (guess != NULL) {
        log_info("guess: %s|\nindex: %d", guess, index);
        if (index >= 0) {
          publish_guess(index, guess);
        }
      } else {
        ft->preexec = true;
        log_info("Null guess, waiting for new prompt...");
        ft->prompt_state->cursor->row = -1;
        ft->prompt_state->cursor->col = -1;
      }
      free(guess);
    }
  }

  // Kill child if we read EOF on pty parent
  if (sigcaught == 0)
    kill(child, SIGTERM);

  // clean up
  figterm_free(ft);
  free(insertion_lock);
}
