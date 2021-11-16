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


Pty* _Nullable pty_init(const char* _Nonnull executable, char* _Nullable const* _Nonnull args, char* _Nullable const* _Nonnull env, const char* _Nonnull logfile);
ssize_t pty_send(Pty* _Nullable pty, const char* _Nonnull buf, int count);
void pty_free(Pty* _Nullable pty);


#endif /* pty_h */
