//
//  API+Icon.swift
//  fig
//
//  Created by Matt Schrage on 10/29/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation
import WebKit
import Cocoa

class Icon {
    static func fileIcon(for string: String) -> NSImage? {
      guard let escapedString = string.addingPercentEncoding(withAllowedCharacters: .urlQueryAllowed),
            let url = URL(string: escapedString) else {
        return nil
      }
      
      return fileIcon(for: url)
    }
    static func fileIcon(for url: URL) -> NSImage? {
        
        var width = 32.0
        var height = 32.0
        var color: NSColor?
        var badge: String?

        if let qs = url.queryDictionary, let w = qs["w"], let wd = Double(w), let h = qs["h"], let hd = Double(h) {
            width = wd
            height = hd
        }
        
        if let qs = url.queryDictionary {
            color = NSColor(hex: qs["color"] ?? "")
            badge = qs["badge"]
        }
        
        // fig://template?background-color=ccc&icon=box
        if let host = url.host, host == "template" {
          guard let icon = Bundle.main.image(forResource: "template") else { return nil }
          return icon.overlayColor(color).overlayText(badge).resized(to: NSSize(width: width, height: height))//?.overlayBadge(color: color,  text: badge)


        }
      
        // fig://icon?asset=git
        if let host = url.host, let qs = url.queryDictionary, let type = qs["asset"], host == "icon" {
            let icon = Bundle.main.image(forResource: type) ?? Bundle.main.image(forResource: "box")!
            return icon.resized(to: NSSize(width: width, height: height))?.overlayBadge(color: color,  text: badge)
        }

        // fig://icon?type=mp4
        if let host = url.host, let qs = url.queryDictionary, let type = qs["type"], host == "icon" {
            if let icon = Bundle.main.image(forResource: type) {
                return icon.resized(to: NSSize(width: width, height: height))?.overlayBadge(color: color,  text: badge)
            }
            
            var t = type
            if (type == "folder") {
                t = NSFileTypeForHFSTypeCode(OSType(kGenericFolderIcon))
            }
            return NSWorkspace.shared.icon(forFileType: t).resized(to: NSSize(width: width, height: height))?.overlayBadge(color: color,  text: badge)

        }
        
        if let host = url.host, let qs = url.queryDictionary, let pid = qs["pid"], host == "icon" {

            return NSRunningApplication(processIdentifier: pid_t(pid) ?? -1)?.icon?.resized(to: NSSize(width: width, height: height))?.overlayBadge(color: color,  text: badge)
        }

        guard var specifier = (url as NSURL).resourceSpecifier else { return nil }
        if (specifier.prefix(2) == "//") { specifier = String(specifier.dropFirst(2)) }
//        if (specifier.prefix(1) !=  "/") { specifier = "/" + specifier }
        let resource = specifier.replacingOccurrences(of: "?\(url.query ?? "<none>")", with: "") as NSString
        let fullPath = resource.expandingTildeInPath.removingPercentEncoding ?? ""
        
        var isDirectory : ObjCBool = false
        let isFile = FileManager.default.fileExists(atPath: fullPath, isDirectory:&isDirectory)
        guard isFile || isDirectory.boolValue else {
            var t = NSString(string: fullPath).pathExtension
            if (String(resource).last == "/") {
                t = NSFileTypeForHFSTypeCode(OSType(kGenericFolderIcon))
            }
            
            return NSWorkspace.shared.icon(forFileType: t).resized(to: NSSize(width: width, height: height))?.overlayBadge(color: color,  text: badge)
        }
        
        return NSWorkspace.shared.icon(forFile: fullPath).resized(to: NSSize(width: width, height: height))?.overlayBadge(color: color,  text: badge)
        
    }
}

extension NSImage {
    func resized(to newSize: NSSize) -> NSImage? {
        if let rep = self.bestRepresentation(for: NSRect(origin: .zero, size: newSize), context: NSGraphicsContext.current, hints: nil) {

            let resizedImage = NSImage(size: newSize)
            resizedImage.addRepresentation(rep)
            return resizedImage
        }

        return nil
    }
    
    func overlayAppIcon() -> NSImage {
        let background = self
        // let side:CGFloat = 32

        let overlay = NSImage(imageLiteralResourceName: NSImage.applicationIconName)//.resized(to: NSSize(width:  background.size.width/2, height:  background.size.height/2))!
        
        let newImage = NSImage(size: background.size)
        newImage.lockFocus()

        var newImageRect: CGRect = .zero
        newImageRect.size = newImage.size
        
        background.draw(in: newImageRect)
        overlay.draw(in: NSRect(x: background.size.width/2, y: 0, width: background.size.width/2 - 4, height: background.size.height/2 - 4))

        newImage.unlockFocus()
        return newImage//.resized(to: NSSize(width: background.size.width * 1.5, height: background.size.height * 1.5))!
    }
  
  func overlayImage(_ image: NSImage) -> NSImage {
        let background = self
        // let side:CGFloat = 32

        let overlay = image//.resized(to: NSSize(width:  background.size.width/2, height:  background.size.height/2))!
        
        let newImage = NSImage(size: background.size)
        newImage.lockFocus()

        var newImageRect: CGRect = .zero
        newImageRect.size = newImage.size
        
        background.draw(in: newImageRect)
        overlay.draw(in: NSRect(x: background.size.width/2, y: 0, width: background.size.width/2 - 4, height: background.size.height/2 - 4))

        newImage.unlockFocus()
        return newImage//.resized(to: NSSize(width: background.size.width * 1.5, height: background.size.height * 1.5))!
    }
  
