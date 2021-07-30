#include "fig.h"
#include <stdlib.h>
#include <stdio.h>
#include <unistd.h>
#include <string.h>
#include <limits.h>


#define MAX_BUFFER_SIZE 1024
#define MAX_HANDLER_ID_LEN 5 + 1

int main(int argc, char *argv[]) {
  bool debug = (getenv("FIG_DEBUG") != NULL);

  if (argc != 2 || isatty(fileno(stdin))) {
    if (debug) printf("fig_callback must include a handlerId and must be the terminating command of a shell pipeline\n");
    exit(1);
  }

  // check if data on stdin
  if ((fseek(stdin, 0, SEEK_END), ftell(stdin)) > 0) {
    if (debug) printf("No data on stdin!\n");
    exit(1);
  }

  // get handler id from `fig-callback 6sD8n`
  char handlerId[MAX_HANDLER_ID_LEN];
  memset(handlerId, '\0', sizeof(handlerId));
  strncpy(handlerId, argv[1], MAX_HANDLER_ID_LEN);
  if (debug) printf("handlerId: %s\n", handlerId);

  // todo(mschrage): determine exit code of previous command, if possible 

  // create tmp file
  char template[] = "/tmp/fig-callback-XXXXXX";
  char filename[PATH_MAX];

  strcpy(filename, template);

  int fd;
  fd = mkstemp(filename);
  FILE* fp = fdopen(fd, "w");

  if (debug) printf("Created tmp file: %s\n", filename);

  // read all of stdin
  char buffer[MAX_BUFFER_SIZE+1] = {0};
  size_t bytes;

  while ((bytes = fread(buffer, 1, MAX_BUFFER_SIZE, stdin)) > 0) {
      if (debug) printf("Read %zu bytes\n", bytes);

      // write to file
      fwrite(buffer, sizeof(char), bytes, fp);
      if (debug) printf("%s\n", buffer);

      if (feof(stdin)) { 
        fflush(fp);
        if (debug) printf("EOF!\n");
        break ;
      }
  }

  if (debug) printf("Done reading from stdin!\n");

  char *tmpbuf = malloc(strlen(filename) + sizeof(handlerId) + sizeof(char) * 50);
  sprintf(
    tmpbuf,
    "fig pty:callback %s %s",
    filename,
    handlerId
  );

  if (debug) printf("Sending '%s' over unix socket!\n", tmpbuf);

  // send to macOS app over unix socket
  fig_socket_send(strcat(tmpbuf, "\n"));
  free(tmpbuf);

}
