//
//  PrivateWindow.m
//  fig
//
//  Created by Matt Schrage on 6/28/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

#import "PrivateWindow.h"

@implementation PrivateWindow

+ (CGWindowID) getCGWindowIDFromRef: (AXUIElementRef)ref {
    CGWindowID winID;
    AXError err = _AXUIElementGetWindow(ref, &winID);
    
    if (err) {
        NSLog(@"%i", err);
    }
    return winID;
}
@end
