import * as WindowPosition from './position';
import * as Settings from './settings';
import * as EditBufferNotifications from './editbuffer';
import * as PTY from './pty';
import * as Shell from './shell';
import * as Keybindings from './keybindings';
import * as Defaults from './defaults';
import * as Telemetry from './telemetry';
import * as fs from './filesystem';
import * as Config from './config';

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
