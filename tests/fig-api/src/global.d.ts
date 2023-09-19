declare global {
  interface Window {
    ipc: {
      postMessage?: (message: string) => void;
    };
    fig: {
      constants?: {
        os?: string;
        arch?: string;
        cli?: string;
        version?: string;
        home?: string;
        user?: string;
        env?: Record<string, string>;
        figDotDir?: string;
        figDataDir?: string;
      };
      quiet?: boolean;
    };
  }
}

export {};
