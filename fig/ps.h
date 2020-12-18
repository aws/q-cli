//
//  ps.h
//  fig
//
//  Created by Matt Schrage on 11/17/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

#ifndef ps_h
#define ps_h

#include <sys/types.h>
#include <sys/sysctl.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/proc_info.h>
#include <libproc.h>
#include <sys/stat.h>
#include <string.h>
#include <time.h>
#include <sys/syslimits.h>


#define FIG_TTY_MAXSIZE 20

//#define FIG_CWD_MAXSIZE 255 PATH_MAX

typedef struct fig_proc_info {
  pid_t pid;
  char tty[FIG_TTY_MAXSIZE];
  char cmd[PROC_PIDPATHINFO_MAXSIZE];
  char cwd[PATH_MAX];
} fig_proc_info;

fig_proc_info* getProcessInfo(const char *tty, int *size);
int candidates(const char * tty);
int printProcesses(const char * tty);

int getProcessInfoForPid(pid_t pid, char *cwd, char *cmd);

#endif /* ps_h */
