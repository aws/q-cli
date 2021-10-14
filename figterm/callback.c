#include "fig.h"
#include <stdlib.h>
#include <stdio.h>
#include <unistd.h>
#include <string.h>
#include <limits.h>

#define VERSION_NUMBER 2
#define MAX_BUFFER_SIZE 1024
#define MAX_HANDLER_ID_LEN 5
#define MAX_EXIT_CODE_LEN 3
int main(int argc, char *argv[]) {
  bool debug = (getenv("FIG_DEBUG") != NULL);

  if (argc < 2) {
    if (debug) printf("fig_callback must include a handlerId.\n");
    exit(1);
  }

  if (strcmp(argv[1], "-v") == 0 || strcmp(argv[1], "--version") == 0) {
    printf("%i\n", VERSION_NUMBER);
    exit(0);
  }

  // check if data on stdin
  if ((fseek(stdin, 0, SEEK_END), ftell(stdin)) > 0) {
    if (debug) printf("No data on stdin!\n");
    exit(1);
  }

  // get handler id from `fig-callback 6sD8n`
  char handlerId[MAX_HANDLER_ID_LEN + 1];
  memset(handlerId, '\0', sizeof(handlerId));
  strncpy(handlerId, argv[1], MAX_HANDLER_ID_LEN);
  if (debug) printf("handlerId: %s\n", handlerId);

  char filename[PATH_MAX];
  char exitcode[MAX_EXIT_CODE_LEN + 1] = "-1";
  if (argc == 4) {
    if (debug) printf("fig_callback specified filepath (%s) and exitCode (%s) to output!\n", argv[2], argv[3]);
    strncpy(filename, argv[2], PATH_MAX);
    
    memset(exitcode, '\0', sizeof(exitcode));
    strncpy(exitcode, argv[3], MAX_EXIT_CODE_LEN);
    goto send;
  }

  // todo(mschrage): determine exit code of previous command, if possible 

  // create tmp file
  char template[] = "/tmp/fig-callback-XXXXXX";

  strcpy(filename, template);

  int fd;
  fd = mkstemp(filename);
  FILE* fp = fdopen(fd, "w");

  if (debug) printf("Created tmp file: %s\n", filename);

  // read all of stdin
  char buffer[MAX_BUFFER_SIZE+1] = {0};
  size_t bytes;

  while ((bytes = fread(buffer, 1, MAX_BUFFER_SIZE, stdin)) >= 0) {
      if (debug) printf("Read %zu bytes\n", bytes);

      // write to file
      fwrite(buffer, sizeof(char), bytes, fp);
      if (debug) printf("%s\n", buffer);

      if (feof(stdin) || ferror(stdin)) { 
        fflush(fp);
        if (debug) printf("EOF!\n");
        break ;
      }
  }
  
  send:
  if (debug) printf("Done reading from stdin!\n");

  char *tmpbuf = malloc(strlen(filename) + sizeof(handlerId) + sizeof(exitcode) + sizeof(char) * 50);
  sprintf(
    tmpbuf,
    "fig pty:callback %s %s %s",
    handlerId,
    filename,
    exitcode
  );

  if (debug) printf("Sending '%s' over unix socket!\n", tmpbuf);

  // send to macOS app over unix socket
  fig_socket_send(strcat(tmpbuf, "\n"));
  free(tmpbuf);

}