declare global {
  namespace fig {
    const constants:
      | {
          version?: string;
          os?: string;
          supportApiProto?: boolean;
          apiProtoUrl?: string;
        }
      | undefined;
    const quiet: boolean | undefined;
  }

  interface Window {
    webkit?: {
      messageHandlers?: Record<string, unknown> & {
        proto?: {
          postMessage: (message: string) => void;
        };
      };
    };
    ipc?: {
      postMessage?: (message: string) => void;
    };
  }
}

export {};
