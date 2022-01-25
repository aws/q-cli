#include "fig.h"


char* get_cwd(pid_t pid) {
#if defined(__APPLE__)
  // https://stackoverflow.com/a/33852283
  char* cwd = malloc(sizeof(char) * PROC_PIDPATHINFO_MAXSIZE);
  struct proc_vnodepathinfo vpi;
  if (proc_pidinfo(pid, PROC_PIDVNODEPATHINFO, 0, &vpi, sizeof(vpi)) <= 0)
    return NULL;
  strcpy(cwd, vpi.pvi_cdir.vip_path);
  return cwd;
#else
  int len;
  unsigned int bufsize = 1024;
  char* buf = calloc(bufsize, sizeof(char));
  if (buf == NULL)
    return NULL;

  char procfile[20];

  sprintf(procfile, "/proc/%d/cwd", pid);

  while (true) {
    len = readlink(procfile, buf, bufsize - 1);
    if (len == -1) {
      free(buf);
      return NULL;
    } else if ((size_t) len != bufsize - 1) {
      buf[len] = '\0';
      break;
    }
    bufsize *= 2;
    buf = (char *) realloc(buf, bufsize);
  }
  return buf;
#endif
}
