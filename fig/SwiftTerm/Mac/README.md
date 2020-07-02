This is the AppKit front-end to the Terminal engine.

This contains both `MacTerminalView` which is an NSView implementation
of the terminal ready to be used in an application and hooked up to
a backend as well as `LocalProcessTerminalView` which is a convenient
way of launching Unix commands like a shell inside the MacTerminalView
