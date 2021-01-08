//
//  lsof.h
//  fig
//
//  Created by Matt Schrage on 1/7/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

#import <Foundation/Foundation.h>
#include <sys/types.h>
#include <sys/sysctl.h>

NS_ASSUME_NONNULL_BEGIN

@interface lsof : NSObject
+ (NSString *)argumentsFromPid:(pid_t)pid;

@end

NS_ASSUME_NONNULL_END
