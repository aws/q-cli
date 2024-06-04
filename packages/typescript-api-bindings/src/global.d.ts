/* eslint-disable no-var, vars-on-top */

declare global {
  var fig:
    | {
        constants:
          | {
              version?: string;
              os?: string;
              supportApiProto?: boolean;
              apiProtoUrl?: string;
            }
          | undefined;
        quiet: boolean | undefined;
      }
    | undefined;

  var webkit:
    | {
        messageHandlers?: Record<string, unknown> & {
          proto?: {
            postMessage: (message: string) => void;
          };
        };
      }
    | undefined;

  var ipc:
    | {
        postMessage?: (message: string) => void;
      }
    | undefined;
}

export {};
