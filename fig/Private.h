//
//  Private.h
//  fig
//
//  Created by Matt Schrage on 6/28/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

#ifndef Private_h
#define Private_h

//extern "C" AXError _AXUIElementGetWindow(AXUIElementRef, CGWindowID* out);

@interface Private

+ (CGWindowID* ) getCGWindowID(ref: AXUIElementRef);

@end






#endif /* Private_h */
