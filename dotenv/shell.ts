import * as uuid from 'uuid';
import PTY, { PTYOptions } from './pty';
import FigCLI from './figcli';
import { socketListen, removeListener } from './unix-server';

type EditBufferMessage = {
  subcommand: string;
  termSessionId: string;
  integrationVersion: string;
  tty: string;
  pid: string;
  histno: string;
  cursor: number;
  buffer: string;
  timestamp: number;
};

const parseEditbufferMessage = (data: Buffer): EditBufferMessage => {
  const original = String(Buffer.from(data.toString(), 'base64'));
  const tokens = original.slice(0, -1).split(' ');
  const buffer = tokens
    .slice(8)
    .join(' ')
    .slice(1, -1); // remove surrounding quotes
  return {
    subcommand: tokens[1],
    termSessionId: tokens[2],
    integrationVersion: tokens[3],
    tty: tokens[4],
    pid: tokens[5],
    histno: tokens[6],
    cursor: parseInt(tokens[7], 10),
    buffer,
    timestamp: Date.now(),
  };
};

class Shell {
  cli: FigCLI;

  pty: PTY;

  firstPromptPromise: Promise<void>;

  buffer = '';

  cursor = 0;

  socketId: string;

  constructor(options: PTYOptions) {
    const { env } = options;
    const envWithId = { ...(env || process.env), TERM_SESSION_ID: uuid.v4() };
    const termSessionId = envWithId.TERM_SESSION_ID;
    this.cli = new FigCLI(termSessionId);
    this.socketId = socketListen('/tmp/fig.socket', data => {
      const message = parseEditbufferMessage(data);
      if (message.termSessionId === termSessionId) {
        this.buffer = message.buffer;
        this.cursor = message.cursor;
      }
    });
    this.firstPromptPromise = this.cli.waitForNextPrompt();
    this.pty = new PTY({ ...options, env: envWithId });
  }

  initialized() {
    return this.firstPromptPromise;
  }

  kill() {
    this.cli.close();
    removeListener('/tmp/fig.socket', this.socketId);
    this.pty.kill();
  }
}

export default Shell;
