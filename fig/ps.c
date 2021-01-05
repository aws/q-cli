//
//  ps.c
//  fig
//
//  Created by Matt Schrage on 11/17/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//
#import "ps.h"

int candidates(const char *tty) {
    int mib[4] = { CTL_KERN, KERN_PROC, KERN_PROC_ALL, 0 };
    size_t bufSize = 0;

    struct kinfo_proc *kp;
    struct kinfo_proc *kprocbuf;


    // mib[0] = CTL_KERN;
    // mib[1] = KERN_PROC;
    // mib[2] = KERN_PROC_TTY;
    // mib[3] = 0;

    if (sysctl(mib, 4, NULL, &bufSize, NULL, 0) < 0) {
        perror("Failure calling sysctl");
        printf("Fail");
        return 0;
    }

    kprocbuf = kp = (struct kinfo_proc *)malloc(bufSize);

    sysctl(mib, 4, kp, &bufSize, NULL, 0);

    int nentries = bufSize / sizeof(struct kinfo_proc);
    int filtered = 0;
    for (int i = nentries; --i >= 0; ++kp) {
        // printf("Hello\n");
        if (kp->kp_proc.p_pid == 0) {
          continue;
        }

        // must have controlling tty
        if ((kp->kp_eproc.e_tdev == NODEV ||
             (kp->kp_proc.p_flag & P_CONTROLT) == 0)) {
          continue;
        }
        
        
        char *dev = malloc(MAXNAMLEN);
        
        // https://linear.app/fig/issue/ENG-44/bugfixes-to-psc
        devname_r(kp->kp_eproc.e_tdev, S_IFCHR, dev, MAXNAMLEN);
        
        
        if (dev == NULL) {
            continue;
        }
        // Incorrect checksum for freed object 0x7f92b0904c00: probably modified after being freed.
        if (strlen(tty) != 0 && strcmp(tty, dev) != 0) {
            free(dev);
            continue;
        }
        
        free(dev);
        
        struct proc_vnodepathinfo vpi;
        int ret;
        ret = proc_pidinfo(kp->kp_proc.p_pid, PROC_PIDVNODEPATHINFO, 0, &vpi, sizeof(vpi));
        
        if (ret <= 0) {
            continue;
        }

        filtered++;
    }
    free(kprocbuf);
    return filtered;
}

//struct fig_proc_info;

fig_proc_info* getProcessInfo(const char * tty, int *size) {
   int mib[4] = { CTL_KERN, KERN_PROC, KERN_PROC_ALL, 0 };
     size_t bufSize = 0;

     struct kinfo_proc *kp;
     struct kinfo_proc *kprocbuf;

     // mib[0] = CTL_KERN;
     // mib[1] = KERN_PROC;
     // mib[2] = KERN_PROC_TTY;
     // mib[3] = 0;

     if (sysctl(mib, 4, NULL, &bufSize, NULL, 0) < 0) {
         perror("Failure calling sysctl");
         printf("Fail");
         return 0;
     }

     kprocbuf = kp = (struct kinfo_proc *)malloc(bufSize);

     sysctl(mib, 4, kp, &bufSize, NULL, 0);
    
     int nentries = bufSize / sizeof(struct kinfo_proc);
    int total = candidates(tty);
    int j = total;
    *size = total;
    fig_proc_info *items = malloc(sizeof(*items) * total);


     // printf("Success! Entries = %i\n", nentries);

     for (int i = nentries; --i >= 0; ++kp) {
         // printf("Hello\n");
         if (kp->kp_proc.p_pid == 0) {

             continue;
         }

         // must have controlling tty
         if ((kp->kp_eproc.e_tdev == NODEV ||
              (kp->kp_proc.p_flag & P_CONTROLT) == 0)) {
           continue;
         }
         
        char *dev = malloc(MAXNAMLEN);
        
        // https://linear.app/fig/issue/ENG-44/bugfixes-to-psc
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
//             printf("proc: path failed");
             continue;
         }

         struct proc_vnodepathinfo vpi;
         ret = proc_pidinfo(kp->kp_proc.p_pid, PROC_PIDVNODEPATHINFO, 0, &vpi, sizeof(vpi));
         
         if (ret <= 0 ) {
//             printf("proc: vpi path failed");
             continue;
         }
         
         fig_proc_info *process;
         process = (fig_proc_info*)malloc( sizeof( fig_proc_info ) );
         process->pid = kp->kp_proc.p_pid;
         //  malloc: Incorrect checksum for freed object 0x10288f400: probably modified after being freed.
         strncpy(process->tty, dev, FIG_TTY_MAXSIZE);
         strncpy(process->cmd, pathBuffer, PROC_PIDPATHINFO_MAXSIZE);
         strncpy(process->cwd, vpi.pvi_cdir.vip_path, PATH_MAX);

         // process->tty = devname(kp->kp_eproc.e_tdev, S_IFCHR);
         // process->cmd = pathBuffer;
         // process->cwd = vpi.pvi_cdir.vip_path;
         items[total - j--] = *process;
         free(dev);
         
         free(process);
         // return process;
         // printf("pid = %d, tty = %s, CMD = %s, CWD = %s\n",kp->kp_proc.p_pid, devname(kp->kp_eproc.e_tdev, S_IFCHR), pathBuffer, vpi.pvi_cdir.vip_path);
     }

     free(kprocbuf);

     return items;
 }

int printProcesses(const char* tty) {
        

    int mib[4] = { CTL_KERN, KERN_PROC, KERN_PROC_TTY, 0 };
    size_t bufSize = 0;

    struct kinfo_proc *kp;

    // mib[0] = CTL_KERN;
    // mib[1] = KERN_PROC;
    // mib[2] = KERN_PROC_TTY;
    // mib[3] = 0;

    if (sysctl(mib, 4, NULL, &bufSize, NULL, 0) < 0) {
        perror("Failure calling sysctl");
        printf("Fail");
        return 0;
    }

    kp = (struct kinfo_proc *)malloc(bufSize);

    sysctl(mib, 4, kp, &bufSize, NULL, 0);

    int nentries = bufSize / sizeof(struct kinfo_proc);

    // printf("Success! Entries = %i\n", nentries);

    for (int i = nentries; --i >= 0; ++kp) {
        // printf("Hello\n");
        if (kp->kp_proc.p_pid == 0) {
          continue;
        }

        // must have controlling tty
        if ((kp->kp_eproc.e_tdev == NODEV ||
            (kp->kp_proc.p_flag & P_CONTROLT) == 0))
          continue;
        
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

//fig_proc_info* getProcessInfo2(const char * tty, int *size) {
//    // get number of procesess
//
//    // get processes (controlled by TTY)
//    int mib[4] = { CTL_KERN, KERN_PROC, KERN_PROC_TTY, 0 };
//
//    // filter processes + add to new pointer
//}

