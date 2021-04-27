#pragma once

#define _POSIX_C_SOURCE 200809L

#if defined(SOLARIS)
#define _XOPEN_SOURCE 600
#else
#define _XOPEN_SOURCE 700
#endif

#include <sys/types.h>
#include <sys/stat.h>
#include <sys/termios.h>
#if defined(MACOS) || !defined(TIOCGWINSZ)
#include <sys/ioctl.h>
#endif

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <signal.h>
#include <sys/wait.h>
#include <fcntl.h>
#include <vterm.h>
#include <stdarg.h>
#include <stdbool.h>
#include <time.h>


#define	MAXLINE	4096
#define FIG_NEW_CMD "FIG_NEW_CMD"
#define FIG_START_SKIP "FIG_START_SKIP"
#define FIG_END_SKIP "FIG_END_SKIP"

// Common structs
typedef struct {
  VTermPos *cursor;
  char **rows;
  int *row_lens;
  int nrows;
  int scroll;
} TermState;

typedef struct {
  VTerm *vt;
  TermState *state;
  TermState *prompt_state;
  bool update_prompt;
  bool is_resizing;
  bool skip;
  int screen_rows;
  int screen_cols;
  int ptyp;
} FigTerm;

// term_state.c
TermState* term_state_new(VTerm*);
void term_state_free(TermState*);
void term_state_init_rows(TermState*,  int);
void term_state_free_rows(TermState*);
void term_state_update_cursor(TermState*, const VTermPos);

// figterm.c
FigTerm* figterm_new(bool, VTermScreenCallbacks*);
void figterm_free(FigTerm*);
void figterm_resize(FigTerm*);
void figterm_handle_winch(int);
int figterm_should_resize();

// string.c
char* ltrim(char*);
char* rtrim(char*);
char* strrstr(const char*, const char*, const size_t, const size_t);


// lib/tty.c
int tty_raw(int);
void tty_atexit(void);
int tty_reset(int);

// lib/pty.c
int ptyp_open(char*, int);
int ptyc_open(char*);
#ifdef TIOCGWINSZ
pid_t pty_fork(int*, char*, int, const struct termios*, const struct winsize*);
#endif

// lib/log.c
enum { LOG_DEBUG, LOG_INFO, LOG_WARN, LOG_ERROR, LOG_FATAL };

void log_msg(int level, const char *file, int line, const char *fmt, ...);
void err_sys_msg(const char *file, int line, const char *fmt, ...) __attribute__((noreturn));

#define log_debug(...) log_msg(LOG_DEBUG, __FILE__, __LINE__, __VA_ARGS__)
#define log_info(...)  log_msg(LOG_INFO,  __FILE__, __LINE__, __VA_ARGS__)
#define log_warn(...)  log_msg(LOG_WARN,  __FILE__, __LINE__, __VA_ARGS__)
#define log_error(...) log_msg(LOG_ERROR, __FILE__, __LINE__, __VA_ARGS__)
#define log_fatal(...) log_msg(LOG_FATAL, __FILE__, __LINE__, __VA_ARGS__)
#define err_sys(...) err_sys_msg(__FILE__, __LINE__, __VA_ARGS__)

// Signal Handling
typedef	void SigHandler(int);
SigHandler* set_sigaction(int, SigHandler*);
