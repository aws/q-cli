#include <stdlib.h>
#include <stdbool.h>
#include <stdio.h>
#include <unistd.h>

#if defined(__APPLE__)
#include <libproc.h>
#endif

int main(int argc, char *argv[]) {
  pid_t ppid = getppid();

  // Get executable path of a process by pid.
  ssize_t ret;
  unsigned int bufsize = 1024;
  char* buf = calloc(bufsize, sizeof(char));

  if (buf == NULL)
    goto error;

#if defined(__APPLE__)
  // TODO(sean): make sure pid exists or that access is allowed?
  ret = proc_pidpath(ppid, buf, sizeof(char) * bufsize);

  if (ret == 0)
    exit(1);
#else
  char procfile[50];
  sprintf(procfile, "/proc/%d/exe", ppid);

  while (true) {
    ret = readlink(procfile, buf, bufsize - 1);
    if (ret == -1) {
      goto error;
    } else if ((size_t) ret != bufsize - 1) {
      buf[ret] = '\0';
      break;
    }
    bufsize *= 2;
    buf = (char *) realloc(buf, bufsize);
  }
#endif
  printf("%s", buf);
  free(buf);
  exit(0);
error:
  free(buf);
  exit(1);
}
