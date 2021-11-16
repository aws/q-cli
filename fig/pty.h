//
//  pty.h
//  fig
//
//  Created by Matt Schrage on 11/15/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

#ifndef pty_h
#define pty_h

#include <stdio.h>

typedef struct {
  int fd;
  int process_pid;
} Pty;


Pty* pty_init(const char* logfile);
int pty_send(Pty* pty, const char* buf, int count);
void pty_free(Pty* pty);


#endif /* pty_h */
