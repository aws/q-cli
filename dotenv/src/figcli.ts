import { socketListen, removeListener } from './unix-server';

type CommandObject = {
  command: string;
  full: string;
  termSessionId: string;
  tty: string;
  pid: string;
  timestamp: number;
};

const parseFigCLIMessage = (data: Buffer): CommandObject => {
  const tokens = String(Buffer.from(data.toString(), 'base64'))
    .slice(0, -1)
    .split(' ');
  return {
    termSessionId: tokens[1],
    command: tokens[2],
    full: `fig ${tokens.slice(2).join(' ')}`,
    pid: tokens[3],
    tty: tokens[4],
    timestamp: Date.now(),
  };
};

class FigCLI {
  mockedCommands: Record<string, string> = {};

  commands: CommandObject[] = [];

  socketId: string;

  constructor(sessionId: string, path = '/tmp/mock_figcli.socket') {
    this.socketId = socketListen(path, data => {
      const message = parseFigCLIMessage(data);
      if (message.termSessionId !== sessionId) return;
      this.commands.push(message);
    });
  }

  mockCommand({ command, output }: { command: string; output: string }) {
    this.mockedCommands[command] = output;
  }

  assertCommandRan(
    command: string,
    options = { count: 1, ignoringArguments: false }
  ) {
    return (
      this.commands.filter((commandObject: CommandObject) => {
        if (options.ignoringArguments) {
          return commandObject.command === `fig ${command}`;
        }
        return commandObject.full === command;
      }).length === options.count
    );
  }

  assertCommandDidNotRun(
    command: string,
    options?: { ignoringArguments?: boolean }
  ) {
    return this.assertCommandRan(command, {
      ignoringArguments: options?.ignoringArguments ?? false,
      count: 0,
    });
  }

  close() {
    removeListener('/tmp/mock_figcli.socket', this.socketId);
  }

  reset() {
    this.commands = [];
  }
}

export default FigCLI;
