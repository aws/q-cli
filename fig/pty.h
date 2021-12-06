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

int pty_init(const int fdp, const char* _Nonnull logfile);
ssize_t pty_send(const int fd, const char* _Nonnull buf, int count);
void pty_free(const int fdp, const int process_pid);

#endif /* pty_h */
