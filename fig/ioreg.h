//
//  ioreg.h
//  fig
//
//  Created by Matt Schrage on 12/9/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

#ifndef ioreg_h
#define ioreg_h

// https://github.com/koekeishiya/skhd/blob/master/src/carbon.c
#include <Carbon/Carbon.h>
#include <CoreFoundation/CoreFoundation.h>
#include <objc/objc-runtime.h>

extern CFDictionaryRef CGSCopyCurrentSessionDictionary(void);
extern bool CGSIsSecureEventInputSet(void);

void
secure_keyboard_entry_process_info(pid_t *pid);

#endif /* ioreg_h */
