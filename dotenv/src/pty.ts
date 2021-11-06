import * as pty from 'node-pty';

export type PTYOptions = {
  shell: string;
  args?: string | string[];
  env?: { [key: string]: string };
  mockedCLICommands?: Record<string, string>;
};

function randomId(len: number) {
  const alphabet = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ';
  let id = '';
  for (let i = 0; i < len; i += 1) {
    const idx = Math.floor(Math.random() * alphabet.length);
    id += alphabet[idx];
  }
  return id;
}

const delimeter = '-----';
const CRLF = '\r\n';
const generatePattern = (handlerId: string) => {
  const pattern = `${delimeter}${handlerId}${delimeter}`;
  return `<<<${pattern}${CRLF}(.*?)${CRLF}${pattern}>>>`;
};

class PTY {
  handlers: Record<string, (res: string) => void> = {};

  watchers: Record<string, () => void> = {};

  internalPty: pty.IPty;

  buffer = '';

  sessionBuffer = '';

  debug = false;

  shouldMockFigCLI = true;

  shouldRelaunchFigTerm = true;

  shouldRemoveAllFigEnvironmentVariables = true;

  constructor({ shell, args, env, mockedCLICommands }: PTYOptions) {
    if (this.debug) {
      console.log('Launching PTY...');
    }
    this.buffer = '';

    const environment = { ...(env ?? process.env) };

    if (this.shouldMockFigCLI && mockedCLICommands) {
      environment.PATH = `${__dirname}/bin:${environment.PATH}`;

      Object.keys(mockedCLICommands).forEach(key => {
        environment[`MOCK_${key.replaceAll(':', '_')}`] =
          mockedCLICommands[key];
      });
    }

    if (this.shouldRelaunchFigTerm) {
      delete environment.FIG_TERM;
    }

    if (this.shouldRemoveAllFigEnvironmentVariables) {
      Object.keys(environment).forEach(key => {
        if (key.startsWith('FIG')) {
          delete environment[key];
        }
      });
    }

    const finalEnv = {
      ...environment,
      FIG_SHELL_EXTRA_ARGS: Array.isArray(args) ? args.join(' ') : args ?? '',
    };

    this.internalPty = pty.spawn(shell, args ?? [], {
      name: 'xterm-color',
      cols: 80,
      rows: 30,
      cwd: process.env.HOME,
      env: finalEnv,
    });

    this.internalPty.on('data', data => {
      this.buffer += data;
      this.sessionBuffer += data;
      for (const pattern of Object.keys(this.watchers)) {
        if (this.buffer.includes(pattern)) {
          const callback = this.watchers[pattern];

          if (callback) {
            callback();
          }
        }
      }

      for (const handlerId of Object.keys(this.handlers)) {
        const regex = new RegExp(generatePattern(handlerId), 'ms');

        if (regex.test(this.buffer)) {
          if (this.debug) {
            console.log(`Found match! ${handlerId}`);
          }

          const groups = this.buffer.match(regex);
          const out = groups?.[1] ?? '';
          const callback = this.handlers[handlerId];

          if (callback) {
            callback(out);
          }
          delete this.handlers[handlerId];
          this.buffer = '';
        }
      }
    });
  }

  write(text: string) {
    this.internalPty.write(text);
  }

  type(text: string, completion: () => void) {
    const chars = text.split('');
    const interval = setInterval(() => {
      if (chars.length === 0) {
        completion();
        clearInterval(interval);
        return;
      }
      const c = chars.shift();
      this.write(c ?? '');
    }, 25);
  }

  execute(command: string, callback: (out: string) => void) {
    if (this.debug) {
      console.log('Running ', command);
    }

    const handlerId = randomId(5);
    this.handlers[handlerId] = callback;
    const wrapper = `${delimeter}${handlerId}${delimeter}`;
    this.internalPty.write(
      `printf "<<<" ; echo "${wrapper}" ; ${command} ; echo "${wrapper}>>>"\r`
    );
  }

  watch(pattern: string, callback: () => void) {
    this.watchers[pattern] = callback;
  }

  resize({ rows, cols }: { rows: number; cols: number }) {
    this.internalPty.resize(cols, rows);
  }

  kill(signal?: string | undefined) {
    if (this.internalPty) {
      this.internalPty.kill(signal);
    }
  }

  executeAsync(command: string) {
    return new Promise<string>(resolve => {
      this.execute(command, (out: string) => {
        setTimeout(() => {
          resolve(out);
        }, 10);
      });
    });
  }

  async typeAsync(text: string) {
    return new Promise<void>(resolve => {
      this.type(text, () => {
        resolve();
      });
    });
  }

  async getEnv() {
    const rawEnv = await this.executeAsync('env');
    const env = rawEnv.split(CRLF).reduce((dict, line) => {
      const split = line.indexOf('=');

      const key = line.substring(0, split);
      const value = line.substring(split + 1);
      // eslint-disable-next-line no-param-reassign
      dict[key] = value;
      return dict;
    }, {} as Record<string, string>);
    return env;
  }

  getSessionTranscript() {
    return this.sessionBuffer;
  }

  // eslint-disable-next-line class-methods-use-this
  delay(timeout: number) {
    return new Promise(resolve => {
      setTimeout(resolve, timeout);
    });
  }
}

export default PTY;
