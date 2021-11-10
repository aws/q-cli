import * as uuid from 'uuid';
import PTY, { PTYOptions } from './pty';
import FigCLI from './figcli';
import { socketListen, removeListener } from './unix-server';
import { LocalMessage } from './local';

const parseMessages = (data: Buffer): LocalMessage[] => {
  const messages: LocalMessage[] = [];
  let buf = data;
  while (buf.length > 0) {
    const dataType = buf.slice(2, 10).toString();
    const lenBytes = buf.slice(10, 18);
    let len = 0;
    for (let i = 0; i < lenBytes.length; i += 1) {
      len = len * 256 + lenBytes[i];
    }

    const msg = buf.slice(18, 18 + len);
    if (dataType === 'fig-json') {
      messages.push(LocalMessage.fromJSON(JSON.parse(msg.toString())));
    } else {
      messages.push(LocalMessage.decode(msg));
    }
    buf = buf.slice(18 + len);
  }
  return messages;
};

class Shell {
  cli: FigCLI;

  pty: PTY;

  firstPromptPromise: Promise<void>;

  nextPromptPromise: Promise<void>;

  nextPromptCallback: () => void;

  sessionId: string;

  buffer = '';

  cursor = 0;

  tmpdir = '/tmp/';

  socketId: string;

  startupTime = -1;

  constructor(options: PTYOptions) {
    const { env } = options;
    const envWithId = {
      TMPDIR: '/tmp/',
      ...(env || process.env),
      TERM_SESSION_ID: uuid.v4(),
    };
    this.sessionId = envWithId.TERM_SESSION_ID;
    this.tmpdir = envWithId.TMPDIR;
    this.cli = new FigCLI(this.sessionId);

    this.nextPromptCallback = () => {};
    this.nextPromptPromise = new Promise<void>(resolve => {
      this.nextPromptCallback = resolve;
    });
    this.firstPromptPromise = this.nextPromptPromise;

    this.socketId = socketListen(`${this.tmpdir}fig.socket`, data => {
      const messages = parseMessages(data);
      messages.forEach(message => this.onMessage(message));
    });

    const start = Date.now();
    this.firstPromptPromise.then(() => {
      this.startupTime = Date.now() - start;
    });
    this.pty = new PTY({ ...options, env: envWithId });
  }

  initialized() {
    return this.firstPromptPromise;
  }

  waitForNextPrompt() {
    return this.nextPromptPromise;
  }

  onMessage(message: LocalMessage) {
    /* eslint-disable no-case-declarations */
    switch (message.type?.$case) {
      case 'hook':
        const { hook } = message.type.hook;
        switch (hook?.$case) {
          case 'init':
            break;
          case 'prompt':
            if (hook.prompt.context?.sessionId === this.sessionId) {
              this.nextPromptCallback();
              this.nextPromptPromise = new Promise<void>(resolve => {
                this.nextPromptCallback = resolve;
              });
            }
            break;
          case 'editBuffer':
            if (hook.editBuffer.context?.sessionId === this.sessionId) {
              this.buffer = hook.editBuffer.text;
              this.cursor = hook.editBuffer.cursor;
            }
            break;
          default:
            break;
        }
        break;
      case 'command':
        break;
      default:
        break;
    }
    /* eslint-enable no-case-declarations */
  }

  kill(signal?: string) {
    this.cli.close();
    removeListener(`${this.tmpdir}fig.socket`, this.socketId);
    this.pty.kill(signal);
  }
}

export default Shell;
