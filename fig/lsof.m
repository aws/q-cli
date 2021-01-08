//
//  lsof.m
//  fig
//
//  Created by Matt Schrage on 1/7/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

#import "lsof.h"

@implementation lsof
+ (NSString *)argumentsFromPid:(pid_t)pid {
    int mib[3], argmax, nargs, c = 0;
    size_t size;
    char *procargs, *sp, *np, *cp;
    
    mib[0] = CTL_KERN;
    mib[1] = KERN_ARGMAX;
    
    size = sizeof(argmax);
    if (sysctl(mib, 2, &argmax, &size, NULL, 0) == -1) {
        return nil;
    }
    
    procargs = (char *)malloc(argmax);
    if (procargs == NULL) {
        return nil;
    }
    
    /*
     * Make a sysctl() call to get the raw argument space of the process.
     * The layout is documented in start.s, which is part of the Csu
     * project.  In summary, it looks like:
     *
     * /---------------\ 0x00000000
     * :               :
     * :               :
     * |---------------|
     * | argc          |
     * |---------------|
     * | arg[0]        |
     * |---------------|
     * :               :
     * :               :
     * |---------------|
     * | arg[argc - 1] |
     * |---------------|
     * | 0             |
     * |---------------|
     * | env[0]        |
     * |---------------|
     * :               :
     * :               :
     * |---------------|
     * | env[n]        |
     * |---------------|
     * | 0             |
     * |---------------| <-- Beginning of data returned by sysctl() is here.
     * | argc          |
     * |---------------|
     * | exec_path     |
     * |:::::::::::::::|
     * |               |
     * | String area.  |
     * |               |
     * |---------------| <-- Top of stack.
     * :               :
     * :               :
     * \---------------/ 0xffffffff
     */
    
    mib[0] = CTL_KERN;
    mib[1] = KERN_PROCARGS2;
    mib[2] = pid;
    
    size = (size_t)argmax;
    if (sysctl(mib, 3, procargs, &size, NULL, 0) == -1) {
        goto Bail;
    }
    
    memcpy(&nargs, procargs, sizeof(nargs));
    cp = procargs + sizeof(nargs);
    
    /* Skip the saved exec_path. */
    for (; cp < &procargs[size]; cp++) {
        if (*cp == '\0') {
            break;
        }
    }
    
    if (cp == &procargs[size]) {
        goto Bail;
    }
    
    /* Skip trailing '\0' characters. */
    for (; cp < &procargs[size]; cp++) {
        if (*cp != '\0') {
            /* Beginning of first argument reached. */
            break;
        }
    }
    
    if (cp == &procargs[size]) {
        goto Bail;
    }
    
    /* Save where the argv[0] string starts. */
    sp = cp;
    
    /*
     * Iterate through the '\0'-terminated strings and convert '\0' to ' '
     * until a string is found that has a '=' character in it (or there are
     * no more strings in procargs).  There is no way to deterministically
     * know where the command arguments end and the environment strings
     * start, which is why the '=' character is searched for as a heuristic.
     */
    BOOL show_args = YES;
    for (np = NULL; c < nargs && cp < &procargs[size]; cp++) {
        if (*cp == '\0') {
            c++;
            if (np != NULL) {
                /* Convert previous '\0'. */
                *np = ' ';
            } else {
                /* *argv0len = cp - sp; */
            }
            /* Note location of current '\0'. */
            np = cp;
            
            if (!show_args) {
                break;
            }
        }
    }
    
    if (np == NULL || np == sp) {
        /* Empty or unterminated string. */
        goto Bail;
    }
    
    printf("%s\n", sp);
    
    {
        NSString *cmd = [NSString stringWithFormat:@"%s", sp];
        free(procargs);

        return cmd;
    }
    
Bail:
    free(procargs);
    return nil;
}
@end
