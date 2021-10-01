#include <stdlib.h>

static int _exit_status;

void exit_with_status(int status) {
  _exit_status = status;
  exit(status);
}

int get_exit_status() {
  return _exit_status;
}
