import WindowPosition from './position';
import Settings from './settings';
import EditBufferNotifications from './editbuffer';
import PTY from './pty'
import Shell from './shell'
import Keybindings from './keybindings'
import fs from './filesystem'
import * as Fig from "./fig";
import * as Internal from "./requests"

// @ts-ignore
window.f =  { WindowPosition, Settings, EditBufferNotifications, PTY, Shell, Keybindings, fs, Internal}

export { WindowPosition, Settings, EditBufferNotifications, PTY, Shell, Keybindings, fs, Internal, Fig}
