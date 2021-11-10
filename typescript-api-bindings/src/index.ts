import WindowPosition from './position';
import Settings from './settings';
import EditBufferNotifications from './editbuffer';
import PTY from './pty';
import Shell from './shell';
import Keybindings from './keybindings';
import Defaults from './defaults';
import Telemetry from './telemetry';
import fs from './filesystem';
import Config from './config';

import * as Fig from './fig';
import * as Internal from './requests';

// @ts-ignore
window.f = {
  WindowPosition,
  Settings,
  EditBufferNotifications,
  PTY,
  Shell,
  Keybindings,
  Defaults,
  Telemetry,
  fs,
  Config,
  Internal,
};

export {
  WindowPosition,
  Settings,
  EditBufferNotifications,
  PTY,
  Shell,
  Keybindings,
  Defaults,
  Telemetry,
  fs,
  Config,
  Internal,
  Fig,
};
