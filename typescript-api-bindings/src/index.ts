import * as Fig from "@fig/fig-api-proto/dist/fig.pb";
import * as WindowPosition from "./position";
import * as Settings from "./settings";
import * as EditBufferNotifications from "./editbuffer";
import * as PTY from "./pty";
import * as Process from "./process";
import * as Shell from "./shell";
import * as Keybindings from "./keybindings";
import * as Event from "./event";
import * as Defaults from "./defaults";
import * as Telemetry from "./telemetry";
import * as fs from "./filesystem";
import * as Config from "./config";
import * as Native from "./native";
import * as Debugger from "./debugger";
import * as State from "./state";
import * as Install from "./install";
import * as Types from "./types";
import * as User from "./user";
import * as History from "./history";

import * as Internal from "./requests";

const lib = {
  Config,
  Debugger,
  Defaults,
  EditBufferNotifications,
  Event,
  Internal,
  Keybindings,
  Native,
  PTY,
  Process,
  Settings,
  Shell,
  State,
  Telemetry,
  WindowPosition,
  fs,
  Install,
  Types,
  User,
  History
};

export {
  Config,
  Debugger,
  Defaults,
  EditBufferNotifications,
  Event,
  Fig,
  fs,
  History,
  Install,
  Internal,
  Keybindings,
  Native,
  Process,
  PTY,
  Settings,
  Shell,
  State,
  Telemetry,
  Types,
  User,
  WindowPosition
};

declare global {
  interface Window {
    f: typeof lib;
  }
}

window.f = lib;