    func overlayColor(_ color: NSColor?) -> NSImage {
      guard let color = color, let bitmapRep = NSBitmapImageRep(bitmapDataPlanes: nil,
                                                           pixelsWide: Int(self.size.width),
                                                           pixelsHigh: Int(self.size.height),
                                                           bitsPerSample: 8,
                                                           samplesPerPixel: 4,
                                                           hasAlpha: true,
                                                           isPlanar: false,
                                                           colorSpaceName: .calibratedRGB,
                                                           bytesPerRow: 0,
                                                           bitsPerPixel: 0) else { return self }
      bitmapRep.size = self.size
      NSGraphicsContext.saveGraphicsState()
      NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: bitmapRep)
      draw(in: NSRect(x: 0, y: 0, width: self.size.width, height: self.size.height), from: .zero, operation: .copy, fraction: 1.0)
      NSGraphicsContext.restoreGraphicsState()

      self.addRepresentation(bitmapRep)
      
      guard let cgImage = self.cgImage(forProposedRect: nil, context: nil, hints: nil) else { return self }

      self.lockFocus()
      color.set()
      
      guard let context = NSGraphicsContext.current?.cgContext else { return self }
      let imageRect = NSRect(origin: NSZeroPoint, size: self.size)

      context.clip(to: imageRect, mask: cgImage)
      imageRect.fill(using: .darken)
      self.unlockFocus()

      
      return self
    }
  
  func overlayText(_ text: String?) -> NSImage {
    guard let text = text, let bitmapRep = NSBitmapImageRep(bitmapDataPlanes: nil,
                                                         pixelsWide: Int(self.size.width),
                                                         pixelsHigh: Int(self.size.height),
                                                         bitsPerSample: 8,
                                                         samplesPerPixel: 4,
                                                         hasAlpha: true,
                                                         isPlanar: false,
                                                         colorSpaceName: .calibratedRGB,
                                                         bytesPerRow: 0,
                                                         bitsPerPixel: 0) else { return self }
    bitmapRep.size = self.size
    NSGraphicsContext.saveGraphicsState()
    NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: bitmapRep)
    draw(in: NSRect(x: 0, y: 0, width: self.size.width, height: self.size.height), from: .zero, operation: .copy, fraction: 1.0)
    NSGraphicsContext.restoreGraphicsState()

    self.addRepresentation(bitmapRep)
    
    self.lockFocus()
    
    let imageRect = NSRect(origin: NSZeroPoint, size: self.size)
    let paragraphStyle: NSMutableParagraphStyle = NSMutableParagraphStyle()
    paragraphStyle.alignment = NSTextAlignment.center
    
    let string = NSAttributedString(string: text,
                                    attributes: [ NSAttributedString.Key.font : NSFont.systemFont(ofSize: floor(imageRect.height * 0.65)),
                                                  NSAttributedString.Key.foregroundColor : NSColor.white,
                                                  NSAttributedString.Key.paragraphStyle : paragraphStyle])

    
    string.draw(in: imageRect.insetBy(dx: 0, dy: imageRect.height * 0.1))
    self.unlockFocus()
    return self
  }
  
    func overlayBadge(color: NSColor?, text: String?) -> NSImage {
        guard color != nil || text != nil else {
            return self
        }
        
        if let bitmapRep = NSBitmapImageRep(
            bitmapDataPlanes: nil, pixelsWide: Int(self.size.width), pixelsHigh: Int(self.size.height),
            bitsPerSample: 8, samplesPerPixel: 4, hasAlpha: true, isPlanar: false,
            colorSpaceName: .calibratedRGB, bytesPerRow: 0, bitsPerPixel: 0
        ) {
            bitmapRep.size = self.size
            NSGraphicsContext.saveGraphicsState()
            NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: bitmapRep)
            draw(in: NSRect(x: 0, y: 0, width: self.size.width, height: self.size.height), from: .zero, operation: .copy, fraction: 1.0)
            NSGraphicsContext.restoreGraphicsState()

            self.addRepresentation(bitmapRep)
            self.lockFocus()

             let rect = NSMakeRect(size.width/2, 0, size.width/2, size.height/2)
             let ctx = NSGraphicsContext.current?.cgContext
//             ctx!.clear(rect)
            ctx!.setFillColor((color ?? NSColor.clear).cgColor)
             ctx!.fillEllipse(in: rect)
            
            if let text = text {
                let paragraphStyle: NSMutableParagraphStyle = NSMutableParagraphStyle()
                paragraphStyle.alignment = NSTextAlignment.center
                
                let string = NSAttributedString(string: text,
                                                attributes: [ NSAttributedString.Key.font : NSFont.systemFont(ofSize: rect.height * 0.9),
                                                              NSAttributedString.Key.foregroundColor : NSColor.white,
                                                              NSAttributedString.Key.paragraphStyle : paragraphStyle])

                
                string.draw(in: rect)
            }

            self.unlockFocus()
            return self
        }
        
        return self
    }
}


extension URL {
    var queryDictionary: [String: String]? {
        var dict = [String:String]()

        if let components = URLComponents(url: self, resolvingAgainstBaseURL: false) {
          if let queryItems = components.queryItems {
            for item in queryItems where item.value != nil {
              dict[item.name] = item.value!
            }
          }
          return dict
        } else {
          return [:]
        }
    }
}
