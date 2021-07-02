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

typedef struct FigTermScreen FigTermScreen;

typedef struct {
  void (*scroll)(int scroll_delta, void *user);
} FigTermScreenCallbacks;

typedef struct {
  VTerm* vt;
  FigTermScreen* screen;
  VTermPos *cmd_cursor;
  char* insertion_lock_path;

  char tty[30];
  char pid[8];

  char* osc;
  bool parsing_osc;

  bool shell_enabled;
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

typedef enum {
  FIGTERM_ATTR_IN_PROMPT = 0,
} FigTermAttr;

// screen.c
FigTermScreen* figterm_screen_new(VTerm*);
void figterm_screen_free(FigTermScreen*);
void figterm_screen_set_unrecognised_fallbacks(FigTermScreen*, const VTermStateFallbacks*, void*);
void figterm_screen_reset(FigTermScreen*, int);
void figterm_screen_set_callbacks(FigTermScreen*, const FigTermScreenCallbacks*, void*);
void figterm_screen_get_cursorpos(FigTermScreen*, VTermPos*);
void figterm_screen_set_attr(FigTermScreen*, FigTermAttr, void*);
size_t figterm_screen_get_text(FigTermScreen*, char*, size_t, const VTermRect, char, int*);

// figterm.c
FigTerm* figterm_new(int, int);
void figterm_free(FigTerm*);
char* figterm_get_buffer(FigTerm*, int*);
void figterm_resize(FigTerm*);
void figterm_log(FigTerm*, char);

// util.c
int get_winsize();
FigInfo* init_fig_info();
FigInfo* get_fig_info();
void free_fig_info();
void set_pty_name(char*);
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
