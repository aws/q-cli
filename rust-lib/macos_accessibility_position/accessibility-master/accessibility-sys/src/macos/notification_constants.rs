#![allow(non_upper_case_globals)]
use core_foundation_sys::base::CFIndex;

// hi grant
pub const kAXMainWindowChangedNotification: &str = "AXMainWindowChanged";
pub const kAXFocusedWindowChangedNotification: &str = "AXFocusedWindowChanged";
pub const kAXFocusedUIElementChangedNotification: &str = "AXFocusedUIElementChanged";
pub const kAXApplicationActivatedNotification: &str = "AXApplicationActivated";
pub const kAXApplicationDeactivatedNotification: &str = "AXApplicationDeactivated";
pub const kAXApplicationHiddenNotification: &str = "AXApplicationHidden";
pub const kAXApplicationShownNotification: &str = "AXApplicationShown";
pub const kAXWindowCreatedNotification: &str = "AXWindowCreated";
pub const kAXWindowMovedNotification: &str = "AXWindowMoved";
pub const kAXWindowResizedNotification: &str = "AXWindowResized";
pub const kAXWindowMiniaturizedNotification: &str = "AXWindowMiniaturized";
pub const kAXWindowDeminiaturizedNotification: &str = "AXWindowDeminiaturized";
pub const kAXDrawerCreatedNotification: &str = "AXDrawerCreated";
pub const kAXSheetCreatedNotification: &str = "AXSheetCreated";
pub const kAXHelpTagCreatedNotification: &str = "AXHelpTagCreated";
pub const kAXValueChangedNotification: &str = "AXValueChanged";
pub const kAXUIElementDestroyedNotification: &str = "AXUIElementDestroyed";
pub const kAXElementBusyChangedNotification: &str = "AXElementBusyChanged";
pub const kAXMenuOpenedNotification: &str = "AXMenuOpened";
pub const kAXMenuClosedNotification: &str = "AXMenuClosed";
pub const kAXMenuItemSelectedNotification: &str = "AXMenuItemSelected";
pub const kAXRowCountChangedNotification: &str = "AXRowCountChanged";
pub const kAXRowExpandedNotification: &str = "AXRowExpanded";
pub const kAXRowCollapsedNotification: &str = "AXRowCollapsed";
pub const kAXSelectedCellsChangedNotification: &str = "AXSelectedCellsChanged";
pub const kAXUnitsChangedNotification: &str = "AXUnitsChanged";
pub const kAXSelectedChildrenMovedNotification: &str = "AXSelectedChildrenMoved";
pub const kAXSelectedChildrenChangedNotification: &str = "AXSelectedChildrenChanged";
pub const kAXResizedNotification: &str = "AXResized";
pub const kAXMovedNotification: &str = "AXMoved";
pub const kAXCreatedNotification: &str = "AXCreated";
pub const kAXSelectedRowsChangedNotification: &str = "AXSelectedRowsChanged";
pub const kAXSelectedColumnsChangedNotification: &str = "AXSelectedColumnsChanged";
pub const kAXSelectedTextChangedNotification: &str = "AXSelectedTextChanged";
pub const kAXTitleChangedNotification: &str = "AXTitleChanged";
pub const kAXLayoutChangedNotification: &str = "AXLayoutChanged";
pub const kAXAnnouncementRequestedNotification: &str = "AXAnnouncementRequested";
pub const kAXUIElementsKey: &str = "AXUIElementsKey";
pub const kAXPriorityKey: &str = "AXPriorityKey";
pub const kAXAnnouncementKey: &str = "AXAnnouncementKey";
pub const kAXUIElementTitleKey: &str = "AXUIElementTitleKey";

pub const kAXPriorityLow: CFIndex = 10;
pub const kAXPriorityMedium: CFIndex = 50;
pub const kAXPriorityHigh: CFIndex = 90;
