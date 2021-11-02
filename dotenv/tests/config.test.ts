import Shell from '../shell';
import Config from '../config';
import Settings from '../settings';

let shell: Shell;

const UPDATE_ALERT = 'Updating .*? to latest version...';
const AUTOUPDATE_TEXT =
  '(To turn off automatic updates, run `fig settings app.disableAutoupdates true`)';

beforeEach(async () => {
  Config.reset();
  Settings.reset();
});

afterEach(() => {
  shell.kill();
});

describe('Testing ~/.fig/user/config', () => {
  test('New version available (show hint)', async () => {
    Config.set({ NEW_VERSION_AVAILABLE: 'v1.0.49' });
    shell = new Shell({
      shell: 'bash',
      mockedCLICommands: { 'app:running': '1' },
    });
    await shell.initialized();

    const transcript = shell.pty.getSessionTranscript();
    expect(transcript.match(UPDATE_ALERT)).toBeTruthy();
    expect(transcript.includes(AUTOUPDATE_TEXT)).toBeTruthy();

    expect(shell.cli.assertCommandRan('fig app:running')).toBe(true);
    expect(shell.cli.assertCommandRan('fig update:app --force')).toBe(true);

    expect(Config.getValue('DISPLAYED_AUTOUPDATE_SETTINGS_HINT')).toBe('1');
  });

  test('New version available (do not show hint)', async () => {
    Config.set({
      NEW_VERSION_AVAILABLE: 'v1.0.49',
      DISPLAYED_AUTOUPDATE_SETTINGS_HINT: '1',
    });

    shell = new Shell({
      shell: 'bash',
      mockedCLICommands: { 'app:running': '1' },
    });
    await shell.initialized();

    const transcript = shell.pty.getSessionTranscript();
    expect(transcript.match(UPDATE_ALERT)).toBeTruthy();
    expect(transcript.includes(AUTOUPDATE_TEXT)).toBe(false);

    expect(shell.cli.assertCommandRan('fig app:running')).toBe(true);
    expect(shell.cli.assertCommandRan('fig update:app --force')).toBe(true);
  });

  test('New version available (app not running)', async () => {
    Config.set({ NEW_VERSION_AVAILABLE: 'v1.0.49' });
    shell = new Shell({
      shell: 'bash',
      mockedCLICommands: { 'app:running': '' },
    });
    await shell.initialized();

    const transcript = shell.pty.getSessionTranscript();
    expect(transcript.match(UPDATE_ALERT)).toBeNull();
    expect(transcript.includes(AUTOUPDATE_TEXT)).toBe(false);

    expect(shell.cli.assertCommandRan('fig app:running')).toBe(true);
    expect(shell.cli.assertCommandDidNotRun('fig update:app --force')).toBe(
      true
    );
  });

  test('New version available. Autoupdates disabled.', async () => {
    Config.set({ NEW_VERSION_AVAILABLE: 'v1.0.49' });
    Settings.set({ 'app.disableAutoupdates': true });

    shell = new Shell({
      shell: 'bash',
      mockedCLICommands: { 'app:running': '1' },
    });
    await shell.initialized();

    const transcript = shell.pty.getSessionTranscript();
    expect(transcript.match('A new version of .*? is available.')).toBeTruthy();
    expect(transcript.includes(AUTOUPDATE_TEXT)).toBe(false);

    expect(shell.cli.assertCommandRan('fig app:running')).toBe(true);
    expect(
      shell.cli.assertCommandRan('fig settings app.disableAutoupdates')
    ).toBe(true);
    expect(shell.cli.assertCommandDidNotRun('fig update:app --force')).toBe(
      true
    );
  });
});
