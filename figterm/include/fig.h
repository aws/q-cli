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
#include <errno.h>
#include <stdbool.h>
#include <time.h>
#include "vterm.h"

#define	MAXLINE	4096

// screen.c
typedef enum {
  FIGTERM_ATTR_IN_PROMPT = 0,
  FIGTERM_ATTR_IN_SUGGESTION = 1,
} FigTermAttr;

typedef struct {
  void (*scroll)(int scroll_delta, void *user);
  int (*movecursor)(VTermPos pos, VTermPos oldpos, int visible, void *user);
  int (*setpenattr)(VTermAttr attr, VTermValue *val, void *user);
} FigTermScreenCallbacks;

typedef struct FigTermScreen FigTermScreen;

// Initialization functions.
FigTermScreen* figterm_screen_new(VTerm*);
void figterm_screen_free(FigTermScreen*);
void figterm_screen_set_unrecognised_fallbacks(FigTermScreen*, const VTermStateFallbacks*, void*);
void figterm_screen_reset(FigTermScreen*, int);
void figterm_screen_set_callbacks(FigTermScreen*, const FigTermScreenCallbacks*, void*);

// Methods.
void figterm_screen_get_cursorpos(FigTermScreen*, VTermPos*);
void figterm_screen_set_attr(FigTermScreen*, FigTermAttr, void*);
size_t figterm_screen_get_text(FigTermScreen*, char*, size_t, const VTermRect, int, char, bool, int*);

// color.c
enum { color_support_term256 = 1 << 0, color_support_term24bit = 1 << 1 };
typedef unsigned int color_support_t;
color_support_t get_color_support();

VTermColor* parse_vterm_color_from_string(const char*, color_support_t);

// history.c
typedef struct HistoryEntry HistoryEntry;
HistoryEntry* history_entry_new(
  char* command,
  char* shell,
  char* pid,
  char* session_id,
  char* cwd,
  unsigned long time,
  bool in_ssh,
  bool in_docker,
  char* hostname,
  unsigned int exit_code
);
void history_entry_free(HistoryEntry*);
void history_entry_set_exit_code(HistoryEntry*, unsigned int);
void history_file_close();
void write_history_entry(HistoryEntry*);

// figterm.c
#define SESSION_ID_MAX_LEN 50

// Holds information about shell processes passed from shell config via osc.
typedef struct {
  char tty[30];
  char pid[8];
  char session_id[SESSION_ID_MAX_LEN + 1];
  char* hostname;

  char shell[10];

  char* fish_suggestion_color_text;
  VTermColor* fish_suggestion_color;

  color_support_t color_support;

  bool in_ssh;
  bool in_docker;

  bool preexec;
  bool in_prompt;

} FigShellState;

typedef struct FigTerm FigTerm;

FigTerm* figterm_new(int, int);
void figterm_free(FigTerm*);

char* figterm_get_buffer(FigTerm*, int*);
void figterm_resize(FigTerm*);
void figterm_log(FigTerm*, char);
void figterm_get_shell_state(FigTerm*, FigShellState*);
void figterm_write(FigTerm*, char*, int);
bool figterm_is_disabled(FigTerm*);
bool figterm_has_seen_prompt(FigTerm*);
bool figterm_can_send_buffer(FigTerm*);
void figterm_update_fish_suggestion_color(FigTerm*, const char*);
pid_t figterm_get_shell_pid(FigTerm*);
char* figterm_get_shell_context(FigTerm*);

void figterm_preexec_hook(FigTerm*);

// util.c
typedef struct {
  char *term_session_id;
  char *fig_integration_version;
  char *pty_name;
} FigInfo;

int get_winsize();
FigInfo* init_fig_info();
FigInfo* get_fig_info();
void free_fig_info();
void set_pty_name(char*);
int set_blocking(int fd, bool blocking);
int fig_socket_send(char*);
int fig_socket_listen();
void fig_socket_cleanup();
char* fig_path(char*);
char* log_path(char*);
char* printf_alloc(const char*, ...);
void publish_json(const char*, ...);
char *escaped_str(const char *);
char *get_term_bundle();

// libfig

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
int ptyp_open(char*);
int ptyc_open(int, char*, const struct termios*, const struct winsize*);

// lib/proc.c
char* get_cwd(pid_t);

// lib/log.c
enum { LOG_FATAL, LOG_ERROR, LOG_WARN, LOG_INFO, LOG_DEBUG };

void log_msg(int level, const char *file, int line, const char *fmt, ...);
void err_sys_msg(const char *file, int line, const char *fmt, ...) __attribute__((noreturn));
void set_logging_level(int);
void set_logging_level_from_string(char*);
int get_logging_level();
void init_log_file(char*);
void close_log_file();

#define log_debug(...) log_msg(LOG_DEBUG, __FILE__, __LINE__, __VA_ARGS__)
#define log_info(...)  log_msg(LOG_INFO,  __FILE__, __LINE__, __VA_ARGS__)
#define log_warn(...)  log_msg(LOG_WARN,  __FILE__, __LINE__, __VA_ARGS__)
#define log_error(...) log_msg(LOG_ERROR, __FILE__, __LINE__, __VA_ARGS__)
#define log_fatal(...) log_msg(LOG_FATAL, __FILE__, __LINE__, __VA_ARGS__)
#define err_sys(...) err_sys_msg(__FILE__, __LINE__, __VA_ARGS__)

#define VA_HEAD(head, ...) head
#define VA_TAIL(head, ...) , ## __VA_ARGS__

#define log_err(...) \
  log_error(VA_HEAD(__VA_ARGS__) "" " (%d): %s" VA_TAIL(__VA_ARGS__), errno, strerror(errno))

// Macros for guarded system calls. Logs and returns from calling function on
// error in syscall. msg must be a string literal, either a format string or string.
// Uses ## __VA_ARGS__ to allow empty variadic macro args (https://stackoverflow.com/a/5897216)
#define CHECK(condition, ret, ...) \
  if (!(condition)) { \
    log_err(__VA_ARGS__); \
    return ret; \
  }

#define CHECK_SYS(call, ...) \
  do { \
    int CHECK_VALUE_call = (call); \
    CHECK(CHECK_VALUE_call > -1, CHECK_VALUE_call, __VA_ARGS__); \
  } while(0)

#define CHECK_NONNULL(call, msg, ...) \
  do { \
    void* CHECK_VALUE_call = (call); \
    CHECK(CHECK_VALUE_call != NULL, NULL, __VA_ARGS__); \
  } while(0)


typedef	void SigHandler(int);
SigHandler* set_sigaction(int, SigHandler*);

// lib/base64.c
unsigned char * base64_encode(const unsigned char *src, size_t len,
			      size_t *out_len);
unsigned char * base64_decode(const unsigned char *src, size_t len,
			      size_t *out_len);
