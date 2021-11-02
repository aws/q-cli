import { readdirSync, PathLike } from 'fs';
import path from 'path';
import Shell from '../shell';
import Config from '../config';
import Settings from '../settings';

const getDirectories = (source: PathLike) =>
  readdirSync(source, { withFileTypes: true })
    .filter(dirent => dirent.isDirectory())
    .map(dirent => dirent.name);

const getAbsolutePath = (relative: string) => path.resolve(relative);

const absolutePath = getAbsolutePath('configs');

const makeTestsForShell = (shell: Shell, name: string) => {
  describe(name, () => {
    let env: Record<string, string>;

    beforeAll(async () => {
      await shell.initialized();
      env = await shell.pty.getEnv();
    });

    afterAll(() => shell.kill());

    beforeEach(async () => {
      Settings.reset();
      Config.reset();
      // Reset terminal size and get a fresh prompt before each test.
      shell.pty.resize({ rows: 30, cols: 80 });
      shell.pty.write('\r');
      await shell.cli.waitForNextPrompt();
    });

    describe('Valid environment', () => {
      test('FIG_TERM=1', () => {
        expect(env.FIG_TERM).toBe('1');
      });

      test('FIG_CHECKED_PROMPTS=1', () => {
        expect(env.FIG_CHECKED_PROMPTS).toBe('1');
      });

      test('FIG_INTEGRATION_VERSION is correct', () => {
        expect(env.FIG_INTEGRATION_VERSION).toBe('5');
      });

      test('PATH contains ~/.fig/bin', () => {
        expect(env.PATH.includes('/.fig/bin')).toBe(true);
      });

      test('TTY var exists', () => {
        expect(env.TTY).not.toBeNull();
      });

      test('TTY equals output of tty command', async () => {
        expect(await shell.pty.executeAsync('tty')).toBe(env.TTY);
      });
    });

    describe('Exercise Figterm', () => {
      test('Type "echo hello world"', async () => {
        await shell.pty.typeAsync('echo hello world!');
        expect(shell.buffer).toBe('echo hello world!');
      });

      test('buffer should reset after typing a character', async () => {
        await shell.pty.typeAsync('\b');
        expect(shell.buffer).toBe('');
      });

      test.skip('buffer should be empty on new prompt.', async () => {
        await shell.pty.typeAsync('\b');
        expect(shell.buffer).toBe('');
      });

      test('Resize window (horizontal)', async () => {
        await shell.pty.typeAsync('echo testing');
        shell.pty.resize({ rows: 30, cols: 30 });
        await shell.pty.typeAsync('11');
        expect(shell.buffer).toBe('echo testing11');
      });

      test('Resize window (vertical)', async () => {
        await shell.pty.typeAsync('echo testing');
        shell.pty.resize({ rows: 15, cols: 80 });
        await shell.pty.typeAsync('111');
        expect(shell.buffer).toBe('echo testing111');
      });
    });
  });
};

describe('shell: bash', () => {
  Settings.reset();
  Config.reset();
  getDirectories(absolutePath).forEach(configDir => {
    const shell = new Shell({
      shell: 'bash',
      args: ['--init-file', `${absolutePath}/${configDir}/.bashrc`],
    });
    makeTestsForShell(shell, configDir);
  });
});

describe('shell: zsh', () => {
  getDirectories(absolutePath).forEach(configDir => {
    const shell = new Shell({
      shell: 'zsh',
      env: {
        ...process.env,
        ZDOTDIR: `/usr/home/app/configs/${configDir}`,
      },
    });
    makeTestsForShell(shell, configDir);
  });
});
