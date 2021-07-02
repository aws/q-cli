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

void publish_buffer(int index, char *buffer, FigTerm* ft) {
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

  int ret = fig_socket_send(tmpbuf);
  log_info("done sending %d", ret);
  free(tmpbuf);
}

static FigTerm* _ft; 

// TODO(sean) This should probably not be done inside a signal handler,
// consider non-blocking i/o and setting a flag instead.
void handle_winch(int sig) { figterm_resize(_ft); }

void loop(int ptyp, pid_t ptyc_pid) {
  int nread;
  char buf[BUFFSIZE + 1];
  FigTerm* ft;
  int index;

  if (set_sigaction(SIGWINCH, handle_winch) == SIG_ERR)
    err_sys("signal_intr error for SIGWINCH");

  ft = _ft = figterm_new(ptyc_pid, ptyp);

  fd_set rfd;

  for (;;) {
    FD_ZERO(&rfd);
    FD_SET(STDIN_FILENO, &rfd);
    FD_SET(ptyp, &rfd);

    int n = select(ptyp + 1, &rfd, 0, 0, NULL);
    if (n < 0 && errno != EINTR) {
      err_sys("select error");
    }
    if (n > 0 && FD_ISSET(STDIN_FILENO, &rfd)) {
      // Copy stdin to pty parent.
      if ((nread = read(STDIN_FILENO, buf, BUFFSIZE)) < 0)
        err_sys("read error from stdin");
      log_debug("Read %d chars on stdin", nread);
      if (write(ptyp, buf, nread) != nread)
        err_sys("write error to parent pty");
      if (nread == 0)
        break;
    }
    if (n > 0 && FD_ISSET(ptyp, &rfd)) {
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

      log_info("Writing %d chars %.*s", nread, nread, buf);
      vterm_input_write(ft->vt, buf, nread);
      char* buffer = figterm_get_buffer(ft, &index);

      if (buffer != NULL) {
        log_info("guess: %s|\nindex: %d", buffer, index);
        figterm_log(ft, '.');
        if (index >= 0)
          publish_buffer(index, buffer, ft);
      }
    }
  }

  // clean up
  figterm_free(ft);
}
