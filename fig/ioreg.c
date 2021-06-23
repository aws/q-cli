//
//  ioreg.c
//  fig
//
//  Created by Matt Schrage on 12/9/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

#include "ioreg.h"

void
secure_keyboard_entry_process_info(pid_t *pid)
{
    
    CFDictionaryRef session = CGSCopyCurrentSessionDictionary();
    if (!session) return;

    CFNumberRef pid_ref = (CFNumberRef) CFDictionaryGetValue(session, CFSTR("kCGSSessionSecureInputPID"));
    if (pid_ref != NULL) {
        CFNumberGetValue(pid_ref, CFNumberGetType(pid_ref), pid);
    }

    CFRelease(session);
}
