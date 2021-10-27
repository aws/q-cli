//
//  PrivateWindow.h
//  fig
//
//  Created by Matt Schrage on 6/28/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

#import <Foundation/Foundation.h>
#import <ApplicationServices/ApplicationServices.h>

NS_ASSUME_NONNULL_BEGIN

@interface PrivateWindow : NSObject
+ (CGWindowID) getCGWindowIDFromRef:(AXUIElementRef)ref;

@end
extern AXError _AXUIElementGetWindow(AXUIElementRef, CGWindowID* out);

NS_ASSUME_NONNULL_END
