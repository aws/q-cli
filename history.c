#include "fig.h"
#include <stdlib.h>

#define strdup_safe(name) (name == NULL ? NULL : strdup(name))

struct HistoryEntry {
  char* command;
  char shell[10];
  char session_id[SESSION_ID_MAX_LEN + 1];
  char* cwd;
  unsigned long time;

  bool in_ssh;
  bool in_docker;
  char* hostname;

  unsigned int exit_code;
};

// https://stackoverflow.com/a/33988826
char *escaped_str(const char *src) {
  int i, j;

  for (i = j = 0; src[i] != '\0'; i++) {
    if (src[i] == '\n' || src[i] == '\t' ||
        src[i] == '\\' || src[i] == '\"') {
      j++;
    }
  }
  char* pw = malloc(sizeof(char) * (i + j + 1));

  for (i = j = 0; src[i] != '\0'; i++) {
    switch (src[i]) {
      case '\n': pw[i+j] = '\\'; pw[i+j+1] = 'n'; j++; break;
      case '\t': pw[i+j] = '\\'; pw[i+j+1] = 't'; j++; break;
      case '\\': pw[i+j] = '\\'; pw[i+j+1] = '\\'; j++; break;
      case '\"': pw[i+j] = '\\'; pw[i+j+1] = '\"'; j++; break;
      default:   pw[i+j] = src[i]; break;
    }
  }
  pw[i+j] = '\0';
  return pw;
}

HistoryEntry* history_entry_new(
    char* command,
    char* shell,
    char* session_id,
    char* cwd,
    unsigned long time,
    bool in_ssh,
    bool in_docker,
    char* hostname,
    unsigned int exit_code) {
  HistoryEntry* entry = malloc(sizeof(HistoryEntry));
  entry->command = strdup_safe(command);
  entry->cwd = strdup_safe(cwd);
  entry->hostname = strdup_safe(hostname);

  strcpy(entry->shell, shell);
  strcpy(entry->session_id, session_id);

  entry->time = time;
  entry->in_ssh = in_ssh;
  entry->in_docker = in_docker;
  entry->exit_code = exit_code;

  return entry;
}

void history_entry_free(HistoryEntry* entry) {
  if (entry != NULL) {
    free(entry->command);
    free(entry->cwd);
    free(entry->hostname);
  }
  free(entry);
}

void history_entry_set_exit_code(HistoryEntry* entry, unsigned int exit_code) {
  entry->exit_code = exit_code;
}

int history_fd = -1;

void history_file_close() {
  if (history_fd >= 0) {
    close(history_fd);
  }
}

void history_file_open() {
  char* fname = fig_path("history");
  history_fd = open(fname, O_WRONLY | O_APPEND | O_CREAT, 0644);
  free(fname);
}

void write_history_entry(HistoryEntry* entry) {
  // Don't write if we don't have a command or the command was exited with ^C
  if (entry == NULL || entry->command == NULL || entry->exit_code == 130)
    return;

  if (history_fd < 0) {
    history_file_open();
  }

  char* command_escaped = escaped_str(entry->command);
  log_info("Adding to history: %s", command_escaped);

  char time_str[20];
  sprintf(time_str, "%lu", entry->time);

  char* tmp = malloc(sizeof(char) * (
      strlen("\n- command: %s\n exit_code: %s\n  shell: %s\n  session_id: %s\n  cwd: %s\n  time: %s\n  docker: %s\n  ssh: %s\n hostname: %s") +
      strlen(command_escaped) +
      // Max value of an exit code is 255 -- so max string length is 3 + 1 for good measure.
      4 +
      strlen(entry->shell) +
      strlen(entry->session_id) +
      strlen(entry->cwd) +
      strlen(time_str) +
      // Max length of a boolean string is 5 for docker + ssh.
      5 +
      5 +
      strlen(entry->hostname)
    ));

  sprintf(
      tmp,
      "\n- command: %s\n  exit_code: %d\n  shell: %s\n  session_id: %s\n  cwd: %s\n  time: %s",
      command_escaped,
      entry->exit_code,
      entry->shell,
      entry->session_id,
      entry->cwd,
      time_str
  );

  if (entry->in_ssh || entry->in_docker) {
    if (entry->in_docker) {
      strcat(tmp, "\n  docker: true");
    }
    if (entry->in_ssh) {
      strcat(tmp, "\n  ssh: true");
    }
    strcat(tmp, "\n  hostname: ");
    strcat(tmp, entry->hostname);
  }

  flock(history_fd, LOCK_EX);
  dprintf(history_fd, "%s", tmp);
  flock(history_fd, LOCK_UN);

  free(tmp);
  free(command_escaped);
}
