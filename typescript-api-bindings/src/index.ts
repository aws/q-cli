import WindowPosition from './position';
import Settings from './settings';
import EditBufferNotifications from './editbuffer';
import PTY from './pty'
import Shell from './shell'
import Keybindings from './keybindings'

import * as Fig from "./fig";
import * as Internal from "./requests"

// @ts-ignore
window.f =  { WindowPosition, Settings, EditBufferNotifications, PTY, Shell, Keybindings, Internal}

export { WindowPosition, Settings, EditBufferNotifications, PTY, Shell, Keybindings, Internal, Fig}
