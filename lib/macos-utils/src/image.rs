use std::path::Path;

use appkit_nsworkspace_bindings::{
    INSWorkspace,
    NSData,
    NSFileTypeDirectory,
    NSUInteger,
    NSWorkspace,
    NSWorkspace_NSDeprecated,
};
use cocoa::appkit::NSImage;
use cocoa::base::{
    id,
    nil as NIL,
};
use cocoa::foundation::{
    NSPoint,
    NSRect,
    NSSize,
};
use objc::rc::autoreleasepool;
use objc::runtime::Object;
use objc::{
    class,
    msg_send,
    sel,
    sel_impl,
};

use super::util::NSString;

const PNG_REPRESENTATION: u64 = 4;

#[allow(clippy::missing_safety_doc)]
unsafe fn resize_image(image: id, size: NSSize) -> Option<id> {
    let rect = NSRect {
        origin: NSPoint { x: 0.0, y: 0.0 },
        size,
    };

    let ns_graphics = class!(NSGraphicsContext);
    let context: id = msg_send![ns_graphics, currentContext];
    let rep = image.bestRepresentationForRect_context_hints_(rect, context, NIL);

    if rep == NIL {
        return None;
    };

    let ns_image_cls = class!(NSImage);
    let image: id = msg_send![ns_image_cls, alloc];
    let image: id = msg_send![image, initWithSize: size];
    image.addRepresentation_(rep);

    Some(image)
}

#[allow(clippy::missing_safety_doc)]
pub unsafe fn png_for_name(name: &str) -> Option<Vec<u8>> {
    autoreleasepool(|| {
        let shared = NSWorkspace::sharedWorkspace();
        let image = shared.iconForFileType_(NSString::from(name).to_appkit_nsstring()).0;
        convert_image(image)
    })
}

#[allow(clippy::missing_safety_doc)]
pub unsafe fn png_for_path(path: &Path) -> Option<Vec<u8>> {
    autoreleasepool(|| {
        let shared = NSWorkspace::sharedWorkspace();
        let image = if path.exists() {
            let file_path: NSString = path.to_str()?.into();
            shared.iconForFile_(file_path.to_appkit_nsstring()).0
        } else {
            let is_dir = std::fs::metadata(path).ok().map(|meta| meta.is_dir()).unwrap_or(false);
            let file_type: NSString = if is_dir {
                NSFileTypeDirectory.into()
            } else {
                path.extension()?.to_str()?.into()
            };
            shared.iconForFileType_(file_type.to_appkit_nsstring()).0
        };

        convert_image(image)
    })
}

unsafe fn convert_image(image: *mut Object) -> Option<Vec<u8>> {
    let image = resize_image(image, NSSize {
        width: 32.0,
        height: 32.0,
    })?;
    let tiff_data = image.TIFFRepresentation();

    let ns_bitmap_cls = class!(NSBitmapImageRep);
    let image_rep: id = msg_send![ns_bitmap_cls, imageRepWithData: tiff_data];

    let png_data: NSData = msg_send![
        image_rep,
        representationUsingType: PNG_REPRESENTATION properties: NIL
    ];

    let len: NSUInteger = msg_send![png_data, length];
    let bytes: *const u8 = msg_send![png_data, bytes];
    let slice = std::slice::from_raw_parts(bytes, len as usize);

    Some(slice.into())
}
