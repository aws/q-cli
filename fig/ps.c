//
//  ps.c
//  fig
//
//  Created by Matt Schrage on 11/17/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

#import "ps.h"

fig_proc_info* getProcessInfo(const char * tty, int *size) {
  size_t bufSize = 0;
  struct kinfo_proc *kp;
  struct kinfo_proc *kprocbuf;
  int mib[4] = { CTL_KERN, KERN_PROC, KERN_PROC_ALL, 0 };
  
  if (sysctl(mib, 4, NULL, &bufSize, NULL, 0) < 0) {
    perror("Failure calling sysctl");
    printf("Fail");
    return 0;
  }
  
  kprocbuf = kp = (struct kinfo_proc *)malloc(bufSize);
  sysctl(mib, 4, kp, &bufSize, NULL, 0);
  int nentries = (int) bufSize / sizeof(struct kinfo_proc);
  
  size_t count = 0;
  size_t itemBuffSize = 5;
  fig_proc_info *items = malloc(itemBuffSize * sizeof *items);
  
  for (int i = nentries; --i >= 0; ++kp) {
    if (kp->kp_proc.p_pid == 0) {
      continue;
    }
    // must have controlling tty
    if ((kp->kp_eproc.e_tdev == NODEV || (kp->kp_proc.p_flag & P_CONTROLT) == 0)) {
      continue;
    }
    // https://linear.app/fig/issue/ENG-44/bugfixes-to-psc
    char *dev = malloc(MAXNAMLEN);
    
    devname_r(kp->kp_eproc.e_tdev, S_IFCHR, dev, MAXNAMLEN);
    if (dev == NULL) {
      continue;
    }
    if (strlen(tty) != 0 && strcmp(tty, dev) != 0) {
      free(dev);
      continue;
    }
    
    int ret;
    char pathBuffer[PROC_PIDPATHINFO_MAXSIZE];
    bzero(pathBuffer, PROC_PIDPATHINFO_MAXSIZE);
    ret = proc_pidpath(kp->kp_proc.p_pid, pathBuffer, sizeof(pathBuffer));
    if (ret <= 0 ) {
      continue;
    }
    
    struct proc_vnodepathinfo vpi;
    ret = proc_pidinfo(kp->kp_proc.p_pid, PROC_PIDVNODEPATHINFO, 0, &vpi, sizeof(vpi));
    if (ret <= 0 ) {
      continue;
    }
    
    fig_proc_info *process;
    process = (fig_proc_info*)malloc( sizeof( fig_proc_info ) );
    process->pid = kp->kp_proc.p_pid;
    // malloc: Incorrect checksum for freed object 0x10288f400: probably modified after being freed.
    strncpy(process->tty, dev, FIG_TTY_MAXSIZE);
    strncpy(process->cmd, pathBuffer, PROC_PIDPATHINFO_MAXSIZE);
    strncpy(process->cwd, vpi.pvi_cdir.vip_path, PATH_MAX);
    // append process to items array
    if (count == itemBuffSize) {
      // need to extend our buffer
      fig_proc_info *tmp = realloc(items, count * 2 * sizeof *items);
      if (tmp) {
        // success - update variable
        printf("needed to realloc");
        items = tmp;
        itemBuffSize *= 2;
      } else {
        // realloc failed to extend the buffer; original buffer is left intact.
        // naively ignore for now... should free original and remalloc, but this should never happen
        printf("couldn't realloc!!!");
        return 0;
      }
    }
    items[count++] = *process;
    free(dev);
    free(process);
  }
  free(kprocbuf);
  *size = (int) count;
  return items;
}

int printProcesses(const char* tty) {
  int mib[4] = { CTL_KERN, KERN_PROC, KERN_PROC_TTY, 0 };
  size_t bufSize = 0;
  struct kinfo_proc *kp;
  
  if (sysctl(mib, 4, NULL, &bufSize, NULL, 0) < 0) {
    perror("Failure calling sysctl");
    printf("Fail");
    return 0;
  }
  
  kp = (struct kinfo_proc *)malloc(bufSize);
  sysctl(mib, 4, kp, &bufSize, NULL, 0);
  int nentries = (int) bufSize / sizeof(struct kinfo_proc);
  
  for (int i = nentries; --i >= 0; ++kp) {
    if (kp->kp_proc.p_pid == 0) {
      continue;
    }
    // must have controlling tty
    if ((kp->kp_eproc.e_tdev == NODEV || (kp->kp_proc.p_flag & P_CONTROLT) == 0)) {
      continue;
    }
    if (strlen(tty) != 0 && strcmp(tty, devname(kp->kp_eproc.e_tdev, S_IFCHR)) != 0) {
      continue;
    }
    char pathBuffer[PROC_PIDPATHINFO_MAXSIZE];
    bzero(pathBuffer, PROC_PIDPATHINFO_MAXSIZE);
    proc_pidpath(kp->kp_proc.p_pid, pathBuffer, sizeof(pathBuffer));
    struct proc_vnodepathinfo vpi;
    proc_pidinfo(kp->kp_proc.p_pid, PROC_PIDVNODEPATHINFO, 0, &vpi, sizeof(vpi));
    printf("pid = %d, tty = %s, CMD = %s, CWD = %s\n",kp->kp_proc.p_pid, devname(kp->kp_eproc.e_tdev, S_IFCHR), pathBuffer, vpi.pvi_cdir.vip_path);
  }
  return 0;
}
