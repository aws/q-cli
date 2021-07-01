#pragma once

#define _POSIX_C_SOURCE 200809L
#define _DEFAULT_SOURCE

#if defined(SOLARIS)
#define _XOPEN_SOURCE 600
#else
#define _XOPEN_SOURCE 700
#endif

#include <sys/types.h>
#include <sys/stat.h>
#include <sys/termios.h>

#if defined(__APPLE__) || !defined(TIOCGWINSZ)
#include <sys/ioctl.h>
#endif

#if defined(__APPLE__)
#include <libproc.h>
#endif

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <signal.h>
#include <sys/wait.h>
#include <fcntl.h>
#include <stdarg.h>
#include <stdbool.h>
#include <time.h>
#include "vterm.h"

#define	MAXLINE	4096

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
  VTermPos* cursor;
  char* osc;
  char tty[30];
  char pid[8];
  bool parsing_osc;
  bool shell_enabled;
  bool altscreen;
  bool in_prompt;
  bool preexec;
  bool disable_figterm;
  int ptyp;
  int ptyc_pid;
} FigTerm;

typedef struct {
  char *term_session_id;
  char *fig_integration_version;
  char *pty_name;
} FigInfo;

// term_state.c
TermState* term_state_new(VTerm*);
void term_state_free(TermState*);
int term_state_init_rows(TermState*,  int);
void term_state_free_rows(TermState*);
void term_state_update_cursor(TermState*, const VTermPos);
int term_state_update(TermState*, VTerm*, VTermRect, bool);
void print_term_state(TermState*, bool);
char* extract_buffer(TermState*, TermState*, int*);


// figterm.c
FigTerm* figterm_new(bool, VTermScreenCallbacks*, VTermStateFallbacks*, int, int);
void figterm_free(FigTerm*);
void figterm_resize(FigTerm*);
void figterm_handle_winch(int);
int figterm_should_resize();

// util.c
int get_winsize();
FigInfo* init_fig_info();
FigInfo* get_fig_info();
void set_pty_name(char*);
void free_fig_info();
char* get_exe(pid_t);
int unix_socket_connect(char*);
int fig_socket_send(char*);
char* fig_path(char*);
char* log_path(char*);

// lib/exit.c
int get_exit_status();
void exit_with_status(int) __attribute__((noreturn));

#define EXIT(status) exit_with_status(status)

// lib/string.c
void replace(char*, char, char);
char* ltrim(char*);
char* rtrim(char*, int);
char* strrstr(const char*, const char*, const size_t, const size_t);

// lib/tty.c
int tty_raw(int);
void tty_atexit(void);
int tty_reset(int);

// lib/pty.c
int ptyp_open(char*, int);
int ptyc_open(char*);
#ifdef TIOCGWINSZ
pid_t pty_fork(int*, int, char*, const struct termios*, const struct winsize*);
#endif

// lib/log.c
enum { LOG_FATAL, LOG_ERROR, LOG_WARN, LOG_INFO, LOG_DEBUG };

void log_msg(int level, const char *file, int line, const char *fmt, ...);
void err_sys_msg(const char *file, int line, const char *fmt, ...) __attribute__((noreturn));
void set_logging_level(int);
void init_log_file(char*);
void close_log_file();

#define log_debug(...) log_msg(LOG_DEBUG, __FILE__, __LINE__, __VA_ARGS__)
#define log_info(...)  log_msg(LOG_INFO,  __FILE__, __LINE__, __VA_ARGS__)
#define log_warn(...)  log_msg(LOG_WARN,  __FILE__, __LINE__, __VA_ARGS__)
#define log_error(...) log_msg(LOG_ERROR, __FILE__, __LINE__, __VA_ARGS__)
#define log_fatal(...) log_msg(LOG_FATAL, __FILE__, __LINE__, __VA_ARGS__)
#define err_sys(...) err_sys_msg(__FILE__, __LINE__, __VA_ARGS__)

// Signal Handling
typedef	void SigHandler(int);
SigHandler* set_sigaction(int, SigHandler*);

// lib/base64.c
unsigned char * base64_encode(const unsigned char *src, size_t len,
			      size_t *out_len);
unsigned char * base64_decode(const unsigned char *src, size_t len,
			      size_t *out_len);

