//
//  WindowPositioning.swift
//  fig
//
//  Created by Matt Schrage on 8/5/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa

enum APIError: Error {
    case generic(message: String)
}

class WindowPositioning {
    
    static func frameRelativeToCursor(width: CGFloat,
                                      height: CGFloat,
                                      anchorOffset: CGPoint) throws -> (frame: CGRect, isAbove: Bool, isClipped: Bool) {
        guard let window = AXWindowServer.shared.whitelistedWindow else {
            throw APIError.generic(message: "Could not find whitelisted window")
        }
        
        guard let cursorRect = Accessibility.getTextRect() else {
            throw APIError.generic(message: "Could not find cursor rect")
        }
        
        guard let currentScreen = NSScreen.screens.filter({ (screen) -> Bool in
            return screen.frame.contains(cursorRect)
        }).first ?? NSScreen.main else {
            throw APIError.generic(message: "Could not determine main screen")
        }
        
        let maxHeight = Settings.shared.getValue(forKey: Settings.autocompleteHeight) as? CGFloat ?? 140.0
        let windowFrame = window.frame
        let screenFrame = currentScreen.frame
        
        return frameRelativeToCursor(currentScreenFrame: screenFrame,
                                     currentWindowFrame: windowFrame,
                                     cursorRect: cursorRect,
                                     width: width,
                                     height: height,
                                     anchorOffset: anchorOffset,
                                     maxHeight: maxHeight)
    }
    
    static func frameRelativeToCursor(currentScreenFrame: CGRect,
                                      currentWindowFrame: CGRect,
                                      cursorRect: CGRect,
                                      width: CGFloat,
                                      height: CGFloat,
                                      anchorOffset: CGPoint,
                                      maxHeight: CGFloat) -> (frame: CGRect, isAbove: Bool, isClipped: Bool) {
        
        let verticalPaddingFromCursor: CGFloat = 5
        var isClipped = false
        
        let popupHasSufficientVerticalSpaceToAppearInTopHalfOfCurrentWindow =
            currentWindowFrame.height < currentWindowFrame.origin.y - cursorRect.origin.y + cursorRect.height + maxHeight

            
        let popupHasSufficientVerticalSpaceToAppearOnCurrentScreen =
            cursorRect.origin.y + maxHeight <= currentScreenFrame.maxY

        let isAbove = popupHasSufficientVerticalSpaceToAppearInTopHalfOfCurrentWindow &&
                      popupHasSufficientVerticalSpaceToAppearOnCurrentScreen

              
        let translatedX = cursorRect.origin.x + anchorOffset.x
    
        let translatedOrigin = isAbove ? NSPoint(x: translatedX,
                                                 y: cursorRect.origin.y + height + verticalPaddingFromCursor) :
                                         NSPoint(x: translatedX,
                                                 y: cursorRect.origin.y - cursorRect.height - verticalPaddingFromCursor)

        let popup = NSRect(x: translatedOrigin.x,
                           y: translatedOrigin.y,
                           width: width,
                           height: height)
        
        let overhang = (currentScreenFrame.maxX) - popup.maxX
        
        if (overhang < 0) {
            isClipped = true
        }
        
    
        let frame = NSRect(x: popup.origin.x + (isClipped ? overhang : 0),
                           y: popup.origin.y,
                           width: popup.width,
                           height: popup.height)
        
        return (frame: frame, isAbove: isAbove, isClipped: isClipped)
        
    }
    
    static func isValidFrameRelativeToCursor(currentScreenFrame: CGRect,
                                             currentWindowFrame: CGRect,
                                             cursorRect: CGRect,
                                             width: CGFloat,
                                             height: CGFloat,
                                             anchorOffset: CGPoint,
                                             maxHeight: CGFloat) -> Bool {

        return WindowPositioning.frameRelativeToCursor(currentScreenFrame: currentScreenFrame,
                                                currentWindowFrame: currentWindowFrame,
                                                cursorRect: cursorRect,
                                                width: width,
                                                height: height,
                                                anchorOffset: anchorOffset,
                                                maxHeight: maxHeight).isClipped
        
    }
}


//extension WindowPositioning: APIProvider {
//    func namespace() -> String? {
//        return "positioning"
//    }
//    
//    func handlers() -> [APIRequestHandler] {
//        let frameParameters = [
//            FigAPIParameter(key: "width", type: Int.self),
//            FigAPIParameter(key: "height", type: Int.self),
//            FigAPIParameter(key: "anchorX", type: Int.self, optional: true),
//        ]
//        let isValidFrameHandler = APIRequestHandler(identifier: "isValidFrame",
//                                                    parameters: frameParameters,
//                                                    function: isValidFrame)
//        let setFrameHandler = APIRequestHandler(identifier: "setFrame",
//                                                parameters: frameParameters,
//                                                function: setFrame)
//        return [
//            isValidFrameHandler,
//            setFrameHandler
//        ]
//    }
//    
//    fileprivate func isValidFrame(payload: APIRequestPayload, callback: APICallback) {
//        let width = payload["width"] as! Int
//        let height = payload["height"] as! Int
//        let anchorX = payload["anchorX"] as? Int
//        
//        // Run code!
//        
//        if let callback = callback {
//            callback("hello")
//        }
//    }
//    
//    fileprivate func setFrame(payload: APIRequestPayload, callback: APICallback) {
//        print(payload)
//    }
//
//}
//
//
//struct FigAPIParameter {
//    let type: Any.Type
//    let key: String
//    let optional: Bool?
//    let defaultValue: Any?
//    init(key: String, type: Any.Type, optional: Bool = false, defaultValue: Any? = nil)  {
//        self.key = key
//        self.type = type
//        self.optional = optional
//        self.defaultValue = defaultValue
//    }
//}
//
//typealias APIRequestPayload = [String: Any]
//typealias APICallback = ((Any) -> Void)?
//
//struct APIRequestHandler {
//    let identifier: String
//    let parameters: [ FigAPIParameter ]
//    let function: (APIRequestPayload, APICallback) -> Void
//}
//
//struct FigAPIRequest {
//    let type: String
//    let data: [String: Any]
//    let callbackId: String?
//    let version: Int
//    init(type: String, data: [String: Any], callbackId: String?, version: Int = 0) {
//        self.type = type
//        self.data = data
//        self.callbackId = callbackId
//        self.version = version
//    }
//}
//
//protocol APIProvider {
//    func handlers() -> [APIRequestHandler]
//    func namespace() -> String?
//}

//protocol APIBroker {
//    func registerRequestHandler(_ handler: APIRequestHandler)
//    func removeRequestHandler(_ handler: APIRequestHandler)
//}
//
//class GenericAPIProvider: APIProvider {
//    let handlers: [APIRequestHandler]
//    let broker:
//    init(manager: APIBroker) {
//
//    }
//
//    static func registerHandlers() {
//
//    }
//
//    func shouldHandleRequest(_ request: FigAPIRequest) -> Bool {
//        <#code#>
//    }
//
//    func handle(_ request: FigAPIRequest) {
//        <#code#>
//    }
//
//
//}
